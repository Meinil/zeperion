use std::{
    collections::HashMap,
    path::Path,
    path::PathBuf,
    process::Stdio,
    sync::{Mutex, OnceLock},
};

use anyhow::{Result, anyhow};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    domain::{
        account::Account, download::RemoteVersionDetails, game_version::InstalledGameVersion,
    },
    infrastructure::{db, fs::AppPaths},
    state::AppStore,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchValidation {
    Ready,
    MissingAccount,
    MissingVersion,
    IncompleteVersion,
    MissingJava,
    IncompatibleJava,
}

#[derive(Debug, Clone)]
pub struct LaunchPlan {
    pub db_path: String,
    pub launch_id: String,
    pub version_name: String,
    pub account_name: String,
    pub java_path: String,
    pub working_dir: String,
    pub args: Vec<String>,
    pub command_line: String,
    pub log_path: String,
}

#[derive(Debug, Clone)]
pub struct LaunchExecutionResult {
    pub log_path: String,
    pub exit_code: Option<i32>,
}

pub struct RunningLaunch {
    pub launch_id: String,
    pub log_path: String,
    pub waiter: tokio::task::JoinHandle<Result<LaunchExecutionResult>>,
}

#[derive(Debug, Clone, Copy)]
struct ActiveLaunchRuntime {
    pid: u32,
}

static ACTIVE_LAUNCHES: OnceLock<Mutex<HashMap<String, ActiveLaunchRuntime>>> = OnceLock::new();

fn active_launches() -> &'static Mutex<HashMap<String, ActiveLaunchRuntime>> {
    ACTIVE_LAUNCHES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn validate_launch(store: &AppStore) -> LaunchValidation {
    validate_launch_with_options(store, false)
}

pub fn validate_launch_with_options(
    store: &AppStore,
    allow_incompatible_java: bool,
) -> LaunchValidation {
    if store.selected_account().is_none() {
        return LaunchValidation::MissingAccount;
    }

    let Some(version) = store.selected_version() else {
        return LaunchValidation::MissingVersion;
    };

    if !version.is_launchable() {
        return LaunchValidation::IncompleteVersion;
    }

    let Some(java) = store.default_java() else {
        return LaunchValidation::MissingJava;
    };

    if java.version.is_empty() {
        return LaunchValidation::IncompatibleJava;
    }

    if !allow_incompatible_java {
        if let Some(required_major) = required_java_major_for_version(&version.version_name) {
            if let Some(actual_major) = parse_java_major_version(&java.version) {
                if actual_major < required_major {
                    return LaunchValidation::IncompatibleJava;
                }
            }
        }
    }

    LaunchValidation::Ready
}

pub fn java_compatibility_summary(store: &AppStore) -> Option<String> {
    let version = store.selected_version()?;
    let java = store.default_java()?;
    let required_major = required_java_major_for_version(&version.version_name)?;
    let actual_major = parse_java_major_version(&java.version)?;

    Some(format!(
        "Minecraft {} requires Java {}+, current Java is {} ({})",
        version.version_name, required_major, actual_major, java.version
    ))
}

pub async fn prepare_launch_plan(store: &AppStore, paths: &AppPaths) -> Result<LaunchPlan> {
    let account = store
        .selected_account()
        .ok_or_else(|| anyhow!("missing selected account"))?;
    let version = store
        .selected_version()
        .ok_or_else(|| anyhow!("missing selected version"))?;
    let java = store
        .default_java()
        .ok_or_else(|| anyhow!("missing default java"))?;

    let manifest = load_version_manifest(version).await?;
    let main_class = manifest
        .main_class
        .clone()
        .ok_or_else(|| anyhow!("manifest does not contain mainClass"))?;

    let version_dir = PathBuf::from(&version.install_dir);
    let client_jar = version_dir.join("client.jar");
    if !client_jar.exists() {
        return Err(anyhow!("client.jar is missing"));
    }

    let libraries_dir = version_dir.join("libraries");
    let classpath = build_classpath(&client_jar, &libraries_dir)?;
    let natives_dir = version_dir.join("natives");
    tokio::fs::create_dir_all(&natives_dir).await?;
    let game_dir = version_dir.join("game");
    tokio::fs::create_dir_all(&game_dir).await?;

    let assets_dir = version_dir.join("assets");
    tokio::fs::create_dir_all(&assets_dir).await?;
    let asset_index = manifest
        .asset_index
        .as_ref()
        .map(|index| index.id.clone())
        .or_else(|| manifest.assets.clone())
        .unwrap_or_else(|| "legacy".to_string());

    let mut args = Vec::new();
    if let Some(flag) = start_on_first_thread_flag() {
        args.push(flag);
    }
    args.extend([
        format!("-Xmx{}M", store.settings.game_memory_mb),
        format!("-Djava.library.path={}", natives_dir.display()),
    ]);
    args.extend(extra_jvm_arguments(account, paths)?);
    args.extend(["-cp".to_string(), classpath, main_class]);
    args.extend(game_arguments(
        account,
        version,
        &game_dir,
        &assets_dir,
        &asset_index,
    ));

    let command_line = format!(
        "{} {}",
        java.path,
        args.iter()
            .map(|arg| shell_escape(arg))
            .collect::<Vec<_>>()
            .join(" ")
    );

    let log_path = paths.logs_dir().join(format!(
        "launch-plan-{}.log",
        Utc::now().format("%Y%m%d-%H%M%S")
    ));
    let content = format!(
        "Zeperion Launcher Launch Plan\nversion={}\naccount={}\njava={}\nworking_dir={}\ncommand={}\n",
        version.version_name,
        account.username,
        java.path,
        game_dir.display(),
        command_line
    );
    tokio::fs::write(&log_path, content).await?;

    Ok(LaunchPlan {
        db_path: paths.db_file().display().to_string(),
        launch_id: Uuid::new_v4().to_string(),
        version_name: version.version_name.clone(),
        account_name: account.username.clone(),
        java_path: java.path.clone(),
        working_dir: game_dir.display().to_string(),
        args,
        command_line,
        log_path: log_path.display().to_string(),
    })
}

pub async fn spawn_launch_process(plan: &LaunchPlan) -> Result<RunningLaunch> {
    let mut history = db::launch_history::LaunchHistoryRecord {
        id: plan.launch_id.clone(),
        version_name: plan.version_name.clone(),
        account_name: plan.account_name.clone(),
        java_path: plan.java_path.clone(),
        command_line: plan.command_line.clone(),
        log_path: plan.log_path.clone(),
        status: "running".to_string(),
        exit_code: None,
        error_message: None,
        launched_at: Utc::now(),
        finished_at: None,
    };
    let pool = db::connect(Path::new(&plan.db_path)).await.ok();
    if let Some(pool) = &pool {
        let _ = db::launch_history::upsert(pool, &history).await;
    }

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&plan.log_path)?;
    let stdout_file = log_file.try_clone()?;
    let stderr_file = log_file.try_clone()?;

    let mut command = tokio::process::Command::new(&plan.java_path);
    command
        .args(&plan.args)
        .current_dir(&plan.working_dir)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));

    match command.spawn() {
        Ok(mut child) => {
            let pid = child
                .id()
                .ok_or_else(|| anyhow!("failed to determine launch process id"))?;
            active_launches()
                .lock()
                .expect("active launches poisoned")
                .insert(plan.launch_id.clone(), ActiveLaunchRuntime { pid });

            let launch_id = plan.launch_id.clone();
            let log_path = plan.log_path.clone();
            let waiter = tokio::spawn(async move {
                let status = child.wait().await;
                active_launches()
                    .lock()
                    .expect("active launches poisoned")
                    .remove(&launch_id);

                match status {
                    Ok(status) => {
                        history.status = "completed".to_string();
                        history.exit_code = status.code();
                        history.finished_at = Some(Utc::now());
                        if let Some(pool) = &pool {
                            let _ = db::launch_history::upsert(pool, &history).await;
                        }
                        Ok(LaunchExecutionResult {
                            log_path: log_path.clone(),
                            exit_code: status.code(),
                        })
                    }
                    Err(error) => {
                        history.status = "failed".to_string();
                        history.error_message = Some(error.to_string());
                        history.finished_at = Some(Utc::now());
                        if let Some(pool) = &pool {
                            let _ = db::launch_history::upsert(pool, &history).await;
                        }
                        Err(error.into())
                    }
                }
            });

            Ok(RunningLaunch {
                launch_id: plan.launch_id.clone(),
                log_path: plan.log_path.clone(),
                waiter,
            })
        }
        Err(error) => {
            history.status = "failed".to_string();
            history.error_message = Some(error.to_string());
            history.finished_at = Some(Utc::now());
            if let Some(pool) = &pool {
                let _ = db::launch_history::upsert(pool, &history).await;
            }
            Err(error.into())
        }
    }
}

pub async fn stop_launch(launch_id: &str) -> Result<()> {
    let pid = active_launches()
        .lock()
        .expect("active launches poisoned")
        .get(launch_id)
        .copied()
        .map(|runtime| runtime.pid)
        .ok_or_else(|| anyhow!("launch process is not running"))?;

    stop_process(pid)
}

fn stop_process(pid: u32) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()?;
        if status.success() {
            return Ok(());
        }
        return Err(anyhow!("failed to stop process {}", pid));
    }

    #[cfg(not(target_os = "windows"))]
    {
        let status = std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()?;
        if status.success() {
            return Ok(());
        }
        Err(anyhow!("failed to stop process {}", pid))
    }
}

async fn load_version_manifest(version: &InstalledGameVersion) -> Result<RemoteVersionDetails> {
    let manifest_path = Path::new(&version.install_dir).join("manifest.json");
    let bytes = tokio::fs::read(manifest_path).await?;
    let manifest = serde_json::from_slice::<RemoteVersionDetails>(&bytes)?;
    Ok(manifest)
}

fn build_classpath(client_jar: &Path, libraries_dir: &Path) -> Result<String> {
    let mut entries = vec![client_jar.display().to_string()];
    if libraries_dir.exists() {
        collect_jars(libraries_dir, &mut entries)?;
    }
    Ok(entries.join(classpath_separator()))
}

fn collect_jars(dir: &Path, entries: &mut Vec<String>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_jars(&path, entries)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("jar") {
            entries.push(path.display().to_string());
        }
    }
    Ok(())
}

fn game_arguments(
    account: &Account,
    version: &InstalledGameVersion,
    game_dir: &Path,
    assets_dir: &Path,
    asset_index: &str,
) -> Vec<String> {
    vec![
        "--username".to_string(),
        account.username.clone(),
        "--version".to_string(),
        version.version_name.clone(),
        "--gameDir".to_string(),
        game_dir.display().to_string(),
        "--assetsDir".to_string(),
        assets_dir.display().to_string(),
        "--assetIndex".to_string(),
        asset_index.to_string(),
        "--uuid".to_string(),
        account.uuid.clone(),
        "--accessToken".to_string(),
        account.launch_access_token(),
        "--userType".to_string(),
        account.launch_user_type().to_string(),
        "--versionType".to_string(),
        version.version_type.clone(),
    ]
}

fn extra_jvm_arguments(account: &Account, paths: &AppPaths) -> Result<Vec<String>> {
    if account.login_type != crate::domain::account::LoginType::ThirdParty {
        return Ok(Vec::new());
    }

    let server = account
        .auth_server_url
        .clone()
        .ok_or_else(|| anyhow!("missing authlib server url for third-party account"))?;
    let metadata = account
        .authlib_metadata_b64
        .clone()
        .ok_or_else(|| anyhow!("missing authlib metadata for third-party account"))?;
    let agent_jar = paths.authlib_injector_jar();
    if !agent_jar.exists() {
        return Err(anyhow!(
            "authlib-injector.jar is missing from {}",
            paths.runtime_dir().display()
        ));
    }

    Ok(vec![
        format!("-javaagent:{}={server}", agent_jar.display()),
        format!("-Dauthlibinjector.yggdrasil.prefetched={metadata}"),
    ])
}

fn start_on_first_thread_flag() -> Option<String> {
    if cfg!(target_os = "macos") {
        Some("-XstartOnFirstThread".to_string())
    } else {
        None
    }
}

fn classpath_separator() -> &'static str {
    if cfg!(windows) { ";" } else { ":" }
}

fn shell_escape(value: &str) -> String {
    if value.contains(' ') {
        format!("\"{}\"", value)
    } else {
        value.to_string()
    }
}

fn required_java_major_for_version(version_name: &str) -> Option<u32> {
    let normalized = version_name.trim().trim_start_matches('v');
    let release = normalized.split('-').next().unwrap_or(normalized);
    let mut segments = release.split('.');
    let major = segments.next()?.parse::<u32>().ok()?;
    let minor = segments
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or_default();
    let patch = segments
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or_default();

    if major > 1 {
        return Some(21);
    }

    if minor > 20 || (minor == 20 && patch >= 5) {
        Some(21)
    } else if minor >= 18 {
        Some(17)
    } else if minor == 17 {
        Some(16)
    } else {
        Some(8)
    }
}

fn parse_java_major_version(version: &str) -> Option<u32> {
    let normalized = version.trim();
    if let Some(rest) = normalized.strip_prefix("1.") {
        return rest
            .split(|ch: char| !ch.is_ascii_digit())
            .find(|part| !part.is_empty())
            .and_then(|part| part.parse::<u32>().ok());
    }

    normalized
        .split(|ch: char| !ch.is_ascii_digit())
        .find(|part| !part.is_empty())
        .and_then(|part| part.parse::<u32>().ok())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{
        LaunchValidation, parse_java_major_version, required_java_major_for_version,
        validate_launch, validate_launch_with_options,
    };
    use crate::{
        domain::{
            account::Account,
            game_version::InstalledGameVersion,
            java::{JavaRuntime, JavaSource},
        },
        state::AppStore,
    };

    fn base_store() -> AppStore {
        let mut store = AppStore::default();
        let mut account = Account::offline("player".to_string());
        account.selected = true;
        store.accounts.push(account);
        store.java_runtimes.push(JavaRuntime {
            id: "java-1".to_string(),
            name: "Java 21".to_string(),
            version: "21".to_string(),
            path: "/usr/bin/java".to_string(),
            source: JavaSource::AutoScan,
            is_default: true,
            created_at: Utc::now(),
        });
        store
    }

    #[test]
    fn rejects_incomplete_version() {
        let mut store = base_store();
        store.versions.push(InstalledGameVersion {
            id: "v1".to_string(),
            version_name: "1.20.1".to_string(),
            version_type: "release".to_string(),
            release_time: Utc::now(),
            install_dir: "/tmp/1.20.1".to_string(),
            is_downloaded: true,
            integrity_status: "metadata_only".to_string(),
        });
        store.selected_version_id = Some("v1".to_string());

        assert_eq!(validate_launch(&store), LaunchValidation::IncompleteVersion);
    }

    #[test]
    fn accepts_complete_version() {
        let mut store = base_store();
        store.versions.push(InstalledGameVersion {
            id: "v1".to_string(),
            version_name: "1.20.1".to_string(),
            version_type: "release".to_string(),
            release_time: Utc::now(),
            install_dir: "/tmp/1.20.1".to_string(),
            is_downloaded: true,
            integrity_status: "complete".to_string(),
        });
        store.selected_version_id = Some("v1".to_string());

        assert_eq!(validate_launch(&store), LaunchValidation::Ready);
    }

    #[test]
    fn rejects_incompatible_java_by_rule() {
        let mut store = base_store();
        store.versions.push(InstalledGameVersion {
            id: "v1".to_string(),
            version_name: "1.20.6".to_string(),
            version_type: "release".to_string(),
            release_time: Utc::now(),
            install_dir: "/tmp/1.20.6".to_string(),
            is_downloaded: true,
            integrity_status: "complete".to_string(),
        });
        store.selected_version_id = Some("v1".to_string());
        store.java_runtimes[0].version = "17.0.10".to_string();

        assert_eq!(validate_launch(&store), LaunchValidation::IncompatibleJava);
        assert_eq!(
            validate_launch_with_options(&store, true),
            LaunchValidation::Ready
        );
    }

    #[test]
    fn parses_java_major_versions() {
        assert_eq!(parse_java_major_version("1.8.0_412"), Some(8));
        assert_eq!(parse_java_major_version("17.0.10"), Some(17));
        assert_eq!(parse_java_major_version("21"), Some(21));
    }

    #[test]
    fn maps_required_java_versions() {
        assert_eq!(required_java_major_for_version("1.16.5"), Some(8));
        assert_eq!(required_java_major_for_version("1.17.1"), Some(16));
        assert_eq!(required_java_major_for_version("1.20.4"), Some(17));
        assert_eq!(required_java_major_for_version("1.20.6"), Some(21));
    }
}

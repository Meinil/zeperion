use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    process::Command as StdCommand,
};

use anyhow::{Result, anyhow};
use tokio::process::Command;
use uuid::Uuid;

use crate::domain::java::{JavaRuntime, JavaSource};

pub async fn scan_java_runtimes() -> Vec<JavaRuntime> {
    let mut candidates = BTreeSet::new();

    if let Some(path) = find_in_path() {
        candidates.insert(path);
    }

    for extra in common_java_locations() {
        if extra.exists() {
            candidates.insert(extra);
        }
    }

    let mut runtimes = Vec::new();
    for candidate in candidates {
        if let Some(runtime) = inspect_java_binary(&candidate).await {
            runtimes.push(runtime);
        }
    }

    if let Some(first) = runtimes.first_mut() {
        first.is_default = true;
    }

    runtimes
}

fn find_in_path() -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    let executable = if cfg!(windows) { "java.exe" } else { "java" };

    env::split_paths(&path_var)
        .map(|dir| dir.join(executable))
        .find(|candidate| candidate.exists())
}

fn common_java_locations() -> Vec<PathBuf> {
    let mut items = Vec::new();

    if cfg!(target_os = "macos") {
        items.push(PathBuf::from("/usr/bin/java"));
        items.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
    }

    if cfg!(target_os = "linux") {
        items.push(PathBuf::from("/usr/bin/java"));
        items.push(PathBuf::from("/usr/lib/jvm/default-java/bin/java"));
    }

    if cfg!(windows) {
        items.push(PathBuf::from(r"C:\Program Files\Java"));
        items.push(PathBuf::from(r"C:\Program Files\Eclipse Adoptium"));
    }

    expand_java_homes(items)
}

fn expand_java_homes(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut expanded = Vec::new();
    for path in paths {
        if path.is_file() {
            expanded.push(path);
            continue;
        }

        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let root = entry.path();
                    let binary = if cfg!(windows) {
                        root.join("bin").join("java.exe")
                    } else {
                        root.join("bin").join("java")
                    };

                    if binary.exists() {
                        expanded.push(binary);
                    }
                }
            }
        }
    }

    expanded
}

async fn inspect_java_binary(path: &Path) -> Option<JavaRuntime> {
    let output = Command::new(path).arg("-version").output().await.ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let version_line = stderr.lines().next().unwrap_or("Unknown Java");
    let version = extract_version(version_line);

    Some(JavaRuntime::scanned(
        "Local Java".to_string(),
        version,
        path.display().to_string(),
    ))
}

pub async fn import_java_from_path(path: &Path) -> Option<JavaRuntime> {
    let binary = resolve_java_binary(path)?;
    let mut runtime = inspect_java_binary(&binary).await?;
    runtime.name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "Imported Java".to_string());
    runtime.path = binary.display().to_string();
    runtime.source = JavaSource::ImportedFolder;
    Some(runtime)
}

pub async fn import_java_from_archive(
    archive: &Path,
    install_root: &Path,
) -> Result<Option<JavaRuntime>> {
    if !is_supported_archive(archive) {
        return Ok(None);
    }

    let archive_root = install_root
        .join("archives")
        .join(Uuid::new_v4().to_string());
    tokio::fs::create_dir_all(&archive_root).await?;

    if let Err(error) = extract_archive(archive, &archive_root) {
        let _ = tokio::fs::remove_dir_all(&archive_root).await;
        return Err(error);
    }

    let Some(binary) = discover_java_binary(&archive_root) else {
        let _ = tokio::fs::remove_dir_all(&archive_root).await;
        return Ok(None);
    };

    let mut runtime = match inspect_java_binary(&binary).await {
        Some(runtime) => runtime,
        None => {
            let _ = tokio::fs::remove_dir_all(&archive_root).await;
            return Ok(None);
        }
    };

    runtime.name = archive
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Imported Java Archive".to_string());
    runtime.path = binary.display().to_string();
    runtime.source = JavaSource::ImportedArchive;
    Ok(Some(runtime))
}

pub fn open_url(url: &str) -> Result<()> {
    let status = if cfg!(target_os = "macos") {
        StdCommand::new("open").arg(url).status()?
    } else if cfg!(windows) {
        StdCommand::new("cmd")
            .args(["/C", "start", "", url])
            .status()?
    } else {
        StdCommand::new("xdg-open").arg(url).status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("failed to open external url"))
    }
}

fn resolve_java_binary(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }

    if !path.is_dir() {
        return None;
    }

    let candidate = if cfg!(windows) {
        path.join("bin").join("java.exe")
    } else {
        path.join("bin").join("java")
    };

    candidate.exists().then_some(candidate)
}

fn discover_java_binary(path: &Path) -> Option<PathBuf> {
    resolve_java_binary(path).or_else(|| find_java_binary_recursive(path))
}

fn find_java_binary_recursive(path: &Path) -> Option<PathBuf> {
    if !path.is_dir() {
        return None;
    }

    let entries = fs::read_dir(path).ok()?;
    for entry in entries.flatten() {
        let item = entry.path();
        if let Some(binary) = resolve_java_binary(&item) {
            return Some(binary);
        }
        if let Some(binary) = find_java_binary_recursive(&item) {
            return Some(binary);
        }
    }

    None
}

fn is_supported_archive(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "zip" | "jar"))
        .unwrap_or(false)
}

fn extract_archive(archive: &Path, target_dir: &Path) -> Result<()> {
    let status = if cfg!(target_os = "macos") {
        StdCommand::new("ditto")
            .arg("-x")
            .arg("-k")
            .arg(archive)
            .arg(target_dir)
            .status()?
    } else if cfg!(windows) {
        let archive_path = archive.display().to_string().replace('\'', "''");
        let target_path = target_dir.display().to_string().replace('\'', "''");
        let script = format!(
            "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
            archive_path, target_path
        );
        StdCommand::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .status()?
    } else {
        StdCommand::new("unzip")
            .arg("-q")
            .arg(archive)
            .arg("-d")
            .arg(target_dir)
            .status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("failed to extract Java archive"))
    }
}

fn extract_version(line: &str) -> String {
    let quoted = line.split('"').nth(1).map(str::to_string);
    quoted.unwrap_or_else(|| line.to_string())
}

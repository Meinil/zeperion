use std::{
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    process::Command,
    time::Duration,
};

use anyhow::{Result, anyhow};
use dioxus::prelude::*;
use dioxus_core::spawn_forever;
use dioxus_router::use_navigator;
use reqwest::Url;
use rfd::AsyncFileDialog;
use uuid::Uuid;

use crate::{
    app::{bootstrap::AppServices, root::tr, routes::Route},
    components::{
        confirm_modal::ConfirmModal,
        side_menu::{SideMenu, SideMenuItem},
    },
    domain::{
        account::{Account, LoginType, OAuthFlow},
        background::{BackgroundImage, BackgroundSource},
        game_version::InstalledGameVersion,
    },
    infrastructure::{
        db, integrity,
        launcher::LaunchValidation,
        network::{
            authlib::AuthlibClient,
            microsoft_auth::{
                MicrosoftAuthClient, MicrosoftAuthConfig, MicrosoftPkcePair, generate_pkce_pair,
            },
        },
    },
    state::{AddOfflineAccountError, AppStore},
};

const MICROSOFT_CALLBACK_PORT: u16 = 28765;
const MICROSOFT_CALLBACK_PATH: &str = "/callback";

#[component]
pub fn HomePage(section: HomeSection) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let navigator = use_navigator();
    let desktop = dioxus::desktop::use_window();
    let show_java_compat_modal = use_signal(|| false);
    let show_stop_launch_modal = use_signal(|| false);
    let language = store.read().settings.language.clone();
    let launch_desktop = desktop.clone();
    let compatibility_desktop = desktop.clone();
    let validation_message = current_validation_message(&services, &store.read());
    let compatibility_fallback = validation_message.clone();
    let java_compatibility_summary =
        crate::infrastructure::launcher::java_compatibility_summary(&store.read());

    let items = vec![
        SideMenuItem {
            key: "accounts".into(),
            label: tr(&services, &language, "home.menu.accounts"),
            active: section == HomeSection::Accounts,
            on_click: {
                let navigator = navigator.clone();
                EventHandler::new(move |_| {
                    navigator.push(Route::HomeAccounts {});
                })
            },
        },
        SideMenuItem {
            key: "versions".into(),
            label: tr(&services, &language, "home.menu.versions"),
            active: section == HomeSection::Versions,
            on_click: {
                let navigator = navigator.clone();
                EventHandler::new(move |_| {
                    navigator.push(Route::HomeVersions {});
                })
            },
        },
        SideMenuItem {
            key: "backgrounds".into(),
            label: tr(&services, &language, "home.menu.backgrounds"),
            active: section == HomeSection::Backgrounds,
            on_click: {
                let navigator = navigator.clone();
                EventHandler::new(move |_| {
                    navigator.push(Route::HomeBackgrounds {});
                })
            },
        },
    ];

    rsx! {
        div { class: "two-pane",
            SideMenu { items }
            section { class: "content-pane",
                div { class: "launch-panel",
                    div {
                        h2 { {tr(&services, &language, "home.launch.title")} }
                        p { {tr(&services, &language, "home.launch.subtitle")} }
                    }
                    div { class: "launch-summary",
                        SummaryCard {
                            label: tr(&services, &language, "home.menu.accounts"),
                            value: store.read().selected_account().map(|a| a.username.clone()).unwrap_or_else(|| tr(&services, &language, "home.launch.no_account"))
                        }
                        SummaryCard {
                            label: tr(&services, &language, "home.menu.versions"),
                            value: store.read().selected_version().map(|v| v.version_name.clone()).unwrap_or_else(|| tr(&services, &language, "home.launch.no_version"))
                        }
                        SummaryCard {
                            label: "Java".to_string(),
                            value: store.read().default_java().map(|j| format!("{} ({})", j.name, j.version)).unwrap_or_else(|| tr(&services, &language, "home.launch.no_java"))
                        }
                    }
                    button {
                        class: "launch-button",
                        onclick: {
                            let mut store = store;
                            let mut show_java_compat_modal = show_java_compat_modal;
                            let mut show_stop_launch_modal = show_stop_launch_modal;
                            let services = services.clone();
                            move |_| {
                                if store.read().active_launch_id.is_some() {
                                    show_stop_launch_modal.set(true);
                                    return;
                                }

                                let validation = {
                                    let snapshot = store.read();
                                    crate::infrastructure::launcher::validate_launch(&snapshot)
                                };
                                match validation {
                                    LaunchValidation::Ready => {
                                        start_launch(services.clone(), store, launch_desktop.clone(), false);
                                    }
                                    LaunchValidation::IncompatibleJava => {
                                        show_java_compat_modal.set(true);
                                    }
                                    _ => {
                                        let message = current_validation_message(&services, &store.read());
                                        store.write().notification = Some(message);
                                    }
                                }
                            }
                        },
                        {if store.read().is_launch_running() {
                            tr(&services, &language, "home.launch.stop")
                        } else {
                            tr(&services, &language, "home.launch.button")
                        }}
                    }
                    p { class: "inline-hint", {validation_message} }
                }
                if show_java_compat_modal() {
                    CompatibilityModal {
                        language: language.clone(),
                        summary: java_compatibility_summary.unwrap_or(compatibility_fallback),
                        on_continue: EventHandler::new({
                            let services = services.clone();
                            let store = store;
                            let mut show_java_compat_modal = show_java_compat_modal;
                            move |_| {
                                show_java_compat_modal.set(false);
                                start_launch(services.clone(), store, compatibility_desktop.clone(), true);
                            }
                        }),
                        on_adjust: EventHandler::new({
                            let navigator = navigator.clone();
                            let mut show_java_compat_modal = show_java_compat_modal;
                            move |_| {
                                show_java_compat_modal.set(false);
                                navigator.push(Route::SettingsJava {});
                            }
                        }),
                        on_close: EventHandler::new({
                            let mut show_java_compat_modal = show_java_compat_modal;
                            move |_| show_java_compat_modal.set(false)
                        })
                    }
                }
                if show_stop_launch_modal() {
                    ConfirmModal {
                        language: language.clone(),
                        title: tr(&services, &language, "home.launch.stop"),
                        message: tr(&services, &language, "home.launch.stop_confirm"),
                        confirm_label: tr(&services, &language, "home.launch.stop"),
                        on_close: EventHandler::new({
                            let mut show_stop_launch_modal = show_stop_launch_modal;
                            move |()| show_stop_launch_modal.set(false)
                        }),
                        on_confirm: EventHandler::new({
                            let services = services.clone();
                            let store = store;
                            let mut show_stop_launch_modal = show_stop_launch_modal;
                            move |()| {
                                show_stop_launch_modal.set(false);
                                if let Some(launch_id) = store.read().active_launch_id.clone() {
                                    let services = services.clone();
                                    let mut store = store;
                                    spawn(async move {
                                        match crate::infrastructure::launcher::stop_launch(&launch_id).await {
                                            Ok(()) => {
                                                let language = store.read().settings.language.clone();
                                                store.write().notification = Some(tr(
                                                    &services,
                                                    &language,
                                                    "notice.launch_stopping",
                                                ));
                                            }
                                            Err(error) => {
                                                let language = store.read().settings.language.clone();
                                                store.write().notification = Some(format!(
                                                    "{} {}",
                                                    tr(&services, &language, "notice.launch_stop_failed"),
                                                    error
                                                ));
                                            }
                                        }
                                    });
                                }
                            }
                        })
                    }
                }
                {render_home_detail(section)}
            }
        }
    }
}

fn current_validation_message(services: &std::sync::Arc<AppServices>, store: &AppStore) -> String {
    match crate::infrastructure::launcher::validate_launch(store) {
        LaunchValidation::Ready => tr(services, &store.settings.language, "status.ready"),
        LaunchValidation::MissingAccount => tr(
            services,
            &store.settings.language,
            "validation.missing_account",
        ),
        LaunchValidation::MissingVersion => tr(
            services,
            &store.settings.language,
            "validation.missing_version",
        ),
        LaunchValidation::IncompleteVersion => tr(
            services,
            &store.settings.language,
            "validation.incomplete_version",
        ),
        LaunchValidation::MissingJava => tr(
            services,
            &store.settings.language,
            "validation.missing_java",
        ),
        LaunchValidation::IncompatibleJava => tr(
            services,
            &store.settings.language,
            "validation.incompatible_java",
        ),
    }
}

fn start_launch(
    services: std::sync::Arc<AppServices>,
    mut store: Signal<AppStore>,
    desktop: dioxus::desktop::DesktopContext,
    allow_incompatible_java: bool,
) {
    let store_snapshot = store.read().clone();
    let validation = crate::infrastructure::launcher::validate_launch_with_options(
        &store_snapshot,
        allow_incompatible_java,
    );
    if validation != LaunchValidation::Ready {
        let message = current_validation_message(&services, &store.read());
        store.write().notification = Some(message);
        return;
    }

    spawn(async move {
        match crate::infrastructure::launcher::prepare_launch_plan(&store_snapshot, &services.paths)
            .await
        {
            Ok(plan) => {
                let language = store.read().settings.language.clone();
                store.write().notification = Some(format!(
                    "{} {}",
                    tr(&services, &language, "notice.launch_plan_ready"),
                    plan.log_path
                ));

                match crate::infrastructure::launcher::spawn_launch_process(&plan).await {
                    Ok(running) => {
                        store.write().set_active_launch(running.launch_id.clone());
                        let language = store.read().settings.language.clone();
                        store.write().notification = Some(format!(
                            "{} {}",
                            tr(&services, &language, "notice.launch_started"),
                            running.log_path
                        ));
                        desktop.set_minimized(true);

                        let services = services.clone();
                        let mut store = store;
                        spawn(async move {
                            match running.waiter.await {
                                Ok(Ok(result)) => {
                                    store.write().clear_active_launch();
                                    let language = store.read().settings.language.clone();
                                    let code = result
                                        .exit_code
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "terminated".to_string());
                                    store.write().notification = Some(format!(
                                        "{} {} ({})",
                                        tr(&services, &language, "notice.launch_exited"),
                                        code,
                                        result.log_path
                                    ));
                                }
                                Ok(Err(error)) => {
                                    store.write().clear_active_launch();
                                    let language = store.read().settings.language.clone();
                                    store.write().notification = Some(format!(
                                        "{} {}",
                                        tr(&services, &language, "notice.launch_spawn_failed"),
                                        error
                                    ));
                                }
                                Err(error) => {
                                    store.write().clear_active_launch();
                                    let language = store.read().settings.language.clone();
                                    store.write().notification = Some(format!(
                                        "{} {}",
                                        tr(&services, &language, "notice.launch_spawn_failed"),
                                        error
                                    ));
                                }
                            }
                        });
                    }
                    Err(error) => {
                        let language = store.read().settings.language.clone();
                        store.write().notification = Some(format!(
                            "{} {}",
                            tr(&services, &language, "notice.launch_spawn_failed"),
                            error
                        ));
                    }
                }
            }
            Err(error) => {
                let language = store.read().settings.language.clone();
                store.write().notification = Some(format!(
                    "{} {}",
                    tr(&services, &language, "notice.launch_plan_failed"),
                    error
                ));
            }
        }
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HomeSection {
    Accounts,
    Versions,
    Backgrounds,
}

fn render_home_detail(section: HomeSection) -> Element {
    let store = use_context::<Signal<AppStore>>();
    let language = store.read().settings.language.clone();

    match section {
        HomeSection::Accounts => rsx! { AccountsPanel { language } },
        HomeSection::Versions => rsx! { VersionsPanel { language } },
        HomeSection::Backgrounds => rsx! { BackgroundsPanel { language } },
    }
}

#[derive(Props, PartialEq, Clone)]
struct SummaryCardProps {
    label: String,
    value: String,
}

#[component]
fn SummaryCard(props: SummaryCardProps) -> Element {
    rsx! {
        div { class: "summary-card",
            span { class: "summary-label", {props.label} }
            strong { {props.value} }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct PanelProps {
    language: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AccountAddKind {
    Offline,
    Microsoft,
    ThirdParty,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OfflineSkinPreset {
    SteveDefault,
    Random,
    AlexSlim,
    AriSlim,
    EfeSlim,
    KaiSlim,
    MakenaSlim,
    NoorSlim,
    SteveSlim,
    SunnySlim,
    ZuriSlim,
    AlexBold,
    AriBold,
    EfeBold,
    KaiBold,
    MakenaBold,
    NoorBold,
    SteveBold,
    SunnyBold,
    ZuriBold,
}

impl OfflineSkinPreset {
    fn as_str(&self) -> &'static str {
        match self {
            Self::SteveDefault => "steve-default",
            Self::Random => "random",
            Self::AlexSlim => "alex-slim",
            Self::AriSlim => "ari-slim",
            Self::EfeSlim => "efe-slim",
            Self::KaiSlim => "kai-slim",
            Self::MakenaSlim => "makena-slim",
            Self::NoorSlim => "noor-slim",
            Self::SteveSlim => "steve-slim",
            Self::SunnySlim => "sunny-slim",
            Self::ZuriSlim => "zuri-slim",
            Self::AlexBold => "alex-bold",
            Self::AriBold => "ari-bold",
            Self::EfeBold => "efe-bold",
            Self::KaiBold => "kai-bold",
            Self::MakenaBold => "makena-bold",
            Self::NoorBold => "noor-bold",
            Self::SteveBold => "steve-bold",
            Self::SunnyBold => "sunny-bold",
            Self::ZuriBold => "zuri-bold",
        }
    }

    fn label_key(&self) -> &'static str {
        match self {
            Self::SteveDefault => "home.accounts.offline.skin.steve_default",
            Self::Random => "home.accounts.offline.skin.random",
            Self::AlexSlim => "home.accounts.offline.skin.alex_slim",
            Self::AriSlim => "home.accounts.offline.skin.ari_slim",
            Self::EfeSlim => "home.accounts.offline.skin.efe_slim",
            Self::KaiSlim => "home.accounts.offline.skin.kai_slim",
            Self::MakenaSlim => "home.accounts.offline.skin.makena_slim",
            Self::NoorSlim => "home.accounts.offline.skin.noor_slim",
            Self::SteveSlim => "home.accounts.offline.skin.steve_slim",
            Self::SunnySlim => "home.accounts.offline.skin.sunny_slim",
            Self::ZuriSlim => "home.accounts.offline.skin.zuri_slim",
            Self::AlexBold => "home.accounts.offline.skin.alex_bold",
            Self::AriBold => "home.accounts.offline.skin.ari_bold",
            Self::EfeBold => "home.accounts.offline.skin.efe_bold",
            Self::KaiBold => "home.accounts.offline.skin.kai_bold",
            Self::MakenaBold => "home.accounts.offline.skin.makena_bold",
            Self::NoorBold => "home.accounts.offline.skin.noor_bold",
            Self::SteveBold => "home.accounts.offline.skin.steve_bold",
            Self::SunnyBold => "home.accounts.offline.skin.sunny_bold",
            Self::ZuriBold => "home.accounts.offline.skin.zuri_bold",
        }
    }
}

const OFFLINE_SKIN_OPTIONS: [OfflineSkinPreset; 20] = [
    OfflineSkinPreset::SteveDefault,
    OfflineSkinPreset::Random,
    OfflineSkinPreset::AlexSlim,
    OfflineSkinPreset::AriSlim,
    OfflineSkinPreset::EfeSlim,
    OfflineSkinPreset::KaiSlim,
    OfflineSkinPreset::MakenaSlim,
    OfflineSkinPreset::NoorSlim,
    OfflineSkinPreset::SteveSlim,
    OfflineSkinPreset::SunnySlim,
    OfflineSkinPreset::ZuriSlim,
    OfflineSkinPreset::AlexBold,
    OfflineSkinPreset::AriBold,
    OfflineSkinPreset::EfeBold,
    OfflineSkinPreset::KaiBold,
    OfflineSkinPreset::MakenaBold,
    OfflineSkinPreset::NoorBold,
    OfflineSkinPreset::SteveBold,
    OfflineSkinPreset::SunnyBold,
    OfflineSkinPreset::ZuriBold,
];

fn account_type_label_key(login_type: &LoginType) -> &'static str {
    match login_type {
        LoginType::Offline => "home.accounts.type.offline",
        LoginType::Microsoft => "home.accounts.type.microsoft",
        LoginType::ThirdParty => "home.accounts.type.third_party",
    }
}

fn account_type_detail_key(login_type: &LoginType) -> &'static str {
    match login_type {
        LoginType::Offline => "home.accounts.source.offline_steve",
        LoginType::Microsoft => "home.accounts.source.microsoft_profile",
        LoginType::ThirdParty => "home.accounts.source.third_party_profile",
    }
}

fn account_status_key(
    services: &std::sync::Arc<AppServices>,
    store: &AppStore,
    account: &Account,
) -> &'static str {
    if account.selected {
        return "home.accounts.status.current";
    }

    match account.login_type {
        LoginType::Offline => "home.accounts.status.ready",
        LoginType::Microsoft => {
            if microsoft_client_id(store).is_ok() {
                if account.last_validated_at.is_some() {
                    "home.accounts.status.ready"
                } else {
                    "home.accounts.status.needs_refresh"
                }
            } else {
                "home.accounts.status.incomplete"
            }
        }
        LoginType::ThirdParty => {
            if services.paths.authlib_injector_jar().exists() {
                "home.accounts.status.ready"
            } else {
                "home.accounts.status.incomplete"
            }
        }
    }
}

fn account_meta_value_key(
    services: &std::sync::Arc<AppServices>,
    store: &AppStore,
    account: &Account,
) -> Option<&'static str> {
    match account.login_type {
        LoginType::Offline => account
            .skin_reference
            .as_deref()
            .map(offline_skin_reference_label_key)
            .or(Some("home.accounts.offline.skin.random")),
        LoginType::Microsoft => {
            if microsoft_client_id(store).is_ok() {
                account
                    .last_validated_at
                    .map(|_| "home.accounts.microsoft.meta_validated")
            } else {
                Some("home.accounts.microsoft.client_id_missing")
            }
        }
        LoginType::ThirdParty => {
            if services.paths.authlib_injector_jar().exists() {
                Some("home.accounts.third_party.runtime_ready")
            } else {
                Some("home.accounts.third_party.runtime_missing")
            }
        }
    }
}

fn account_avatar_text(username: &str) -> String {
    username
        .chars()
        .find(|c| !c.is_whitespace())
        .map(|c| c.to_uppercase().collect::<String>())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Z".to_string())
}

fn offline_skin_reference_label_key(reference: &str) -> &'static str {
    match reference {
        "steve-default" => "home.accounts.offline.skin.steve_default",
        "random" => "home.accounts.offline.skin.random",
        "alex-slim" => "home.accounts.offline.skin.alex_slim",
        "ari-slim" => "home.accounts.offline.skin.ari_slim",
        "efe-slim" => "home.accounts.offline.skin.efe_slim",
        "kai-slim" => "home.accounts.offline.skin.kai_slim",
        "makena-slim" => "home.accounts.offline.skin.makena_slim",
        "noor-slim" => "home.accounts.offline.skin.noor_slim",
        "steve-slim" => "home.accounts.offline.skin.steve_slim",
        "sunny-slim" => "home.accounts.offline.skin.sunny_slim",
        "zuri-slim" => "home.accounts.offline.skin.zuri_slim",
        "alex-bold" => "home.accounts.offline.skin.alex_bold",
        "ari-bold" => "home.accounts.offline.skin.ari_bold",
        "efe-bold" => "home.accounts.offline.skin.efe_bold",
        "kai-bold" => "home.accounts.offline.skin.kai_bold",
        "makena-bold" => "home.accounts.offline.skin.makena_bold",
        "noor-bold" => "home.accounts.offline.skin.noor_bold",
        "steve-bold" => "home.accounts.offline.skin.steve_bold",
        "sunny-bold" => "home.accounts.offline.skin.sunny_bold",
        "zuri-bold" => "home.accounts.offline.skin.zuri_bold",
        _ => "home.accounts.offline.skin.random",
    }
}

fn offline_preview_avatar_text(username: &str, preset: OfflineSkinPreset) -> String {
    let username_mark = username
        .trim()
        .chars()
        .find(|c| !c.is_whitespace())
        .map(|c| c.to_uppercase().collect::<String>())
        .filter(|value| !value.is_empty());
    if let Some(mark) = username_mark {
        return mark;
    }

    match preset {
        OfflineSkinPreset::Random => "R".to_string(),
        OfflineSkinPreset::SteveDefault
        | OfflineSkinPreset::SteveSlim
        | OfflineSkinPreset::SteveBold => "S".to_string(),
        OfflineSkinPreset::AlexSlim | OfflineSkinPreset::AlexBold => "A".to_string(),
        OfflineSkinPreset::AriSlim | OfflineSkinPreset::AriBold => "A".to_string(),
        OfflineSkinPreset::EfeSlim | OfflineSkinPreset::EfeBold => "E".to_string(),
        OfflineSkinPreset::KaiSlim | OfflineSkinPreset::KaiBold => "K".to_string(),
        OfflineSkinPreset::MakenaSlim | OfflineSkinPreset::MakenaBold => "M".to_string(),
        OfflineSkinPreset::NoorSlim | OfflineSkinPreset::NoorBold => "N".to_string(),
        OfflineSkinPreset::SunnySlim | OfflineSkinPreset::SunnyBold => "S".to_string(),
        OfflineSkinPreset::ZuriSlim | OfflineSkinPreset::ZuriBold => "Z".to_string(),
    }
}

fn offline_preview_avatar_class(preset: OfflineSkinPreset) -> &'static str {
    match preset {
        OfflineSkinPreset::Random => "account-preview-avatar-mark random",
        OfflineSkinPreset::SteveDefault
        | OfflineSkinPreset::AlexSlim
        | OfflineSkinPreset::AriSlim
        | OfflineSkinPreset::EfeSlim
        | OfflineSkinPreset::KaiSlim
        | OfflineSkinPreset::MakenaSlim
        | OfflineSkinPreset::NoorSlim
        | OfflineSkinPreset::SteveSlim
        | OfflineSkinPreset::SunnySlim
        | OfflineSkinPreset::ZuriSlim => "account-preview-avatar-mark slim",
        OfflineSkinPreset::AlexBold
        | OfflineSkinPreset::AriBold
        | OfflineSkinPreset::EfeBold
        | OfflineSkinPreset::KaiBold
        | OfflineSkinPreset::MakenaBold
        | OfflineSkinPreset::NoorBold
        | OfflineSkinPreset::SteveBold
        | OfflineSkinPreset::SunnyBold
        | OfflineSkinPreset::ZuriBold => "account-preview-avatar-mark bold",
    }
}

fn offline_preview_character_class(preset: OfflineSkinPreset) -> &'static str {
    match preset {
        OfflineSkinPreset::Random => "offline-character random",
        OfflineSkinPreset::SteveDefault
        | OfflineSkinPreset::AlexSlim
        | OfflineSkinPreset::AriSlim
        | OfflineSkinPreset::EfeSlim
        | OfflineSkinPreset::KaiSlim
        | OfflineSkinPreset::MakenaSlim
        | OfflineSkinPreset::NoorSlim
        | OfflineSkinPreset::SteveSlim
        | OfflineSkinPreset::SunnySlim
        | OfflineSkinPreset::ZuriSlim => "offline-character slim",
        OfflineSkinPreset::AlexBold
        | OfflineSkinPreset::AriBold
        | OfflineSkinPreset::EfeBold
        | OfflineSkinPreset::KaiBold
        | OfflineSkinPreset::MakenaBold
        | OfflineSkinPreset::NoorBold
        | OfflineSkinPreset::SteveBold
        | OfflineSkinPreset::SunnyBold
        | OfflineSkinPreset::ZuriBold => "offline-character bold",
    }
}

fn offline_skin_reference_from_preset(preset: OfflineSkinPreset) -> String {
    preset.as_str().to_string()
}

fn microsoft_redirect_uri() -> Result<Url> {
    Url::parse(&format!(
        "http://127.0.0.1:{MICROSOFT_CALLBACK_PORT}{MICROSOFT_CALLBACK_PATH}"
    ))
    .context("failed to construct Microsoft redirect URI")
}

fn microsoft_client_id(store: &AppStore) -> Result<String> {
    std::env::var("ZEPERION_MICROSOFT_CLIENT_ID")
        .or_else(|_| std::env::var("MICROSOFT_CLIENT_ID"))
        .or_else(|_| {
            let persisted = store.settings.microsoft_client_id.trim().to_string();
            if persisted.is_empty() {
                Err(std::env::VarError::NotPresent)
            } else {
                Ok(persisted)
            }
        })
        .context(
            "Microsoft login requires ZEPERION_MICROSOFT_CLIENT_ID, MICROSOFT_CLIENT_ID, or a saved client id in Settings",
        )
}

fn microsoft_auth_client(store: &AppStore) -> Result<MicrosoftAuthClient> {
    let client_id = microsoft_client_id(store)?;
    let redirect_uri = microsoft_redirect_uri()?;
    Ok(MicrosoftAuthClient::new(MicrosoftAuthConfig::new(
        client_id,
        redirect_uri,
    )))
}

fn open_in_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .context("failed to open browser with open")?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .context("failed to open browser with xdg-open")?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .context("failed to open browser with start")?;
    }

    Ok(())
}

async fn wait_for_microsoft_authorization_code(expected_state: String) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        let listener = TcpListener::bind(("127.0.0.1", MICROSOFT_CALLBACK_PORT))
            .context("failed to bind Microsoft callback listener")?;
        listener
            .set_nonblocking(false)
            .context("failed to configure Microsoft callback listener")?;

        let (mut stream, _) = listener
            .accept()
            .context("failed to receive Microsoft callback")?;
        stream
            .set_read_timeout(Some(Duration::from_secs(180)))
            .context("failed to configure Microsoft callback timeout")?;

        let mut buffer = [0_u8; 4096];
        let size = stream
            .read(&mut buffer)
            .context("failed to read Microsoft callback request")?;
        let request = String::from_utf8_lossy(&buffer[..size]);
        let first_line = request
            .lines()
            .next()
            .ok_or_else(|| anyhow!("Microsoft callback request was empty"))?;
        let path = first_line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| anyhow!("Microsoft callback request line was malformed"))?;

        let url = Url::parse(&format!("http://localhost{path}"))
            .context("failed to parse Microsoft callback URL")?;
        let mut code = None;
        let mut state = None;
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "code" => code = Some(value.into_owned()),
                "state" => state = Some(value.into_owned()),
                _ => {}
            }
        }

        let success = state.as_deref() == Some(expected_state.as_str()) && code.is_some();
        let body = if success {
            "Microsoft login completed. You can return to Zeperion Launcher."
        } else {
            "Microsoft login callback failed. You can close this tab and retry."
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();

        if state.as_deref() != Some(expected_state.as_str()) {
            return Err(anyhow!("Microsoft callback state did not match"));
        }

        code.ok_or_else(|| anyhow!("Microsoft callback did not include an authorization code"))
    })
    .await
    .context("failed to join Microsoft callback listener task")?
}

fn reuse_existing_account_identity(store: &AppStore, account: &mut Account) {
    let existing = store.accounts.iter().find(|item| {
        item.login_type == account.login_type
            && item.uuid == account.uuid
            && item.auth_server_url == account.auth_server_url
    });

    if let Some(existing) = existing {
        account.id = existing.id.clone();
        account.created_at = existing.created_at;
    }
}

fn persist_account_in_store(store: &mut AppStore, mut account: Account) -> Account {
    for existing in &mut store.accounts {
        existing.selected = false;
    }
    account.selected = true;
    store.upsert_account(account)
}

fn set_notification(store: &mut Signal<AppStore>, message: String) {
    store.write().notification = Some(message);
}

fn push_message(store: &mut Signal<AppStore>, message: String) {
    store.write().push_message(message);
}

fn start_microsoft_login(
    services: std::sync::Arc<AppServices>,
    mut store: Signal<AppStore>,
    language: String,
    flow: String,
    mut device_note: Signal<String>,
    mut status_note: Signal<String>,
    mut error_note: Signal<String>,
    mut loading: Signal<bool>,
    on_close: EventHandler<()>,
) {
    loading.set(true);
    error_note.set(String::new());
    status_note.set(tr(
        &services,
        &language,
        "home.accounts.microsoft.stage_open",
    ));
    set_notification(
        &mut store,
        tr(&services, &language, "notice.microsoft_login_start"),
    );
    spawn(async move {
        let result: Result<Account> = async {
            let client = {
                let snapshot = store.read().clone();
                microsoft_auth_client(&snapshot)?
            };
            let flow_name = flow.clone();
            let (session, oauth_flow) = if flow_name == "device_code" {
                status_note.set(tr(
                    &services,
                    &language,
                    "home.accounts.microsoft.stage_device",
                ));
                let device_code = client.request_device_code().await?;
                let browse_url = device_code
                    .verification_uri_complete
                    .clone()
                    .unwrap_or_else(|| device_code.verification_uri.clone());
                let _ = open_in_browser(&browse_url);
                device_note.set(device_code.user_code.clone());
                let session = client.authenticate_with_device_code(&device_code).await?;
                (session, OAuthFlow::DeviceCode)
            } else {
                status_note.set(tr(
                    &services,
                    &language,
                    "home.accounts.microsoft.stage_open",
                ));
                let pkce = generate_pkce_pair();
                let state = Uuid::new_v4().simple().to_string();
                let url = client.authorization_url(Some(&pkce), Some(&state))?;
                open_in_browser(url.as_str())?;
                let code = wait_for_microsoft_authorization_code(state).await?;
                status_note.set(tr(
                    &services,
                    &language,
                    "home.accounts.microsoft.stage_exchange",
                ));
                let session =
                    authenticate_microsoft_authorization_code(&client, &code, &pkce).await?;
                (session, OAuthFlow::AuthorizationCode)
            };

            status_note.set(tr(
                &services,
                &language,
                "home.accounts.microsoft.stage_xbox",
            ));
            if !session.has_entitlement() {
                return Err(anyhow!(
                    "{}",
                    tr(
                        &services,
                        &language,
                        "notice.microsoft_login_missing_entitlement"
                    )
                ));
            }

            status_note.set(tr(
                &services,
                &language,
                "home.accounts.microsoft.stage_profile",
            ));
            let refresh_token = session
                .refresh_token()
                .ok_or_else(|| anyhow!("Microsoft login did not return a refresh token"))?
                .to_string();

            let mut account = Account::microsoft(
                session.profile.name.clone(),
                session.profile.id.clone(),
                session.minecraft_access_token().to_string(),
                refresh_token,
                oauth_flow,
            );
            account.avatar_url = Some(format!(
                "https://crafatar.com/avatars/{}?size=72&overlay",
                account.uuid
            ));
            account.selected = true;
            reuse_existing_account_identity(&store.read(), &mut account);
            Ok(account)
        }
        .await;

        match result {
            Ok(account) => {
                status_note.set(tr(&services, &language, "home.accounts.status.ready"));
                let persisted = {
                    let mut snapshot = store.write();
                    persist_account_in_store(&mut snapshot, account)
                };
                let pool = services.pool.clone();
                let _ = db::accounts::upsert(&pool, &persisted).await;
                push_message(
                    &mut store,
                    format!(
                        "{} {}",
                        tr(&services, &language, "notice.account_added"),
                        persisted.username
                    ),
                );
                set_notification(
                    &mut store,
                    tr(&services, &language, "notice.microsoft_login_success"),
                );
                loading.set(false);
                on_close.call(());
            }
            Err(error) => {
                let message = format!(
                    "{} {}",
                    tr(&services, &language, "notice.microsoft_login_failed"),
                    error
                );
                push_message(&mut store, message.clone());
                set_notification(&mut store, message);
                error_note.set(error.to_string());
                loading.set(false);
            }
        }
    });
}

async fn authenticate_microsoft_authorization_code(
    client: &MicrosoftAuthClient,
    code: &str,
    pkce: &MicrosoftPkcePair,
) -> Result<crate::infrastructure::network::microsoft_auth::MicrosoftMinecraftSession> {
    client
        .authenticate_with_authorization_code(code, Some(&pkce.verifier))
        .await
}

fn start_third_party_login(
    services: std::sync::Arc<AppServices>,
    mut store: Signal<AppStore>,
    language: String,
    server_url: String,
    username: String,
    password: String,
    mut status_note: Signal<String>,
    mut error_note: Signal<String>,
    mut loading: Signal<bool>,
    on_close: EventHandler<()>,
) {
    loading.set(true);
    error_note.set(String::new());
    status_note.set(tr(
        &services,
        &language,
        "home.accounts.third_party.stage_connect",
    ));
    set_notification(
        &mut store,
        tr(&services, &language, "notice.third_party_login_start"),
    );
    spawn(async move {
        let result: Result<Account> = async {
            let client = AuthlibClient::new();
            status_note.set(tr(
                &services,
                &language,
                "home.accounts.third_party.stage_metadata",
            ));
            let metadata = client.fetch_metadata(&server_url).await?;
            status_note.set(tr(
                &services,
                &language,
                "home.accounts.third_party.stage_auth",
            ));
            let response = client
                .authenticate(&server_url, &username, &password, None, true)
                .await?;
            status_note.set(tr(
                &services,
                &language,
                "home.accounts.third_party.stage_role",
            ));
            let profile = response
                .resolved_profile()
                .ok_or_else(|| anyhow!("third-party server did not return a usable role"))?
                .clone();

            let mut account = Account::third_party(
                profile.name.clone(),
                profile.id.clone(),
                response.access_token.clone(),
                response.client_token.clone(),
                metadata.server_root_url.to_string(),
                Some(metadata.prefetched_base64.clone()),
                Some(profile.id.clone()),
                Some(format!(
                    "https://crafatar.com/avatars/{}?size=72&overlay",
                    profile.id
                )),
            );
            account.selected = true;
            reuse_existing_account_identity(&store.read(), &mut account);
            Ok(account)
        }
        .await;

        match result {
            Ok(account) => {
                status_note.set(tr(&services, &language, "home.accounts.status.ready"));
                let persisted = {
                    let mut snapshot = store.write();
                    persist_account_in_store(&mut snapshot, account)
                };
                let pool = services.pool.clone();
                let _ = db::accounts::upsert(&pool, &persisted).await;
                push_message(
                    &mut store,
                    format!(
                        "{} {}",
                        tr(&services, &language, "notice.account_added"),
                        persisted.username
                    ),
                );
                set_notification(
                    &mut store,
                    tr(&services, &language, "notice.third_party_login_success"),
                );
                loading.set(false);
                on_close.call(());
            }
            Err(error) => {
                let message = format!(
                    "{} {}",
                    tr(&services, &language, "notice.third_party_login_failed"),
                    error
                );
                push_message(&mut store, message.clone());
                set_notification(&mut store, message);
                error_note.set(error.to_string());
                loading.set(false);
            }
        }
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BackgroundAddTab {
    Local,
    Network,
}

#[derive(Props, PartialEq, Clone)]
struct CompatibilityModalProps {
    language: String,
    summary: String,
    on_continue: EventHandler<MouseEvent>,
    on_adjust: EventHandler<MouseEvent>,
    on_close: EventHandler<MouseEvent>,
}

#[component]
fn CompatibilityModal(props: CompatibilityModalProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card",
                div { class: "modal-header",
                    h3 { {tr(&services, &props.language, "home.launch.compatibility_title")} }
                    button {
                        class: "toast-close",
                        onclick: move |evt| props.on_close.call(evt),
                        "×"
                    }
                }
                p { class: "meta-text", {tr(&services, &props.language, "validation.incompatible_java")} }
                p { class: "meta-text", {props.summary.clone()} }
                div { class: "pill-row account-tab-switcher",
                    button {
                        class: "mini-btn",
                        onclick: move |evt| props.on_adjust.call(evt),
                        {tr(&services, &props.language, "home.launch.adjust_java")}
                    }
                    button {
                        class: "mini-btn active",
                        onclick: move |evt| props.on_continue.call(evt),
                        {tr(&services, &props.language, "home.launch.continue_anyway")}
                    }
                }
            }
        }
    }
}

#[component]
fn AccountsPanel(props: PanelProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let show_add_modal = use_signal(|| false);
    let mut accounts = store.read().accounts.clone();
    accounts.sort_by_key(|account| !account.selected);
    let pending_delete_id = store.read().pending_account_delete_id.clone();
    let pending_delete_name = pending_delete_id.as_ref().and_then(|account_id| {
        store
            .read()
            .accounts
            .iter()
            .find(|account| &account.id == account_id)
            .map(|account| account.username.clone())
    });

    rsx! {
        div { class: "detail-panel",
            div { class: "toolbar-row",
                h3 { {tr(&services, &props.language, "home.accounts.title")} }
                button {
                    class: "mini-btn active",
                    onclick: {
                        let mut show_add_modal = show_add_modal;
                        move |_| show_add_modal.set(true)
                    },
                    {tr(&services, &props.language, "home.accounts.add")}
                }
            }
            if store.read().accounts.is_empty() {
                p { class: "empty-state", {tr(&services, &props.language, "home.accounts.empty")} }
            } else {
                div { class: "list-stack",
                    for account in accounts {
                        div { class: "list-item",
                            div { class: "account-mini-avatar", {account_avatar_text(&account.username)} }
                            div { class: "list-item-copy account-list-copy",
                                div { class: "account-list-title-row",
                                    strong { {account.username.clone()} }
                                    span { class: "status-pill", {tr(&services, &props.language, account_type_label_key(&account.login_type))} }
                                    span { class: "status-pill soft", {tr(&services, &props.language, account_status_key(&services, &store.read(), &account))} }
                                }
                                div { class: "account-list-meta-row",
                                    span { class: "meta-text", {tr(&services, &props.language, account_type_detail_key(&account.login_type))} }
                                    if let Some(meta_key) = account_meta_value_key(&services, &store.read(), &account) {
                                        span { class: "meta-text", {tr(&services, &props.language, meta_key)} }
                                    }
                                    if let Some(server_url) = account.auth_server_url.as_deref() {
                                        span { class: "meta-text", {server_url.to_string()} }
                                    }
                                }
                            }
                            div { class: "item-actions",
                                button {
                                    class: if account.selected { "mini-btn active" } else { "mini-btn" },
                                    onclick: {
                                        let account_id = account.id.clone();
                                        let services = services.clone();
                                        let mut store = store;
                                        move |_| {
                                            if !store.write().select_account(&account_id) {
                                                return;
                                            }
                                            let pool = services.pool.clone();
                                            let selected_id = account_id.clone();
                                            tokio::spawn(async move {
                                                let _ = db::accounts::set_selected(&pool, &selected_id).await;
                                            });
                                        }
                                    },
                                    {tr(&services, &props.language, "action.select")}
                                }
                                button {
                                    class: "mini-btn danger",
                                    onclick: {
                                        let account_id = account.id.clone();
                                        let mut store = store;
                                        move |_| store.write().request_delete_account(account_id.clone())
                                    },
                                    {tr(&services, &props.language, "home.accounts.delete")}
                                }
                            }
                        }
                    }
                }
            }
        }
        if show_add_modal() {
            AccountAddModal {
                language: props.language.clone(),
                on_close: EventHandler::new({
                    let mut show_add_modal = show_add_modal;
                    move |()| show_add_modal.set(false)
                })
            }
        }
        if let Some(account_id) = pending_delete_id {
            AccountDeleteModal {
                language: props.language.clone(),
                username: pending_delete_name.unwrap_or_default(),
                on_close: EventHandler::new({
                    let mut store = store;
                    move |_| store.write().cancel_delete_account()
                }),
                on_confirm: EventHandler::new({
                    let services = services.clone();
                    let mut store = store;
                    let language = props.language.clone();
                    move |_| {
                    let pool = services.pool.clone();
                    let deleting_id = account_id.clone();
                    let next_selected_id = {
                        let snapshot = store.read();
                        let deleting_selected = snapshot
                            .accounts
                            .iter()
                            .find(|account| account.id == deleting_id)
                            .map(|account| account.selected)
                            .unwrap_or(false);
                        if deleting_selected {
                            snapshot
                                .accounts
                                .iter()
                                .find(|account| account.id != deleting_id)
                                .map(|account| account.id.clone())
                        } else {
                            None
                        }
                    };
                    store.write().remove_account(&deleting_id);
                    store.write().notification =
                        Some(tr(&services, &language, "notice.account_deleted"));
                    tokio::spawn(async move {
                        let _ = db::accounts::delete(&pool, &deleting_id).await;
                        if let Some(selected_id) = next_selected_id {
                            let _ = db::accounts::set_selected(&pool, &selected_id).await;
                        }
                    });
                }
            })
        }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct AccountAddModalProps {
    language: String,
    on_close: EventHandler<()>,
}

#[component]
fn AccountAddModal(props: AccountAddModalProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let mut offline_username = use_signal(String::new);
    let mut offline_skin_preset = use_signal(|| OfflineSkinPreset::Random);
    let mut offline_error = use_signal(String::new);
    let mut microsoft_flow = use_signal(|| "authorization_code".to_string());
    let mut microsoft_device_hint = use_signal(String::new);
    let microsoft_status = use_signal(String::new);
    let microsoft_error = use_signal(String::new);
    let microsoft_loading = use_signal(|| false);
    let mut third_party_server = use_signal(String::new);
    let mut third_party_username = use_signal(String::new);
    let mut third_party_password = use_signal(String::new);
    let mut third_party_role = use_signal(|| "auto".to_string());
    let third_party_status = use_signal(String::new);
    let mut third_party_error = use_signal(String::new);
    let third_party_loading = use_signal(|| false);
    let active_tab = use_signal(|| AccountAddKind::Offline);

    let offline_skin_options = OFFLINE_SKIN_OPTIONS;
    let microsoft_ready = microsoft_client_id(&store.read()).is_ok();
    let authlib_ready = services.paths.authlib_injector_jar().exists();
    let offline_name_taken = {
        let candidate = offline_username().trim().to_string();
        !candidate.is_empty()
            && store.read().accounts.iter().any(|account| {
                account
                    .username
                    .trim()
                    .eq_ignore_ascii_case(candidate.as_str())
            })
    };

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card modal-card-wide account-add-modal",
                div { class: "account-modal-top",
                    div { class: "account-modal-topbar",
                        div { class: "pill-row account-tab-strip",
                            button {
                                class: if active_tab() == AccountAddKind::Offline { "mini-btn active" } else { "mini-btn" },
                                onclick: {
                                    let mut active_tab = active_tab;
                                    move |_| active_tab.set(AccountAddKind::Offline)
                                },
                                {tr(&services, &props.language, "home.accounts.tab_offline")}
                            }
                            button {
                                class: if active_tab() == AccountAddKind::Microsoft { "mini-btn active" } else { "mini-btn" },
                                onclick: {
                                    let mut active_tab = active_tab;
                                    move |_| active_tab.set(AccountAddKind::Microsoft)
                                },
                                {tr(&services, &props.language, "home.accounts.tab_microsoft")}
                            }
                            button {
                                class: if active_tab() == AccountAddKind::ThirdParty { "mini-btn active" } else { "mini-btn" },
                                onclick: {
                                    let mut active_tab = active_tab;
                                    move |_| active_tab.set(AccountAddKind::ThirdParty)
                                },
                                {tr(&services, &props.language, "home.accounts.tab_third_party")}
                            }
                        }
                        button {
                            class: "toast-close",
                            onclick: move |_| props.on_close.call(()),
                            "×"
                        }
                    }
                }
                div { class: "account-modal-body",
                if active_tab() == AccountAddKind::Offline {
                    div { class: "account-add-grid account-add-grid-offline",
                        div { class: "account-pane-left",
                            div { class: "account-preview-card account-preview-card-offline",
                                div { class: "account-preview-topline",
                                    span { class: "account-preview-kicker", {tr(&services, &props.language, "home.accounts.tab_offline")} }
                                    span { class: "account-preview-chip", {tr(&services, &props.language, "home.accounts.offline.preview_chip")} }
                                }
                                div { class: "account-preview-hero",
                                    div { class: "account-preview-hero-copy",
                                        strong { {if offline_username().trim().is_empty() {
                                            tr(&services, &props.language, "home.accounts.offline.preview_name_placeholder")
                                        } else {
                                            offline_username()
                                        }} }
                                        span { class: "meta-text", {tr(&services, &props.language, offline_skin_preset().label_key())} }
                                    }
                                    div { class: "account-preview-avatar account-preview-avatar-tall",
                                        div { class: "offline-character-stage",
                                            div {
                                                class: offline_preview_character_class(offline_skin_preset()),
                                                div { class: "offline-character-head",
                                                    div {
                                                        class: offline_preview_avatar_class(offline_skin_preset()),
                                                        {offline_preview_avatar_text(&offline_username(), offline_skin_preset())}
                                                    }
                                                }
                                                div { class: "offline-character-body" }
                                                div { class: "offline-character-arms",
                                                    div { class: "offline-character-arm offline-character-arm-left" }
                                                    div { class: "offline-character-arm offline-character-arm-right" }
                                                }
                                                div { class: "offline-character-legs",
                                                    div { class: "offline-character-leg offline-character-leg-left" }
                                                    div { class: "offline-character-leg offline-character-leg-right" }
                                                }
                                            }
                                            div { class: "offline-character-shadow" }
                                        }
                                    }
                                }
                                div { class: "account-preview-copy",
                                    span { class: "meta-text", {tr(&services, &props.language, "home.accounts.offline.preview_hint")} }
                                    strong { {tr(&services, &props.language, "home.accounts.offline.preview_title")} }
                                }
                                div { class: "account-preview-badges",
                                    span { class: "status-pill", {tr(&services, &props.language, offline_skin_preset().label_key())} }
                                    span { class: "status-pill", {tr(&services, &props.language, "home.accounts.offline.uuid_auto")} }
                                }
                                p { class: "field-help", {tr(&services, &props.language, "home.accounts.offline.preview_body")} }
                            }
                        }
                        div { class: "account-pane-right",
                            div { class: "account-form-scroll",
                                div { class: "account-form-card account-form-card-offline",
                                    div { class: "account-section-heading",
                                        span { class: "account-modal-kicker", {tr(&services, &props.language, "home.accounts.offline.note")} }
                                        strong { {tr(&services, &props.language, "home.accounts.add")} }
                                        p { class: "field-help", {tr(&services, &props.language, "home.accounts.offline.description")} }
                                    }
                                    div { class: "account-field",
                                        label { r#for: "offline-username", {tr(&services, &props.language, "home.accounts.offline.username_label")} }
                                        input {
                                            id: "offline-username",
                                            value: offline_username(),
                                            placeholder: tr(&services, &props.language, "home.accounts.offline.username_placeholder"),
                                            oninput: move |evt| {
                                                offline_error.set(String::new());
                                                offline_username.set(evt.value());
                                            }
                                        }
                                        if !offline_error().is_empty() {
                                            p { class: "field-error", role: "alert", {offline_error()} }
                                        } else if offline_name_taken {
                                            p { class: "field-error", role: "alert", {tr(&services, &props.language, "validation.duplicate_account_name")} }
                                        } else {
                                            p { class: "field-help", {tr(&services, &props.language, "home.accounts.offline.username_hint")} }
                                        }
                                    }
                                    div { class: "account-field",
                                        label { r#for: "offline-skin", {tr(&services, &props.language, "home.accounts.offline.skin_label")} }
                                        select {
                                            id: "offline-skin",
                                            value: offline_skin_preset().as_str(),
                                            onchange: move |evt| {
                                                let next = match evt.value().as_str() {
                                                    "random" => OfflineSkinPreset::Random,
                                                    "alex-slim" => OfflineSkinPreset::AlexSlim,
                                                    "ari-slim" => OfflineSkinPreset::AriSlim,
                                                    "efe-slim" => OfflineSkinPreset::EfeSlim,
                                                    "kai-slim" => OfflineSkinPreset::KaiSlim,
                                                    "makena-slim" => OfflineSkinPreset::MakenaSlim,
                                                    "noor-slim" => OfflineSkinPreset::NoorSlim,
                                                    "steve-slim" => OfflineSkinPreset::SteveSlim,
                                                    "sunny-slim" => OfflineSkinPreset::SunnySlim,
                                                    "zuri-slim" => OfflineSkinPreset::ZuriSlim,
                                                    "alex-bold" => OfflineSkinPreset::AlexBold,
                                                    "ari-bold" => OfflineSkinPreset::AriBold,
                                                    "efe-bold" => OfflineSkinPreset::EfeBold,
                                                    "kai-bold" => OfflineSkinPreset::KaiBold,
                                                    "makena-bold" => OfflineSkinPreset::MakenaBold,
                                                    "noor-bold" => OfflineSkinPreset::NoorBold,
                                                    "steve-bold" => OfflineSkinPreset::SteveBold,
                                                    "sunny-bold" => OfflineSkinPreset::SunnyBold,
                                                    "zuri-bold" => OfflineSkinPreset::ZuriBold,
                                                    _ => OfflineSkinPreset::Random,
                                                };
                                                offline_skin_preset.set(next);
                                            },
                                            for preset in offline_skin_options {
                                                option {
                                                    value: preset.as_str(),
                                                    {tr(&services, &props.language, preset.label_key())}
                                                }
                                            }
                                        }
                                        p { class: "field-help", {tr(&services, &props.language, "home.accounts.offline.skin_hint")} }
                                    }
                                    div { class: "account-inline-callout",
                                        span { class: "status-pill", {tr(&services, &props.language, "home.accounts.offline.uuid_auto")} }
                                        p { class: "field-help", {tr(&services, &props.language, "home.accounts.offline.uuid_auto_hint")} }
                                    }
                                }
                            }
                        }
                    }
                } else if active_tab() == AccountAddKind::Microsoft {
                    div { class: "account-add-grid account-add-grid-single",
                        div { class: "account-pane-left",
                            div { class: "account-preview-card",
                                div { class: "account-preview-topline",
                                    span { class: "account-preview-kicker", {tr(&services, &props.language, "home.accounts.tab_microsoft")} }
                                    span { class: "account-preview-chip", {tr(&services, &props.language, "home.accounts.microsoft.flow_code")} }
                                }
                                div { class: "account-preview-copy",
                                    strong { {tr(&services, &props.language, "home.accounts.microsoft.flow_title")} }
                                    span { class: "meta-text", {tr(&services, &props.language, "home.accounts.microsoft.flow_summary")} }
                                }
                                div { class: "account-preview-badges",
                                    span { class: "status-pill", {tr(&services, &props.language, "home.accounts.microsoft.flow_code")} }
                                    span { class: "status-pill", {tr(&services, &props.language, "home.accounts.microsoft.flow_device")} }
                                }
                                p { class: "field-help", {tr(&services, &props.language, "home.accounts.microsoft.preview_hint")} }
                            }
                        }
                        div { class: "account-pane-right",
                            div { class: "account-form-scroll",
                                div { class: "account-form-card",
                                    div { class: "account-section-heading",
                                        span { class: "account-modal-kicker", {tr(&services, &props.language, "home.accounts.microsoft.note")} }
                                        strong { {tr(&services, &props.language, "home.accounts.add")} }
                                        p { class: "field-help", {tr(&services, &props.language, "home.accounts.microsoft.description")} }
                                    }
                                    div { class: if microsoft_ready { "account-inline-callout is-ready" } else { "account-inline-callout is-warning" },
                                        span { class: "status-pill", {tr(&services, &props.language, if microsoft_ready {
                                            "home.accounts.microsoft.client_id_ready"
                                        } else {
                                            "home.accounts.microsoft.client_id_missing"
                                        })} }
                                        p { class: "field-help", {tr(&services, &props.language, if microsoft_ready {
                                            "home.accounts.microsoft.client_id_ready_hint"
                                        } else {
                                            "home.accounts.microsoft.client_id_missing_hint"
                                        })} }
                                    }
                                    div { class: "account-field",
                                        label { r#for: "microsoft-flow", {tr(&services, &props.language, "home.accounts.microsoft.flow_label")} }
                                        select {
                                            id: "microsoft-flow",
                                            value: microsoft_flow(),
                                            onchange: move |evt| microsoft_flow.set(evt.value()),
                                            option { value: "authorization_code", {tr(&services, &props.language, "home.accounts.microsoft.flow_code")} }
                                            option { value: "device_code", {tr(&services, &props.language, "home.accounts.microsoft.flow_device")} }
                                        }
                                        p { class: "field-help", {tr(&services, &props.language, "home.accounts.microsoft.flow_hint")} }
                                    }
                                    div { class: "account-field",
                                        label { r#for: "microsoft-device", {tr(&services, &props.language, "home.accounts.microsoft.device_label")} }
                                        input {
                                            id: "microsoft-device",
                                            value: microsoft_device_hint(),
                                            placeholder: tr(&services, &props.language, "home.accounts.microsoft.device_placeholder"),
                                            oninput: move |evt| microsoft_device_hint.set(evt.value())
                                        }
                                        p { class: "field-help", {tr(&services, &props.language, "home.accounts.microsoft.device_hint")} }
                                    }
                                    if !microsoft_status().is_empty() || !microsoft_error().is_empty() {
                                        div { class: if microsoft_error().is_empty() { "account-inline-callout" } else { "account-inline-callout is-warning" },
                                            if !microsoft_status().is_empty() {
                                                span { class: "status-pill", {microsoft_status()} }
                                            }
                                            if !microsoft_error().is_empty() {
                                                p { class: "field-error", role: "alert", {microsoft_error()} }
                                            }
                                        }
                                    } else {
                                        div { class: "account-inline-callout",
                                            span { class: "status-pill", {tr(&services, &props.language, "home.accounts.microsoft.flow_code")} }
                                            p { class: "field-help", {tr(&services, &props.language, "home.accounts.microsoft.flow_hint")} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "account-add-grid account-add-grid-single",
                        div { class: "account-pane-left",
                            div { class: "account-preview-card",
                                div { class: "account-preview-topline",
                                    span { class: "account-preview-kicker", {tr(&services, &props.language, "home.accounts.tab_third_party")} }
                                    span { class: "account-preview-chip", {tr(&services, &props.language, "home.accounts.third_party.roles_auto")} }
                                }
                                div { class: "account-preview-copy",
                                    strong { {tr(&services, &props.language, "home.accounts.third_party.flow_title")} }
                                    span { class: "meta-text", {tr(&services, &props.language, "home.accounts.third_party.flow_summary")} }
                                }
                                div { class: "account-preview-badges",
                                    span { class: "status-pill", {tr(&services, &props.language, "home.accounts.third_party.preview_hint")} }
                                }
                                p { class: "field-help", {tr(&services, &props.language, "home.accounts.third_party.preview_body")} }
                            }
                        }
                        div { class: "account-pane-right",
                            div { class: "account-form-scroll",
                                div { class: "account-form-card",
                                    div { class: "account-section-heading",
                                        span { class: "account-modal-kicker", {tr(&services, &props.language, "home.accounts.third_party.note")} }
                                        strong { {tr(&services, &props.language, "home.accounts.add")} }
                                        p { class: "field-help", {tr(&services, &props.language, "home.accounts.third_party.description")} }
                                    }
                                    div { class: if authlib_ready { "account-inline-callout is-ready" } else { "account-inline-callout is-warning" },
                                        span { class: "status-pill", {tr(&services, &props.language, if authlib_ready {
                                            "home.accounts.third_party.runtime_ready"
                                        } else {
                                            "home.accounts.third_party.runtime_missing"
                                        })} }
                                        p { class: "field-help", {tr(&services, &props.language, if authlib_ready {
                                            "home.accounts.third_party.runtime_ready_hint"
                                        } else {
                                            "home.accounts.third_party.runtime_missing_hint"
                                        })} }
                                    }
                                    div { class: "account-field",
                                        label { r#for: "third-server", {tr(&services, &props.language, "home.accounts.third_party.server_label")} }
                                        input {
                                            id: "third-server",
                                            value: third_party_server(),
                                            placeholder: tr(&services, &props.language, "home.accounts.third_party.server_placeholder"),
                                            oninput: move |evt| {
                                                third_party_error.set(String::new());
                                                third_party_server.set(evt.value());
                                            }
                                        }
                                        p { class: "field-help", {tr(&services, &props.language, "home.accounts.third_party.server_hint")} }
                                    }
                                    div { class: "field-grid",
                                        div { class: "account-field",
                                            label { r#for: "third-user", {tr(&services, &props.language, "home.accounts.third_party.username_label")} }
                                            input {
                                                id: "third-user",
                                                value: third_party_username(),
                                                placeholder: tr(&services, &props.language, "home.accounts.third_party.username_placeholder"),
                                                oninput: move |evt| {
                                                    third_party_error.set(String::new());
                                                    third_party_username.set(evt.value());
                                                }
                                            }
                                        }
                                        div { class: "account-field",
                                            label { r#for: "third-pass", {tr(&services, &props.language, "home.accounts.third_party.password_label")} }
                                            input {
                                                id: "third-pass",
                                                r#type: "password",
                                                value: third_party_password(),
                                                placeholder: tr(&services, &props.language, "home.accounts.third_party.password_placeholder"),
                                                oninput: move |evt| {
                                                    third_party_error.set(String::new());
                                                    third_party_password.set(evt.value());
                                                }
                                            }
                                        }
                                    }
                                    div { class: "account-field",
                                        label { r#for: "third-role", {tr(&services, &props.language, "home.accounts.third_party.roles_label")} }
                                        select {
                                            id: "third-role",
                                            value: third_party_role(),
                                            onchange: move |evt| third_party_role.set(evt.value()),
                                            option { value: "auto", {tr(&services, &props.language, "home.accounts.third_party.roles_auto")} }
                                            option { value: "manual", {tr(&services, &props.language, "home.accounts.third_party.roles_manual")} }
                                        }
                                        p { class: "field-help", {tr(&services, &props.language, "home.accounts.third_party.roles_hint")} }
                                    }
                                    if !third_party_status().is_empty() || !third_party_error().is_empty() {
                                        div { class: if third_party_error().is_empty() { "account-inline-callout" } else { "account-inline-callout is-warning" },
                                            if !third_party_status().is_empty() {
                                                span { class: "status-pill", {third_party_status()} }
                                            }
                                            if !third_party_error().is_empty() {
                                                p { class: "field-error", role: "alert", {third_party_error()} }
                                            }
                                        }
                                    } else {
                                        div { class: "account-inline-callout",
                                            span { class: "status-pill", {tr(&services, &props.language, "home.accounts.third_party.preview_hint")} }
                                            p { class: "field-help", {tr(&services, &props.language, "home.accounts.third_party.preview_body")} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                }
                div { class: "pill-row account-footer",
                    button {
                        class: "mini-btn",
                        onclick: move |_| props.on_close.call(()),
                        {tr(&services, &props.language, "home.accounts.close")}
                    }
                    if active_tab() == AccountAddKind::Offline {
                        button {
                            class: "mini-btn active",
                            disabled: offline_username().trim().is_empty() || offline_name_taken,
                            onclick: {
                                let services = services.clone();
                                let mut store = store;
                                let mut offline_username = offline_username;
                                let mut offline_error = offline_error;
                                let offline_skin_preset = offline_skin_preset;
                                let on_close = props.on_close.clone();
                                let language = props.language.clone();
                                move |_| {
                                    let username = offline_username().trim().to_string();
                                    let skin_reference =
                                        Some(offline_skin_reference_from_preset(offline_skin_preset()));
                                    let account = match {
                                        let mut snapshot = store.write();
                                        snapshot.add_offline_account_with_options(
                                            username,
                                            None,
                                            skin_reference,
                                        )
                                    } {
                                        Ok(account) => account,
                                        Err(AddOfflineAccountError::EmptyUsername) => {
                                            let message = tr(&services, &language, "validation.missing_account_name");
                                            offline_error.set(message.clone());
                                            set_notification(&mut store, message);
                                            return;
                                        }
                                        Err(AddOfflineAccountError::DuplicateUsername) => {
                                            let message = tr(&services, &language, "validation.duplicate_account_name");
                                            offline_error.set(message.clone());
                                            set_notification(&mut store, message);
                                            return;
                                        }
                                    };
                                    offline_username.set(String::new());
                                    offline_error.set(String::new());
                                    on_close.call(());

                                    let pool = services.pool.clone();
                                    let mut store = store;
                                    let services = services.clone();
                                    let language = language.clone();
                                    spawn_forever(async move {
                                        match db::accounts::insert(&pool, &account).await {
                                            Ok(()) => {
                                                push_message(
                                                    &mut store,
                                                    format!(
                                                        "{} {}",
                                                        tr(&services, &language, "notice.account_added"),
                                                        account.username
                                                    ),
                                                );
                                                set_notification(
                                                    &mut store,
                                                    tr(&services, &language, "notice.account_added"),
                                                );
                                            }
                                            Err(error) => {
                                                set_notification(
                                                    &mut store,
                                                    format!(
                                                        "{} {}",
                                                        tr(&services, &language, "notice.account_add_failed"),
                                                        error
                                                    ),
                                                );
                                            }
                                        }
                                    });
                                }
                            },
                            {tr(&services, &props.language, "home.accounts.offline.primary")}
                        }
                    } else if active_tab() == AccountAddKind::Microsoft {
                        button {
                            class: "mini-btn active",
                            disabled: microsoft_loading() || !microsoft_ready,
                            onclick: {
                                let services = services.clone();
                                let store = store;
                                let language = props.language.clone();
                                let microsoft_flow = microsoft_flow;
                                let microsoft_device_hint = microsoft_device_hint;
                                let microsoft_status = microsoft_status;
                                let microsoft_error = microsoft_error;
                                let microsoft_loading = microsoft_loading;
                                let on_close = props.on_close.clone();
                                move |_| {
                                    start_microsoft_login(
                                        services.clone(),
                                        store,
                                        language.clone(),
                                        microsoft_flow(),
                                        microsoft_device_hint,
                                        microsoft_status,
                                        microsoft_error,
                                        microsoft_loading,
                                        on_close.clone(),
                                    );
                                }
                            },
                            {if microsoft_loading() {
                                tr(&services, &props.language, "home.accounts.action_loading")
                            } else {
                                tr(&services, &props.language, "home.accounts.microsoft.primary")
                            }}
                        }
                    } else {
                        button {
                            class: "mini-btn active",
                            disabled: third_party_loading(),
                            onclick: {
                                let services = services.clone();
                                let store = store;
                                let language = props.language.clone();
                                let third_party_server = third_party_server;
                                let third_party_username = third_party_username;
                                let third_party_password = third_party_password;
                                let third_party_status = third_party_status;
                                let mut third_party_error = third_party_error;
                                let third_party_loading = third_party_loading;
                                let on_close = props.on_close.clone();
                                move |_| {
                                    let server_url = third_party_server().trim().to_string();
                                    let username = third_party_username().trim().to_string();
                                    let password = third_party_password().trim().to_string();
                                    if server_url.is_empty() || username.is_empty() || password.is_empty() {
                                        let mut store = store;
                                        third_party_error.set(tr(&services, &language, "validation.missing_login_fields"));
                                        set_notification(
                                            &mut store,
                                            tr(&services, &language, "validation.missing_login_fields"),
                                        );
                                        return;
                                    }

                                    start_third_party_login(
                                        services.clone(),
                                        store,
                                        language.clone(),
                                        server_url,
                                        username,
                                        password,
                                        third_party_status,
                                        third_party_error,
                                        third_party_loading,
                                        on_close.clone(),
                                    );
                                }
                            },
                            {if third_party_loading() {
                                tr(&services, &props.language, "home.accounts.action_loading")
                            } else {
                                tr(&services, &props.language, "home.accounts.third_party.primary")
                            }}
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct AccountDeleteModalProps {
    language: String,
    username: String,
    on_close: EventHandler<MouseEvent>,
    on_confirm: EventHandler<MouseEvent>,
}

#[component]
fn AccountDeleteModal(props: AccountDeleteModalProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let message = if props.username.is_empty() {
        tr(&services, &props.language, "home.accounts.delete_confirm")
    } else {
        format!(
            "{}\n{}",
            props.username,
            tr(&services, &props.language, "home.accounts.delete_confirm")
        )
    };

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card",
                div { class: "modal-header",
                    h3 { {tr(&services, &props.language, "home.accounts.delete")} }
                    button {
                        class: "toast-close",
                        onclick: move |evt| props.on_close.call(evt),
                        "×"
                    }
                }
                p { class: "meta-text", {message} }
                div { class: "pill-row",
                    button {
                        class: "mini-btn danger",
                        onclick: move |evt| props.on_close.call(evt),
                        {tr(&services, &props.language, "home.accounts.cancel_delete")}
                    }
                    button {
                        class: "mini-btn danger active",
                        onclick: move |evt| props.on_confirm.call(evt),
                        {tr(&services, &props.language, "home.accounts.delete")}
                    }
                }
            }
        }
    }
}

#[component]
fn VersionsPanel(props: PanelProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let pending_remove_version = use_signal(|| None::<InstalledGameVersion>);
    let downloaded_versions: Vec<InstalledGameVersion> = store
        .read()
        .versions
        .iter()
        .filter(|version| version.is_downloaded)
        .cloned()
        .collect();

    rsx! {
        div { class: "detail-panel",
            h3 { {tr(&services, &props.language, "home.versions.title")} }
            if downloaded_versions.is_empty() {
                p { class: "empty-state", {tr(&services, &props.language, "home.versions.empty")} }
            } else {
                div { class: "list-stack",
                    for version in downloaded_versions {
                        div { class: "list-item",
                            div { class: "list-item-copy",
                                strong { {version.version_name.clone()} }
                                span { class: "meta-text", {format!("{} · {}", version.version_type, version.integrity_status)} }
                            }
                            div { class: "item-actions",
                                button {
                                    class: if store.read().selected_version_id.as_deref() == Some(version.id.as_str()) { "mini-btn active" } else { "mini-btn" },
                                    onclick: {
                                        let version_id = version.id.clone();
                                        let mut store = store;
                                        move |_| {
                                            let _ = store.write().set_selected_version_id(version_id.clone());
                                        }
                                    },
                                    {tr(&services, &props.language, "action.select")}
                                }
                                button {
                                    class: "mini-btn danger",
                                    onclick: {
                                        let services = services.clone();
                                        let store = store;
                                        let version_value = version.clone();
                                        let language = props.language.clone();
                                        move |_| {
                                            let services = services.clone();
                                            let version_value = version_value.clone();
                                            let mut store = store;
                                            let language = language.clone();
                                            spawn(async move {
                                                match integrity::check_version_integrity(&version_value).await {
                                                    Ok(status) => {
                                                        let updated = InstalledGameVersion {
                                                            integrity_status: status.clone(),
                                                            is_downloaded: status == "complete",
                                                            ..version_value
                                                        };
                                                        let _ = db::versions::upsert(&services.pool, &updated).await;
                                                        store.write().upsert_installed_version(updated);
                                                        store.write().notification = Some(tr(&services, &language, "notice.integrity_ok"));
                                                    }
                                                    Err(_) => {
                                                        let updated = InstalledGameVersion {
                                                            integrity_status: "corrupted".to_string(),
                                                            is_downloaded: false,
                                                            ..version_value
                                                        };
                                                        let _ = db::versions::upsert(&services.pool, &updated).await;
                                                        store.write().upsert_installed_version(updated);
                                                        store.write().notification = Some(tr(&services, &language, "notice.integrity_failed"));
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    {tr(&services, &props.language, "home.versions.check_integrity")}
                                }
                                button {
                                    class: "mini-btn",
                                    onclick: {
                                        let version_value = version.clone();
                                        let mut pending_remove_version = pending_remove_version;
                                        move |_| pending_remove_version.set(Some(version_value.clone()))
                                    },
                                    {tr(&services, &props.language, "home.versions.remove")}
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(version_value) = pending_remove_version() {
            ConfirmModal {
                language: props.language.clone(),
                title: tr(&services, &props.language, "home.versions.remove"),
                message: format!("{}\n{}", version_value.version_name, tr(&services, &props.language, "home.versions.remove_confirm")),
                confirm_label: tr(&services, &props.language, "action.delete"),
                on_close: EventHandler::new({
                    let mut pending_remove_version = pending_remove_version;
                    move |()| pending_remove_version.set(None)
                }),
                on_confirm: EventHandler::new({
                    let services = services.clone();
                    let store = store;
                    let language = props.language.clone();
                    let mut pending_remove_version = pending_remove_version;
                    move |()| {
                        let services = services.clone();
                        let mut store = store;
                        let language = language.clone();
                        let version_value = version_value.clone();
                        pending_remove_version.set(None);
                        spawn(async move {
                            let version_dir = std::path::PathBuf::from(&version_value.install_dir);
                            let _ = tokio::fs::remove_dir_all(&version_dir).await;
                            let _ = db::versions::delete_by_name(&services.pool, &version_value.version_name).await;
                            let _ = db::download_tasks::delete_by_package_id(&services.pool, &version_value.version_name).await;
                            store.write().remove_version_by_name(&version_value.version_name);
                            store.write().notification = Some(tr(&services, &language, "notice.version_removed"));
                        });
                    }
                })
            }
        }
    }
}

#[component]
fn BackgroundsPanel(props: PanelProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let show_add_modal = use_signal(|| false);
    let show_preview_modal = use_signal(|| false);
    let preview_background = use_signal(|| None::<BackgroundImage>);
    let pending_remove_background = use_signal(|| None::<BackgroundImage>);

    rsx! {
        div { class: "detail-panel",
            div { class: "toolbar-row",
                h3 { {tr(&services, &props.language, "home.backgrounds.title")} }
                button {
                    class: "mini-btn active",
                    onclick: {
                        let mut show_add_modal = show_add_modal;
                        move |_| show_add_modal.set(true)
                    },
                    {tr(&services, &props.language, "home.backgrounds.add")}
                }
            }
            if store.read().backgrounds.is_empty() {
                p { class: "empty-state", {tr(&services, &props.language, "home.backgrounds.empty")} }
            } else {
                div { class: "list-stack",
                    for item in store.read().backgrounds.clone() {
                        div { class: "list-item",
                            button {
                                class: "background-thumb-button",
                                onclick: {
                                    let mut show_preview_modal = show_preview_modal;
                                    let mut preview_background = preview_background;
                                    let item = item.clone();
                                    move |_| {
                                        preview_background.set(Some(item.clone()));
                                        show_preview_modal.set(true);
                                    }
                                },
                                img {
                                    class: "background-thumb",
                                    src: background_asset_url(&item.id),
                                    alt: item.source_value.clone()
                                }
                            }
                            div { class: "list-item-copy",
                                strong { class: "truncate-text", {background_display_name(&item)} }
                                span { class: "meta-text", {item.source_type.as_str()} }
                            }
                            div { class: "item-actions",
                                button {
                                    class: if item.is_active { "mini-btn active" } else { "mini-btn" },
                                    onclick: {
                                        let background_id = item.id.clone();
                                        let services = services.clone();
                                        let mut store = store;
                                        move |_| {
                                            if !store.write().set_active_background(&background_id) {
                                                return;
                                            }
                                            let pool = services.pool.clone();
                                            let selected_id = background_id.clone();
                                            spawn(async move {
                                                let _ = db::backgrounds::set_active(&pool, &selected_id).await;
                                            });
                                        }
                                    },
                                    {tr(&services, &props.language, "action.activate")}
                                }
                                button {
                                    class: "mini-btn danger",
                                    onclick: {
                                        let mut pending_remove_background = pending_remove_background;
                                        let item = item.clone();
                                        move |_| pending_remove_background.set(Some(item.clone()))
                                    },
                                    {tr(&services, &props.language, "home.backgrounds.remove")}
                                }
                            }
                        }
                    }
                }
            }
        }
        if show_add_modal() {
            BackgroundAddModal {
                language: props.language.clone(),
                on_close: EventHandler::new({
                    let mut show_add_modal = show_add_modal;
                    move |()| show_add_modal.set(false)
                })
            }
        }
        if let Some(item) = pending_remove_background() {
            ConfirmModal {
                language: props.language.clone(),
                title: tr(&services, &props.language, "home.backgrounds.remove"),
                message: format!("{}\n{}", background_display_name(&item), tr(&services, &props.language, "home.backgrounds.remove_confirm")),
                confirm_label: tr(&services, &props.language, "action.delete"),
                on_close: EventHandler::new({
                    let mut pending_remove_background = pending_remove_background;
                    move |()| pending_remove_background.set(None)
                }),
                on_confirm: EventHandler::new({
                    let services = services.clone();
                    let store = store;
                    let language = props.language.clone();
                    let mut pending_remove_background = pending_remove_background;
                    move |()| {
                        let pool = services.pool.clone();
                        let item = item.clone();
                        let mut store = store;
                        let language = language.clone();
                        pending_remove_background.set(None);
                        store.write().remove_background(&item.id);
                        store.write().notification = Some(tr(&services, &language, "notice.background_deleted"));
                        spawn(async move {
                            let _ = db::backgrounds::delete(&pool, &item.id).await;
                            let _ = tokio::fs::remove_file(&item.local_path).await;
                        });
                    }
                })
            }
        }
        if show_preview_modal() {
            if let Some(item) = preview_background() {
                BackgroundPreviewModal {
                    language: props.language.clone(),
                    item,
                    on_close: EventHandler::new({
                        let mut show_preview_modal = show_preview_modal;
                        let mut preview_background = preview_background;
                        move |()| {
                            preview_background.set(None);
                            show_preview_modal.set(false);
                        }
                    })
                }
            }
        }
    }
}

async fn persist_local_background(
    services: &std::sync::Arc<AppServices>,
    store: Signal<AppStore>,
    language: &str,
    source_path: &str,
) -> Result<()> {
    let stored_path =
        copy_background_from_local_path(&services.paths, source_path, services, language).await?;
    let background = build_background_record(
        BackgroundSource::LocalFile,
        source_path.to_string(),
        stored_path,
        store.read().backgrounds.is_empty(),
    );
    persist_background_record(services, store, language, background).await
}

async fn persist_remote_background(
    services: &std::sync::Arc<AppServices>,
    store: Signal<AppStore>,
    language: &str,
    url: &str,
) -> Result<()> {
    let stored_path = download_background_to_local(&services.paths, url).await?;
    let background = build_background_record(
        BackgroundSource::Url,
        url.to_string(),
        stored_path,
        store.read().backgrounds.is_empty(),
    );
    persist_background_record(services, store, language, background).await
}

fn build_background_record(
    source_type: BackgroundSource,
    source_value: String,
    local_path: String,
    is_active: bool,
) -> BackgroundImage {
    BackgroundImage {
        id: Uuid::new_v4().to_string(),
        source_type,
        source_value,
        local_path,
        is_active,
        created_at: chrono::Utc::now(),
    }
}

async fn persist_background_record(
    services: &std::sync::Arc<AppServices>,
    mut store: Signal<AppStore>,
    language: &str,
    background: BackgroundImage,
) -> Result<()> {
    store.write().add_background(background.clone());
    store.write().notification = Some(tr(services, language, "notice.background_added"));
    db::backgrounds::insert(&services.pool, &background).await?;
    if background.is_active {
        db::backgrounds::set_active(&services.pool, &background.id).await?;
    }
    Ok(())
}

async fn copy_background_from_local_path(
    paths: &crate::infrastructure::fs::AppPaths,
    source_path: &str,
    services: &std::sync::Arc<AppServices>,
    language: &str,
) -> Result<String> {
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(anyhow!(tr(
            services,
            language,
            "home.backgrounds.import_missing"
        )));
    }

    let source_bytes = tokio::fs::read(source).await?;
    let source_md5 = format!("{:x}", md5::compute(&source_bytes));
    let mut entries = tokio::fs::read_dir(paths.backgrounds_dir()).await?;
    while let Some(entry) = entries.next_entry().await? {
        let existing_path = entry.path();
        if !existing_path.is_file() {
            continue;
        }

        let existing_bytes = tokio::fs::read(&existing_path).await?;
        let existing_md5 = format!("{:x}", md5::compute(&existing_bytes));
        if existing_md5 == source_md5 {
            return Err(anyhow!(tr(
                services,
                language,
                "home.backgrounds.duplicate"
            )));
        }
    }

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png");
    for _ in 0..5 {
        let target = paths
            .backgrounds_dir()
            .join(format!("{}.{}", Uuid::new_v4(), extension));
        if tokio::fs::try_exists(&target).await? {
            continue;
        }

        tokio::fs::write(&target, &source_bytes).await?;
        return Ok(target.display().to_string());
    }

    Err(anyhow!(tr(
        services,
        language,
        "home.backgrounds.save_conflict"
    )))
}

async fn download_background_to_local(
    paths: &crate::infrastructure::fs::AppPaths,
    url: &str,
) -> Result<String> {
    let parsed = Url::parse(url)?;
    let extension = parsed
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| Path::new(name).extension())
        .and_then(|value| value.to_str())
        .unwrap_or("png");
    let target = paths
        .backgrounds_dir()
        .join(format!("{}.{}", Uuid::new_v4(), extension));
    let bytes = reqwest::get(parsed)
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    tokio::fs::write(&target, bytes).await?;
    Ok(target.display().to_string())
}

fn background_display_name(item: &BackgroundImage) -> String {
    Path::new(&item.local_path)
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| item.local_path.clone())
}

fn background_asset_url(id: &str) -> String {
    format!("/background-image/{id}")
}

#[derive(Props, PartialEq, Clone)]
struct BackgroundAddModalProps {
    language: String,
    on_close: EventHandler<()>,
}

#[derive(Props, PartialEq, Clone)]
struct BackgroundPreviewModalProps {
    language: String,
    item: BackgroundImage,
    on_close: EventHandler<()>,
}

#[component]
fn BackgroundAddModal(props: BackgroundAddModalProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let active_tab = use_signal(|| BackgroundAddTab::Local);
    let local_path = use_signal(String::new);
    let mut remote_url = use_signal(String::new);

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card",
                div { class: "modal-header",
                    h3 { {tr(&services, &props.language, "home.backgrounds.add")} }
                    button {
                        class: "toast-close",
                        onclick: move |_| props.on_close.call(()),
                        "×"
                    }
                }
                div { class: "pill-row",
                    button {
                        class: if active_tab() == BackgroundAddTab::Local { "mini-btn active" } else { "mini-btn" },
                        onclick: {
                            let mut active_tab = active_tab;
                            move |_| active_tab.set(BackgroundAddTab::Local)
                        },
                        {tr(&services, &props.language, "home.backgrounds.tab_local")}
                    }
                    button {
                        class: if active_tab() == BackgroundAddTab::Network { "mini-btn active" } else { "mini-btn" },
                        onclick: {
                            let mut active_tab = active_tab;
                            move |_| active_tab.set(BackgroundAddTab::Network)
                        },
                        {tr(&services, &props.language, "home.backgrounds.tab_network")}
                    }
                }
                if active_tab() == BackgroundAddTab::Local {
                    div { class: "inline-form",
                        input {
                            value: local_path(),
                            readonly: true,
                            placeholder: tr(&services, &props.language, "home.backgrounds.local_placeholder"),
                        }
                        button {
                            class: "mini-btn",
                            onclick: {
                                let mut local_path = local_path;
                                move |_| {
                                    spawn(async move {
                                        if let Some(file) = AsyncFileDialog::new()
                                            .add_filter("Image", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
                                            .pick_file()
                                            .await
                                        {
                                            local_path.set(file.path().display().to_string());
                                        }
                                    });
                                }
                            },
                            {tr(&services, &props.language, "home.backgrounds.choose_file")}
                        }
                    }
                } else {
                    div { class: "inline-form",
                        input {
                            value: remote_url(),
                            placeholder: tr(&services, &props.language, "home.backgrounds.url_placeholder"),
                            oninput: move |evt| remote_url.set(evt.value())
                        }
                    }
                }
                div { class: "pill-row",
                    button {
                        class: "mini-btn",
                        onclick: move |_| props.on_close.call(()),
                        {tr(&services, &props.language, "home.backgrounds.cancel")}
                    }
                    button {
                        class: "mini-btn active",
                        onclick: {
                            let services = services.clone();
                            let store = store;
                            let local_path = local_path;
                            let remote_url = remote_url;
                            let active_tab = active_tab;
                            let language = props.language.clone();
                            let on_close = props.on_close.clone();
                            move |_| {
                                let selected = active_tab();
                                let source_value = match selected {
                                    BackgroundAddTab::Local => local_path().trim().to_string(),
                                    BackgroundAddTab::Network => remote_url().trim().to_string(),
                                };
                                if source_value.is_empty() {
                                    return;
                                }

                                let services = services.clone();
                                let store_for_save = store;
                                let mut store_for_notice = store;
                                let language = language.clone();
                                let on_close = on_close.clone();
                                spawn(async move {
                                    let result = match selected {
                                        BackgroundAddTab::Local => {
                                            persist_local_background(&services, store_for_save, &language, &source_value).await
                                        }
                                        BackgroundAddTab::Network => {
                                            persist_remote_background(&services, store_for_save, &language, &source_value).await
                                        }
                                    };

                                    if let Err(error) = result {
                                        store_for_notice.write().notification = Some(error.to_string());
                                    } else {
                                        on_close.call(());
                                    }
                                });
                            }
                        },
                        {tr(&services, &props.language, "home.backgrounds.confirm")}
                    }
                }
            }
        }
    }
}

#[component]
fn BackgroundPreviewModal(props: BackgroundPreviewModalProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card modal-card-wide background-preview-modal",
                div { class: "modal-header",
                    h3 { {tr(&services, &props.language, "home.backgrounds.preview")} }
                    button {
                        class: "toast-close",
                        onclick: move |_| props.on_close.call(()),
                        "×"
                    }
                }
                div { class: "background-preview-frame",
                    img {
                        class: "background-preview-image",
                        src: background_asset_url(&props.item.id),
                        alt: props.item.source_value.clone()
                    }
                }
                div { class: "pill-row",
                    button {
                        class: "mini-btn",
                        onclick: move |_| props.on_close.call(()),
                        {tr(&services, &props.language, "home.backgrounds.close_preview")}
                    }
                }
            }
        }
    }
}

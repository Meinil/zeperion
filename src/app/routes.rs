use dioxus::prelude::*;
use dioxus_router::Routable;

use crate::pages::{
    download::{DownloadPage, DownloadSection},
    home::{HomePage, HomeSection},
    more::MorePage,
    settings::{SettingsPage, SettingsSection},
};

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[layout(crate::app::root::AppLayout)]
    #[route("/")]
    HomeAccounts {},
    #[route("/home/accounts")]
    HomeAccountsAlias {},
    #[route("/home/versions")]
    HomeVersions {},
    #[route("/home/backgrounds")]
    HomeBackgrounds {},

    #[route("/download/core")]
    DownloadCore {},
    #[route("/download/mods")]
    DownloadMods {},
    #[route("/download/worlds")]
    DownloadWorlds {},
    #[route("/download/resources")]
    DownloadResources {},
    #[route("/download/modpacks")]
    DownloadModpacks {},

    #[route("/settings/general")]
    SettingsGeneral {},
    #[route("/settings/java")]
    SettingsJava {},
    #[route("/settings/mirrors")]
    SettingsMirrors {},
    #[route("/settings/help")]
    SettingsHelp {},

    #[route("/more")]
    More {},
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopLevelRoute {
    Home,
    Download,
    Settings,
    More,
}

impl Route {
    pub fn top_level(&self) -> TopLevelRoute {
        match self {
            Self::HomeAccounts {}
            | Self::HomeAccountsAlias {}
            | Self::HomeVersions {}
            | Self::HomeBackgrounds {} => TopLevelRoute::Home,
            Self::DownloadCore {}
            | Self::DownloadMods {}
            | Self::DownloadWorlds {}
            | Self::DownloadResources {}
            | Self::DownloadModpacks {} => TopLevelRoute::Download,
            Self::SettingsGeneral {}
            | Self::SettingsJava {}
            | Self::SettingsMirrors {}
            | Self::SettingsHelp {} => TopLevelRoute::Settings,
            Self::More {} | Self::NotFound { .. } => TopLevelRoute::More,
        }
    }
}

#[component]
pub fn HomeAccounts() -> Element {
    rsx! { HomePage { section: HomeSection::Accounts } }
}

#[component]
pub fn HomeAccountsAlias() -> Element {
    rsx! { HomePage { section: HomeSection::Accounts } }
}

#[component]
pub fn HomeVersions() -> Element {
    rsx! { HomePage { section: HomeSection::Versions } }
}

#[component]
pub fn HomeBackgrounds() -> Element {
    rsx! { HomePage { section: HomeSection::Backgrounds } }
}

#[component]
pub fn DownloadCore() -> Element {
    rsx! { DownloadPage { section: DownloadSection::Core } }
}

#[component]
pub fn DownloadMods() -> Element {
    rsx! { DownloadPage { section: DownloadSection::Mods } }
}

#[component]
pub fn DownloadWorlds() -> Element {
    rsx! { DownloadPage { section: DownloadSection::Worlds } }
}

#[component]
pub fn DownloadResources() -> Element {
    rsx! { DownloadPage { section: DownloadSection::Resources } }
}

#[component]
pub fn DownloadModpacks() -> Element {
    rsx! { DownloadPage { section: DownloadSection::Modpacks } }
}

#[component]
pub fn SettingsGeneral() -> Element {
    rsx! { SettingsPage { section: SettingsSection::General } }
}

#[component]
pub fn SettingsJava() -> Element {
    rsx! { SettingsPage { section: SettingsSection::Java } }
}

#[component]
pub fn SettingsMirrors() -> Element {
    rsx! { SettingsPage { section: SettingsSection::Mirrors } }
}

#[component]
pub fn SettingsHelp() -> Element {
    rsx! { SettingsPage { section: SettingsSection::Help } }
}

#[component]
pub fn More() -> Element {
    rsx! { MorePage {} }
}

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    };

    rsx! {
        section { class: "content-pane single-pane",
            div { class: "detail-panel",
                h2 { "404" }
                p { class: "meta-text", "Route not found: {path}" }
            }
        }
    }
}

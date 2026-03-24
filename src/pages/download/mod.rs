use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{Result, anyhow};
use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;
use dioxus_router::{Navigator, use_navigator};
use futures_util::{StreamExt, TryStreamExt, stream};

use crate::{
    app::{bootstrap::AppServices, root::tr, routes::Route},
    components::{
        confirm_modal::ConfirmModal,
        side_menu::{SideMenu, SideMenuItem},
    },
    domain::{
        download::{
            AssetIndexFile, DownloadStatus, DownloadTaskRecord, RemoteGameVersion,
            RemoteVersionDetails,
        },
        game_version::InstalledGameVersion,
        settings::{BMCLAPI_MIRROR_URL, LEGACY_BMCLAPI_DOC_URL, MirrorSource, MirrorType},
    },
    infrastructure::{
        db, integrity,
        network::{downloader, mojang},
        task_runtime,
    },
    state::AppStore,
};

const SECTION_MANIFEST: &str = "manifest";
const SECTION_CLIENT: &str = "client";
const SECTION_LIBRARIES: &str = "libraries";
const SECTION_ASSET_INDEX: &str = "asset_index";
const SECTION_ASSET_OBJECTS: &str = "asset_objects";

#[component]
pub fn DownloadPage(section: DownloadSection) -> Element {
    let services = use_context::<Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let navigator = use_navigator();
    let show_task_detail_modal = use_signal(|| false);
    let task_detail_id = use_signal(|| None::<String>);
    let snapshot = store.read().clone();
    let language = snapshot.settings.language.clone();
    let manifest_loading = snapshot.manifest_loading;
    let manifest_error = snapshot.manifest_error.clone();
    let remote_versions = snapshot.remote_versions.clone();
    let download_tasks = snapshot.download_tasks.clone();
    let has_running_tasks = download_tasks
        .iter()
        .any(|task| task_runtime::is_running(&task.id));

    let items = vec![
        side_item(
            &services,
            &language,
            &navigator,
            section,
            DownloadSection::Core,
            Route::DownloadCore {},
            "core",
            "download.menu.core",
        ),
        side_item(
            &services,
            &language,
            &navigator,
            section,
            DownloadSection::Mods,
            Route::DownloadMods {},
            "mods",
            "download.menu.mods",
        ),
        side_item(
            &services,
            &language,
            &navigator,
            section,
            DownloadSection::Worlds,
            Route::DownloadWorlds {},
            "worlds",
            "download.menu.worlds",
        ),
        side_item(
            &services,
            &language,
            &navigator,
            section,
            DownloadSection::Resources,
            Route::DownloadResources {},
            "resources",
            "download.menu.resources",
        ),
        side_item(
            &services,
            &language,
            &navigator,
            section,
            DownloadSection::Modpacks,
            Route::DownloadModpacks {},
            "modpacks",
            "download.menu.modpacks",
        ),
    ];

    let filtered_versions: Vec<RemoteGameVersion> = remote_versions
        .iter()
        .filter(|remote| {
            remote.matches_filters(
                &snapshot.download_filters,
                snapshot.is_installed_version(&remote.id),
            )
        })
        .take(18)
        .cloned()
        .collect();

    rsx! {
        div { class: "two-pane",
            SideMenu { items }
            section { class: "content-pane",
                div { class: "detail-panel",
                    h2 { {tr(&services, &language, "download.title")} }
                    p { {tr(&services, &language, "download.description")} }
                    div { class: "toolbar-row",
                        button {
                            class: "mini-btn",
                            disabled: manifest_loading || has_running_tasks,
                            onclick: {
                                let services = services.clone();
                                let mut store = store;
                                move |_| {
                                    let has_running_tasks = store
                                        .read()
                                        .download_tasks
                                        .iter()
                                        .any(|task| task_runtime::is_running(&task.id));
                                    if has_running_tasks {
                                        store.write().notification = Some(
                                            "Please pause or finish active downloads before refreshing the version manifest."
                                                .to_string(),
                                        );
                                    } else {
                                        refresh_manifest(services.clone(), store);
                                    }
                                }
                            },
                            {tr(&services, &language, "download.refresh")}
                        }
                        div { class: "filter-row",
                            FilterChip {
                                active: store.read().download_filters.show_release,
                                label: tr(&services, &language, "download.filter.release"),
                                onclick: {
                                    let mut store = store;
                                    move |_| {
                                        let current = store.read().download_filters.show_release;
                                        store.write().download_filters.show_release = !current;
                                    }
                                }
                            }
                            FilterChip {
                                active: store.read().download_filters.show_snapshot,
                                label: tr(&services, &language, "download.filter.snapshot"),
                                onclick: {
                                    let mut store = store;
                                    move |_| {
                                        let current = store.read().download_filters.show_snapshot;
                                        store.write().download_filters.show_snapshot = !current;
                                    }
                                }
                            }
                            FilterChip {
                                active: store.read().download_filters.show_old,
                                label: tr(&services, &language, "download.filter.old"),
                                onclick: {
                                    let mut store = store;
                                    move |_| {
                                        let current = store.read().download_filters.show_old;
                                        store.write().download_filters.show_old = !current;
                                    }
                                }
                            }
                            FilterChip {
                                active: store.read().download_filters.show_downloaded_only,
                                label: tr(&services, &language, "download.filter.downloaded"),
                                onclick: {
                                    let mut store = store;
                                    move |_| {
                                        let current = store.read().download_filters.show_downloaded_only;
                                        store.write().download_filters.show_downloaded_only = !current;
                                    }
                                }
                            }
                        }
                    }
                    p { class: "inline-hint", {tr(&services, &language, "download.todo")} }
                    if manifest_loading {
                        p { class: "empty-state", {tr(&services, &language, "download.loading")} }
                    } else if let Some(error) = manifest_error {
                        p { class: "empty-state", {format!("{} {}", tr(&services, &language, "download.error_prefix"), error)} }
                    } else if remote_versions.is_empty() {
                        p { class: "empty-state", {tr(&services, &language, "download.empty")} }
                    } else if filtered_versions.is_empty() {
                        p { class: "empty-state", {tr(&services, &language, "download.remote.no_results")} }
                    } else {
                        div { class: "version-grid",
                            for remote in filtered_versions {
                                RemoteVersionCard {
                                    remote,
                                    language: language.clone(),
                                }
                            }
                        }
                    }
                }
                div { class: "detail-panel",
                    h3 { {tr(&services, &language, "download.tasks.title")} }
                    if download_tasks.is_empty() {
                        p { class: "empty-state", {tr(&services, &language, "download.tasks.empty")} }
                    } else {
                        div { class: "list-stack",
                            for task in download_tasks.iter().take(8).cloned() {
                                DownloadTaskItem {
                                    task,
                                    language: language.clone(),
                                    show_task_detail_modal,
                                    task_detail_id,
                                }
                            }
                        }
                    }
                }
            }
            if *show_task_detail_modal.read() {
                if let Some(task_id) = task_detail_id.read().clone() {
                    DownloadTaskDetailModal {
                        language: language.clone(),
                        task_id,
                        on_close: {
                            let mut show_task_detail_modal = show_task_detail_modal;
                            let mut task_detail_id = task_detail_id;
                            EventHandler::new(move |()| {
                                show_task_detail_modal.set(false);
                                task_detail_id.set(None);
                            })
                        }
                    }
                }
            }
        }
    }
}

fn refresh_manifest(_services: Arc<AppServices>, mut store: Signal<AppStore>) {
    store.write().set_manifest_loading(true);
    spawn(async move {
        let selected_mirror = selected_mirror_source(&store.read());
        match mojang::fetch_version_manifest_with_source(&selected_mirror).await {
            Ok(versions) => store.write().replace_remote_versions(versions),
            Err(error) => store.write().set_manifest_error(error.to_string()),
        }
    });
}

fn side_item(
    services: &Arc<AppServices>,
    language: &str,
    navigator: &Navigator,
    current: DownloadSection,
    target: DownloadSection,
    route: Route,
    key: &str,
    label_key: &str,
) -> SideMenuItem {
    SideMenuItem {
        key: key.to_string(),
        label: tr(services, language, label_key),
        active: current == target,
        on_click: EventHandler::new({
            let navigator = navigator.clone();
            move |_| {
                navigator.push(route.clone());
            }
        }),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DownloadSection {
    Core,
    Mods,
    Worlds,
    Resources,
    Modpacks,
}

#[derive(Props, Clone, PartialEq)]
struct FilterChipProps {
    active: bool,
    label: String,
    onclick: EventHandler<MouseEvent>,
}

#[component]
fn FilterChip(props: FilterChipProps) -> Element {
    rsx! {
        button {
            class: if props.active { "mini-btn active" } else { "mini-btn" },
            onclick: move |evt| props.onclick.call(evt),
            {props.label}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct RemoteVersionCardProps {
    remote: RemoteGameVersion,
    language: String,
}

#[component]
fn RemoteVersionCard(props: RemoteVersionCardProps) -> Element {
    let services = use_context::<Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let versions = store.read().versions.clone();
    let installed = versions
        .iter()
        .find(|version| version.version_name == props.remote.id)
        .cloned();
    let running = task_runtime::is_running(&format!("core-{}", props.remote.id));
    let release_time = format_local_time(props.remote.release_time);
    let integrity_status = installed
        .as_ref()
        .map(|version| version.integrity_status.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let primary_label = if running {
        "download.remote.downloading"
    } else if integrity_status == "complete" {
        "download.remote.core_ready"
    } else {
        "download.remote.download_core"
    };

    rsx! {
        article { class: "version-card",
            div { class: "version-card-header",
                strong { {props.remote.id.clone()} }
                span { class: "todo-pill", {props.remote.version_type.clone()} }
            }
            div { class: "meta-column",
                span { class: "summary-label", {tr(&services, &props.language, "download.remote.release_time")} }
                strong { {release_time} }
            }
            div { class: "meta-column",
                span { class: "summary-label", {tr(&services, &props.language, "download.remote.type")} }
                strong { {props.remote.version_type.clone()} }
            }
            div { class: "meta-column",
                span { class: "summary-label", {tr(&services, &props.language, "download.remote.integrity")} }
                strong { {integrity_label(&services, &props.language, &integrity_status)} }
            }
            button {
                class: if integrity_status == "complete" { "mini-btn active" } else { "mini-btn" },
                disabled: running || integrity_status == "complete",
                onclick: {
                    let services = services.clone();
                    let remote = props.remote.clone();
                    let store = store;
                    let language = props.language.clone();
                    move |_| {
                        spawn_core_download(services.clone(), store, remote.clone(), language.clone());
                    }
                },
                {tr(&services, &props.language, primary_label)}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DownloadTaskItemProps {
    task: DownloadTaskRecord,
    language: String,
    show_task_detail_modal: Signal<bool>,
    task_detail_id: Signal<Option<String>>,
}

#[component]
fn DownloadTaskItem(props: DownloadTaskItemProps) -> Element {
    let services = use_context::<Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let running = task_runtime::is_running(&props.task.id);
    let action_label = if props.task.status == DownloadStatus::Failed {
        "download.tasks.retry"
    } else {
        "download.tasks.resume"
    };
    let progress_text = format_progress_percent(&props.task);

    rsx! {
        article { class: "download-task-card",
            div { class: "download-task-main",
                div { class: "download-task-copy",
                    strong { {props.task.package_name.clone()} }
                    span { class: "meta-text", {format!("{} · {}", task_type_label(&services, &props.language, &props.task.task_type), status_label(&services, &props.language, &props.task.status))} }
                    span { class: "meta-text", {format_local_time(props.task.updated_at)} }
                }
                div { class: "download-task-stats",
                    div { class: "meta-column",
                        span { class: "summary-label", {tr(&services, &props.language, "download.tasks.progress")} }
                        strong { {progress_text} }
                    }
                }
            }
            div { class: "download-task-actions",
                button {
                    class: "mini-btn",
                    onclick: {
                        let mut show_task_detail_modal = props.show_task_detail_modal;
                        let mut task_detail_id = props.task_detail_id;
                        let task_id = props.task.id.clone();
                        move |_| {
                            task_detail_id.set(Some(task_id.clone()));
                            show_task_detail_modal.set(true);
                        }
                    },
                    {tr(&services, &props.language, "download.tasks.details")}
                }
                if running {
                    button {
                        class: "mini-btn",
                        onclick: {
                            let task = props.task.clone();
                            move |_| {
                                let _ = task_runtime::request_pause(&task.id);
                            }
                        },
                        {tr(&services, &props.language, "download.tasks.pause")}
                    }
                    button {
                        class: "mini-btn danger",
                        onclick: {
                            let services = services.clone();
                            let mut store = store;
                            let task = props.task.clone();
                            move |_| {
                                if task_runtime::request_cancel(&task.id) {
                                    let mut updated = task.clone();
                                    updated.status = DownloadStatus::Cancelled;
                                    updated.updated_at = chrono::Utc::now();
                                    store.write().add_download_task(updated.clone());
                                    let pool = services.pool.clone();
                                    tokio::spawn(async move {
                                        let _ = db::download_tasks::upsert(&pool, &updated).await;
                                    });
                                }
                            }
                        },
                        {tr(&services, &props.language, "download.tasks.cancel")}
                    }
                } else if props.task.is_resumable() && props.task.task_type != "metadata_sync" {
                    button {
                        class: "mini-btn",
                        onclick: {
                            let services = services.clone();
                            let store = store;
                            let task = props.task.clone();
                            let language = props.language.clone();
                            move |_| {
                                spawn_resume_task(services.clone(), store, task.clone(), language.clone());
                            }
                        },
                        {tr(&services, &props.language, action_label)}
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DownloadTaskDetailModalProps {
    language: String,
    task_id: String,
    on_close: EventHandler<()>,
}

#[component]
fn DownloadTaskDetailModal(props: DownloadTaskDetailModalProps) -> Element {
    let services = use_context::<Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let pending_cancel_task = use_signal(|| None::<DownloadTaskRecord>);
    let pending_remove_task = use_signal(|| None::<DownloadTaskRecord>);
    let task = store
        .read()
        .download_tasks
        .iter()
        .find(|task| task.id == props.task_id)
        .cloned();

    let snapshot = task
        .as_ref()
        .map(|task| collect_download_detail_snapshot(&services, &props.language, task))
        .transpose();
    let speed_text = snapshot
        .as_ref()
        .ok()
        .and_then(|item| item.as_ref())
        .map(|item| item.speed_text.clone())
        .unwrap_or_else(|| "-".to_string());
    let current_file = snapshot
        .as_ref()
        .ok()
        .and_then(|item| item.as_ref())
        .and_then(|item| item.current_target.clone())
        .unwrap_or_else(|| "-".to_string());
    let sections = snapshot
        .as_ref()
        .ok()
        .and_then(|item| item.as_ref())
        .map(|item| item.sections.clone())
        .unwrap_or_default();
    let overall_progress_text = snapshot
        .as_ref()
        .ok()
        .and_then(|item| item.as_ref())
        .map(|item| item.overall_progress_text.clone())
        .unwrap_or_else(|| "-".to_string());
    let overall_percent = snapshot
        .as_ref()
        .ok()
        .and_then(|item| item.as_ref())
        .map(|item| item.overall_percent)
        .unwrap_or_default();
    let running = task
        .as_ref()
        .map(|task| task_runtime::is_running(&task.id))
        .unwrap_or(false);
    let resumable_task = task
        .as_ref()
        .filter(|task| task.is_resumable() && task.task_type != "metadata_sync")
        .cloned();
    let removable_task = task
        .as_ref()
        .filter(|task| task.is_completed() && task.task_type != "metadata_sync")
        .cloned();

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card modal-card-wide download-detail-modal",
                div { class: "download-detail-header-block",
                    div { class: "modal-header",
                        h3 { {tr(&services, &props.language, "download.details.title")} }
                        button {
                            class: "toast-close",
                            onclick: move |_| props.on_close.call(()),
                            "×"
                        }
                    }
                    if let Some(task) = task.clone() {
                        if snapshot.is_ok() {
                            div { class: "detail-grid download-detail-summary",
                                div { class: "summary-card",
                                    span { class: "summary-label", {tr(&services, &props.language, "download.details.version")} }
                                    strong { {task.package_name.clone()} }
                                }
                                div { class: "summary-card download-detail-progress-card",
                                    span { class: "summary-label", {tr(&services, &props.language, "download.details.progress")} }
                                    strong { {overall_progress_text} }
                                    div { class: "download-progress-track download-detail-progress-track",
                                        div {
                                            class: "download-progress-fill",
                                            style: format!("width: {:.2}%;", overall_percent)
                                        }
                                    }
                                }
                                div { class: "summary-card",
                                    span { class: "summary-label", {tr(&services, &props.language, "download.details.speed")} }
                                    strong { {speed_text} }
                                }
                                div { class: "summary-card",
                                    span { class: "summary-label", {tr(&services, &props.language, "download.details.current_file")} }
                                    strong { class: "truncate-text", title: current_file.clone(), {current_file.clone()} }
                                }
                            }
                        }
                    }
                }
                if let Some(_task) = &task {
                    if snapshot.is_ok() {
                        div { class: "download-detail-body",
                            div { class: "download-detail-section-header",
                                strong { {tr(&services, &props.language, "download.details.resources")} }
                            }
                            div { class: "download-detail-sections list-stack",
                                for section in sections {
                                    div { class: "detail-row download-detail-row",
                                        div { class: "list-item-copy",
                                            strong { {section.label} }
                                            span { class: "meta-text", {section.status} }
                                        }
                                        div { class: "meta-column download-detail-progress-meta",
                                            span { class: "summary-label", {tr(&services, &props.language, "download.details.progress")} }
                                            strong { {section.progress_text} }
                                        }
                                        div { class: "download-progress-track download-detail-row-track",
                                            div {
                                                class: "download-progress-fill",
                                                style: format!("width: {:.2}%;", section.percent)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        p { class: "empty-state", {tr(&services, &props.language, "download.details.inspect_failed")} }
                    }
                } else {
                    p { class: "empty-state", {tr(&services, &props.language, "download.details.task_missing")} }
                }
                div { class: "download-detail-footer",
                    if running {
                        button {
                            class: "mini-btn",
                            onclick: {
                                let task_id = props.task_id.clone();
                                move |_| {
                                    let _ = task_runtime::request_pause(&task_id);
                                }
                            },
                            {tr(&services, &props.language, "download.tasks.pause")}
                        }
                    } else if let Some(resume_task) = resumable_task {
                        button {
                            class: "mini-btn active",
                            onclick: {
                                let services = services.clone();
                                let store = store;
                                let language = props.language.clone();
                                move |_| {
                                    spawn_resume_task(services.clone(), store, resume_task.clone(), language.clone());
                                }
                            },
                            {tr(&services, &props.language, "download.tasks.resume")}
                        }
                    }
                    if let Some(remove_task) = removable_task {
                        button {
                            class: "mini-btn danger",
                            onclick: {
                                let mut pending_remove_task = pending_remove_task;
                                move |_| pending_remove_task.set(Some(remove_task.clone()))
                            },
                            {tr(&services, &props.language, "action.delete")}
                        }
                    } else if let Some(cancel_task) = task.clone() {
                        button {
                            class: "mini-btn danger",
                            onclick: {
                                let mut pending_cancel_task = pending_cancel_task;
                                move |_| pending_cancel_task.set(Some(cancel_task.clone()))
                            },
                            {tr(&services, &props.language, "download.tasks.cancel")}
                        }
                    }
                    button {
                        class: "mini-btn",
                        onclick: move |_| props.on_close.call(()),
                        {tr(&services, &props.language, "download.details.close")}
                    }
                }
            }
        }
        if let Some(remove_task) = pending_remove_task() {
            ConfirmModal {
                language: props.language.clone(),
                title: tr(&services, &props.language, "download.remote.remove_version"),
                message: format!(
                    "{}\n{}",
                    remove_task.package_name,
                    tr(&services, &props.language, "download.remote.remove_confirm")
                ),
                confirm_label: tr(&services, &props.language, "action.delete"),
                on_close: EventHandler::new({
                    let mut pending_remove_task = pending_remove_task;
                    move |()| pending_remove_task.set(None)
                }),
                on_confirm: EventHandler::new({
                    let services = services.clone();
                    let store = store;
                    let language = props.language.clone();
                    let mut pending_remove_task = pending_remove_task;
                    let on_close = props.on_close.clone();
                    move |()| {
                        pending_remove_task.set(None);
                        on_close.call(());
                        spawn_remove_version(
                            services.clone(),
                            store,
                            remove_task.package_name.clone(),
                            language.clone(),
                        );
                    }
                })
            }
        }
        if let Some(cancel_task) = pending_cancel_task() {
            ConfirmModal {
                language: props.language.clone(),
                title: tr(&services, &props.language, "download.tasks.cancel"),
                message: tr(&services, &props.language, "download.details.cancel_confirm"),
                confirm_label: tr(&services, &props.language, "download.tasks.cancel"),
                on_close: EventHandler::new({
                    let mut pending_cancel_task = pending_cancel_task;
                    move |()| pending_cancel_task.set(None)
                }),
                on_confirm: EventHandler::new({
                    let services = services.clone();
                    let store = store;
                    let language = props.language.clone();
                    let mut pending_cancel_task = pending_cancel_task;
                    let on_close = props.on_close.clone();
                    move |()| {
                        pending_cancel_task.set(None);
                        on_close.call(());
                        spawn_cancel_download_task(services.clone(), store, cancel_task.clone(), language.clone());
                    }
                })
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct RepairVersionModalProps {
    language: String,
    remote: RemoteGameVersion,
    on_redownload: EventHandler<MouseEvent>,
    on_remove: EventHandler<MouseEvent>,
    on_close: EventHandler<MouseEvent>,
}

#[component]
fn RepairVersionModal(props: RepairVersionModalProps) -> Element {
    let services = use_context::<Arc<AppServices>>();

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card",
                div { class: "modal-header",
                    h3 { {tr(&services, &props.language, "download.repair.title")} }
                    button {
                        class: "toast-close",
                        onclick: move |evt| props.on_close.call(evt),
                        "×"
                    }
                }
                p { class: "meta-text", {tr(&services, &props.language, "download.repair.body")} }
                div { class: "meta-column",
                    span { class: "summary-label", {tr(&services, &props.language, "download.remote.release_time")} }
                    strong { {format_local_time(props.remote.release_time)} }
                }
                div { class: "meta-column",
                    span { class: "summary-label", {tr(&services, &props.language, "download.remote.integrity")} }
                    strong { {tr(&services, &props.language, "download.remote.integrity.corrupted")} }
                }
                div { class: "pill-row",
                    button {
                        class: "mini-btn active",
                        onclick: move |evt| props.on_redownload.call(evt),
                        {tr(&services, &props.language, "download.repair.redownload")}
                    }
                    button {
                        class: "mini-btn danger",
                        onclick: move |evt| props.on_remove.call(evt),
                        {tr(&services, &props.language, "download.repair.remove")}
                    }
                }
                div { class: "meta-text",
                    {format!("{} · {}", props.remote.id.clone(), props.remote.version_type.clone())}
                }
            }
        }
    }
}

fn spawn_core_download(
    services: Arc<AppServices>,
    store: Signal<AppStore>,
    remote: RemoteGameVersion,
    language: String,
) {
    if task_runtime::is_running(&format!("core-{}", remote.id)) {
        return;
    }

    spawn(async move {
        let mut store = store;
        match run_core_download_from_remote(&services, store, remote.clone()).await {
            Ok(()) => {
                store.write().notification =
                    Some(tr(&services, &language, "notice.core_downloaded"));
            }
            Err(error) => {
                store.write().notification = Some(format!(
                    "{} {}",
                    tr(&services, &language, "notice.core_download_failed"),
                    error
                ));
            }
        }
    });
}

fn spawn_resume_task(
    services: Arc<AppServices>,
    store: Signal<AppStore>,
    task: DownloadTaskRecord,
    language: String,
) {
    if task_runtime::is_running(&task.id) {
        return;
    }

    spawn(async move {
        let mut store = store;
        let result = if task.task_type == "metadata_sync" {
            sync_metadata_for_task(&services, store, &task).await
        } else {
            run_core_download_from_task(&services, store, &task).await
        };

        match result {
            Ok(()) => {
                store.write().notification =
                    Some(tr(&services, &language, "notice.core_downloaded"));
            }
            Err(error) => {
                store.write().notification = Some(format!(
                    "{} {}",
                    tr(&services, &language, "notice.download_resume_failed"),
                    error
                ));
            }
        }
    });
}

fn spawn_remove_version(
    services: Arc<AppServices>,
    store: Signal<AppStore>,
    version_name: String,
    language: String,
) {
    spawn(async move {
        let mut store = store;
        if let Some(version) = store
            .read()
            .installed_version_by_name(&version_name)
            .cloned()
        {
            let version_dir = PathBuf::from(&version.install_dir);
            let _ = tokio::fs::remove_dir_all(&version_dir).await;
        }
        let _ = db::versions::delete_by_name(&services.pool, &version_name).await;
        let _ = db::download_tasks::delete_by_package_id(&services.pool, &version_name).await;
        store.write().remove_version_by_name(&version_name);
        store.write().notification = Some(tr(&services, &language, "notice.version_removed"));
    });
}

fn spawn_cancel_download_task(
    services: Arc<AppServices>,
    store: Signal<AppStore>,
    task: DownloadTaskRecord,
    language: String,
) {
    if task_runtime::is_running(&task.id) {
        let _ = task_runtime::request_cancel(&task.id);
    }

    spawn(async move {
        let mut store = store;
        tokio::time::sleep(std::time::Duration::from_millis(160)).await;

        let version_name = task.package_id.clone();
        let version_dir = version_dir_for_task(&task);

        let _ = tokio::fs::remove_dir_all(&version_dir).await;
        let _ = db::download_tasks::delete_by_package_id(&services.pool, &version_name).await;
        let _ = db::download_tasks::delete_by_id(&services.pool, &task.id).await;
        let _ = db::versions::delete_by_name(&services.pool, &version_name).await;

        store.write().remove_version_by_name(&version_name);
        store
            .write()
            .download_tasks
            .retain(|item| item.id != task.id && item.package_id != version_name);
        task_runtime::unregister(&task.id);
        store.write().notification = Some(tr(
            &services,
            &language,
            "notice.download_cancelled_removed",
        ));
    });
}

async fn run_core_download_from_remote(
    services: &Arc<AppServices>,
    store: Signal<AppStore>,
    remote: RemoteGameVersion,
) -> Result<()> {
    let selected_mirror = selected_mirror_source(&store.read());
    let details = mojang::fetch_version_details_with_source(&selected_mirror, &remote.url).await?;
    run_core_download(
        services,
        store,
        details,
        Some(remote.url),
        Some(remote.version_type),
        selected_mirror,
    )
    .await
}

async fn run_core_download_from_task(
    services: &Arc<AppServices>,
    store: Signal<AppStore>,
    task: &DownloadTaskRecord,
) -> Result<()> {
    let selected_mirror = selected_mirror_source(&store.read());
    let details = resolve_version_details_for_resume(task, &selected_mirror).await?;

    run_core_download(
        services,
        store,
        details,
        task.metadata().detail_url,
        None,
        selected_mirror,
    )
    .await
}

async fn sync_metadata_for_task(
    services: &Arc<AppServices>,
    store: Signal<AppStore>,
    task: &DownloadTaskRecord,
) -> Result<()> {
    let selected_mirror = selected_mirror_source(&store.read());
    let detail_url = task
        .metadata()
        .detail_url
        .ok_or_else(|| anyhow!("metadata task cannot be resumed without detail_url"))?;
    let details = mojang::fetch_version_details_with_source(&selected_mirror, &detail_url).await?;
    persist_version_metadata(services, store, &details, Some(detail_url)).await?;
    Ok(())
}

async fn resolve_version_details_for_resume(
    task: &DownloadTaskRecord,
    selected_mirror: &MirrorSource,
) -> Result<RemoteVersionDetails> {
    let version_dir = version_dir_for_task(task);
    let manifest_path = version_dir.join("manifest.json");

    if manifest_path.exists() {
        let bytes = tokio::fs::read(&manifest_path).await?;
        if let Ok(details) = serde_json::from_slice::<RemoteVersionDetails>(&bytes) {
            return Ok(details);
        }
    }

    if let Some(detail_url) = task.metadata().detail_url {
        return mojang::fetch_version_details_with_source(selected_mirror, &detail_url).await;
    }

    let remote = mojang::fetch_version_manifest_with_source(selected_mirror)
        .await?
        .into_iter()
        .find(|item| item.id == task.package_id || item.id == task.package_name)
        .ok_or_else(|| {
            anyhow!(
                "unable to find remote version details for {}",
                task.package_id
            )
        })?;

    mojang::fetch_version_details_with_source(selected_mirror, &remote.url).await
}

async fn run_core_download(
    services: &Arc<AppServices>,
    mut store: Signal<AppStore>,
    details: RemoteVersionDetails,
    detail_url: Option<String>,
    version_type_hint: Option<String>,
    selected_mirror: MirrorSource,
) -> Result<()> {
    let version_dir = services.paths.downloads_dir().join(&details.id);
    let mut installed =
        persist_version_metadata(services, store, &details, detail_url.clone()).await?;
    let task_id = format!("core-{}", details.id);
    let control = task_runtime::register(task_id.clone());
    let progress_gate = Arc::new(Mutex::new(()));

    let base_total: i64 = details.downloads.client.size
        + details
            .libraries
            .iter()
            .filter_map(|library| {
                library
                    .artifact_for_current_platform()
                    .map(|item| item.size)
            })
            .sum::<i64>()
        + details
            .asset_index
            .as_ref()
            .map(|item| item.size)
            .unwrap_or_default();
    let aggregate = AggregateProgress::new(base_total);

    let running_task = DownloadTaskRecord::core_download(
        details.id.clone(),
        details.id.clone(),
        version_dir.display().to_string(),
        Some(base_total),
        0,
        DownloadStatus::Running,
        detail_url.clone(),
    );
    persist_task(services, store, running_task).await;

    let result = async {
        let provider =
            crate::infrastructure::network::provider::provider_for_source(&selected_mirror);
        let client_job = DownloadJob {
            url: provider.rewrite_download_url(&details.downloads.client.url),
            target: version_dir.join("client.jar"),
            sha1: details.downloads.client.sha1.clone(),
            size: details.downloads.client.size,
        };
        let library_jobs: Vec<DownloadJob> = details
            .libraries
            .iter()
            .filter_map(|library| {
                let artifact = library.artifact_for_current_platform()?;
                Some(DownloadJob {
                    url: provider.rewrite_download_url(&artifact.url),
                    target: version_dir.join("libraries").join(&artifact.path),
                    sha1: artifact.sha1.clone(),
                    size: artifact.size,
                })
            })
            .collect();
        let asset_index_job = details.asset_index.as_ref().map(|asset_index| DownloadJob {
            url: provider.rewrite_download_url(&asset_index.url),
            target: version_dir
                .join("assets")
                .join("indexes")
                .join(format!("{}.json", asset_index.id)),
            sha1: asset_index.sha1.clone(),
            size: asset_index.size,
        });
        let libraries_total = library_jobs.iter().map(|job| job.size).sum::<i64>();
        let libraries_progress =
            SectionGroupProgress::new(libraries_total, library_jobs.len() as i64);

        publish_single_section(&task_id, SECTION_MANIFEST, 1, 1, "ready");
        publish_single_section(&task_id, SECTION_CLIENT, 0, client_job.size, "pending");
        publish_group_section(
            &task_id,
            SECTION_LIBRARIES,
            0,
            libraries_total,
            0,
            library_jobs.len() as i64,
            "pending",
        );
        if let Some(job) = asset_index_job.as_ref() {
            publish_single_section(&task_id, SECTION_ASSET_INDEX, 0, job.size, "pending");
        }

        let (_, _, asset_index_path) = tokio::try_join!(
            ensure_artifact(
                services,
                store,
                &task_id,
                &client_job,
                &control,
                aggregate.clone(),
                progress_gate.clone(),
                Some(SectionProgressReporter::single(
                    SECTION_CLIENT,
                    client_job.size
                )),
            ),
            download_jobs_parallel(
                services,
                store,
                &task_id,
                library_jobs,
                &control,
                aggregate.clone(),
                8,
                progress_gate.clone(),
                Some(SectionProgressReporter::group(
                    SECTION_LIBRARIES,
                    libraries_progress.clone()
                )),
            ),
            async {
                if let Some(job) = asset_index_job.clone() {
                    ensure_artifact(
                        services,
                        store,
                        &task_id,
                        &job,
                        &control,
                        aggregate.clone(),
                        progress_gate.clone(),
                        Some(SectionProgressReporter::single(
                            SECTION_ASSET_INDEX,
                            job.size,
                        )),
                    )
                    .await?;
                    Ok::<Option<PathBuf>, anyhow::Error>(Some(job.target))
                } else {
                    Ok::<Option<PathBuf>, anyhow::Error>(None)
                }
            }
        )?;

        if let Some(index_path) = asset_index_path {
            let bytes = tokio::fs::read(&index_path).await?;
            let asset_index_file: AssetIndexFile = serde_json::from_slice(&bytes)?;
            let object_total: i64 = asset_index_file
                .objects
                .values()
                .map(|item| item.size)
                .sum();
            let object_count = asset_index_file.objects.len() as i64;
            let object_progress = SectionGroupProgress::new(object_total, object_count);
            let (downloaded_bytes, total_bytes) = aggregate.add_total(object_total);
            report_task_progress(
                services,
                store,
                &task_id,
                DownloadStatus::Running,
                downloaded_bytes,
                total_bytes,
                progress_gate.clone(),
            )
            .await;
            publish_group_section(
                &task_id,
                SECTION_ASSET_OBJECTS,
                0,
                object_total,
                0,
                object_count,
                "pending",
            );

            let mut object_jobs: Vec<DownloadJob> = asset_index_file
                .objects
                .values()
                .map(|object| {
                    let prefix = &object.hash[..2];
                    DownloadJob {
                        url: provider.rewrite_download_url(&format!(
                            "https://resources.download.minecraft.net/{}/{}",
                            prefix, object.hash
                        )),
                        target: version_dir
                            .join("assets")
                            .join("objects")
                            .join(prefix)
                            .join(&object.hash),
                        sha1: object.hash.clone(),
                        size: object.size,
                    }
                })
                .collect();
            object_jobs.sort_by(|left, right| right.size.cmp(&left.size));

            download_jobs_parallel(
                services,
                store,
                &task_id,
                object_jobs,
                &control,
                aggregate.clone(),
                16,
                progress_gate.clone(),
                Some(SectionProgressReporter::group(
                    SECTION_ASSET_OBJECTS,
                    object_progress,
                )),
            )
            .await?;
        }

        let (_, total_bytes) = aggregate.snapshot();
        Ok::<i64, anyhow::Error>(total_bytes.unwrap_or(base_total))
    }
    .await;

    match result {
        Ok(total_bytes) => {
            let status = integrity::check_version_integrity(&installed).await?;
            installed.integrity_status = status.clone();
            installed.is_downloaded = status == "complete";
            if let Some(version_type) = version_type_hint {
                installed.version_type = version_type;
            }
            db::versions::upsert(&services.pool, &installed).await?;
            store.write().upsert_installed_version(installed.clone());

            let completed_task = DownloadTaskRecord::core_download(
                details.id.clone(),
                details.id.clone(),
                version_dir.display().to_string(),
                Some(total_bytes),
                total_bytes,
                DownloadStatus::Completed,
                detail_url,
            );
            persist_task(services, store, completed_task).await;
            task_runtime::unregister(&task_id);
            Ok(())
        }
        Err(error) => {
            let status = if task_runtime::is_cancel_requested(&control) {
                DownloadStatus::Cancelled
            } else if task_runtime::is_pause_requested(&control) {
                DownloadStatus::Paused
            } else {
                DownloadStatus::Failed
            };
            let (current, total_bytes) = aggregate.snapshot();
            let failed_task = DownloadTaskRecord::core_download(
                details.id.clone(),
                details.id.clone(),
                version_dir.display().to_string(),
                total_bytes,
                current,
                status,
                detail_url,
            );
            persist_task(services, store, failed_task).await;
            task_runtime::unregister(&task_id);
            Err(error)
        }
    }
}

fn selected_mirror_source(store: &AppStore) -> MirrorSource {
    store
        .custom_mirrors
        .iter()
        .find(|mirror| mirror.base_url == store.settings.selected_mirror)
        .cloned()
        .unwrap_or_else(|| MirrorSource {
            id: "selected-mirror".to_string(),
            name: "Selected Mirror".to_string(),
            base_url: crate::domain::settings::normalize_selected_mirror_url(
                &store.settings.selected_mirror,
            ),
            is_builtin: false,
            mirror_type: if matches!(
                store.settings.selected_mirror.as_str(),
                BMCLAPI_MIRROR_URL | LEGACY_BMCLAPI_DOC_URL
            ) {
                MirrorType::BmclApiCompatible
            } else {
                MirrorType::OfficialCompatible
            },
        })
}

async fn ensure_artifact(
    services: &Arc<AppServices>,
    store: Signal<AppStore>,
    task_id: &str,
    job: &DownloadJob,
    control: &Arc<std::sync::atomic::AtomicU8>,
    aggregate: AggregateProgress,
    progress_gate: Arc<Mutex<()>>,
    section_reporter: Option<SectionProgressReporter>,
) -> Result<i64> {
    task_runtime::set_current_target(
        task_id,
        job.target
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| job.target.display().to_string()),
    );

    let progress_key = job.target.display().to_string();

    if job.target.exists() {
        match downloader::verify_file_sha1(&job.target, &job.sha1).await {
            Ok(()) => {
                let (downloaded_bytes, total_bytes) =
                    aggregate.set_downloaded(progress_key, job.size);
                if let Some(section_reporter) = section_reporter.as_ref() {
                    section_reporter.update(
                        task_id,
                        &job.target.display().to_string(),
                        job.size,
                        job.size,
                    );
                }
                if task_runtime::should_emit_progress(task_id, true) {
                    report_task_progress(
                        services,
                        store,
                        task_id,
                        DownloadStatus::Running,
                        downloaded_bytes,
                        total_bytes,
                        progress_gate.clone(),
                    )
                    .await;
                }
                return Ok(job.size);
            }
            Err(_) => {
                let _ = tokio::fs::remove_file(&job.target).await;
            }
        }
    }

    let services_for_progress = services.clone();
    let store_for_progress = store;
    let task_id_for_progress = task_id.to_string();
    let progress_key_for_progress = progress_key.clone();
    let aggregate_for_progress = aggregate.clone();
    let progress_gate_for_progress = progress_gate.clone();
    let section_reporter_for_progress = section_reporter.clone();
    let download = downloader::download_to_file_atomic_with_resume(
        &job.url,
        &job.target,
        Some(control),
        move |downloaded, _| {
            let services = services_for_progress.clone();
            let store = store_for_progress;
            let task_id = task_id_for_progress.clone();
            let progress_key = progress_key_for_progress.clone();
            let aggregate = aggregate_for_progress.clone();
            let progress_gate = progress_gate_for_progress.clone();
            let section_reporter = section_reporter_for_progress.clone();
            async move {
                let (downloaded_bytes, total_bytes) =
                    aggregate.set_downloaded(progress_key, downloaded);
                task_runtime::record_progress(&task_id, downloaded_bytes);
                if let Some(section_reporter) = section_reporter.as_ref() {
                    section_reporter.update(
                        &task_id,
                        &job.target.display().to_string(),
                        downloaded,
                        job.size,
                    );
                }
                if task_runtime::should_emit_progress(&task_id, false) {
                    report_task_progress(
                        &services,
                        store,
                        &task_id,
                        DownloadStatus::Running,
                        downloaded_bytes,
                        total_bytes,
                        progress_gate,
                    )
                    .await;
                }
            }
        },
    )
    .await?;

    match download {
        Some(()) => {
            downloader::verify_file_sha1(&job.target, &job.sha1).await?;
            let (downloaded_bytes, total_bytes) = aggregate.set_downloaded(progress_key, job.size);
            if let Some(section_reporter) = section_reporter.as_ref() {
                section_reporter.update(
                    task_id,
                    &job.target.display().to_string(),
                    job.size,
                    job.size,
                );
            }
            if task_runtime::should_emit_progress(task_id, true) {
                report_task_progress(
                    services,
                    store,
                    task_id,
                    DownloadStatus::Running,
                    downloaded_bytes,
                    total_bytes,
                    progress_gate,
                )
                .await;
            }
            Ok(job.size)
        }
        None => Err(anyhow!("download interrupted")),
    }
}

async fn download_jobs_parallel(
    services: &Arc<AppServices>,
    store: Signal<AppStore>,
    task_id: &str,
    jobs: Vec<DownloadJob>,
    control: &Arc<std::sync::atomic::AtomicU8>,
    aggregate: AggregateProgress,
    concurrency: usize,
    progress_gate: Arc<Mutex<()>>,
    section_reporter: Option<SectionProgressReporter>,
) -> Result<()> {
    stream::iter(jobs.into_iter().map(|job| {
        let services = services.clone();
        let store = store;
        let task_id = task_id.to_string();
        let control = control.clone();
        let aggregate = aggregate.clone();
        let progress_gate = progress_gate.clone();
        let section_reporter = section_reporter.clone();
        async move {
            ensure_artifact(
                &services,
                store,
                &task_id,
                &job,
                &control,
                aggregate,
                progress_gate,
                section_reporter,
            )
            .await?;
            Ok::<(), anyhow::Error>(())
        }
    }))
    .buffer_unordered(concurrency.max(1))
    .try_collect::<Vec<_>>()
    .await?;

    Ok(())
}

async fn persist_version_metadata(
    services: &Arc<AppServices>,
    mut store: Signal<AppStore>,
    details: &RemoteVersionDetails,
    _detail_url: Option<String>,
) -> Result<InstalledGameVersion> {
    let version_dir = services.paths.downloads_dir().join(&details.id);
    tokio::fs::create_dir_all(&version_dir).await?;

    let manifest_path = version_dir.join("manifest.json");
    let manifest_json = serde_json::to_vec_pretty(details)?;
    tokio::fs::write(&manifest_path, manifest_json).await?;

    let installed = store
        .read()
        .installed_version_by_name(&details.id)
        .cloned()
        .map(|mut version| {
            version.version_type = details.version_type.clone();
            version.release_time = details.release_time;
            version.install_dir = version_dir.display().to_string();
            version.integrity_status = if version.integrity_status == "complete" {
                "complete".to_string()
            } else {
                "metadata_only".to_string()
            };
            version
        })
        .unwrap_or_else(|| InstalledGameVersion {
            id: uuid::Uuid::new_v4().to_string(),
            version_name: details.id.clone(),
            version_type: details.version_type.clone(),
            release_time: details.release_time,
            install_dir: version_dir.display().to_string(),
            is_downloaded: false,
            integrity_status: "metadata_only".to_string(),
        });

    db::versions::upsert(&services.pool, &installed).await?;
    store.write().upsert_installed_version(installed.clone());
    let _ = db::download_tasks::delete_non_core_by_package_id(&services.pool, &details.id).await;
    Ok(installed)
}

async fn persist_task(
    services: &Arc<AppServices>,
    mut store: Signal<AppStore>,
    task: DownloadTaskRecord,
) {
    store.write().add_download_task(task.clone());
    let _ = db::download_tasks::upsert(&services.pool, &task).await;
}

async fn report_task_progress(
    services: &Arc<AppServices>,
    mut store: Signal<AppStore>,
    task_id: &str,
    status: DownloadStatus,
    downloaded_bytes: i64,
    total_bytes: Option<i64>,
    progress_gate: Arc<Mutex<()>>,
) {
    let task_snapshot = {
        let _guard: MutexGuard<'_, ()> = progress_gate.lock().expect("progress gate poisoned");
        store.write().update_download_task_progress(
            task_id,
            status.clone(),
            downloaded_bytes,
            total_bytes,
        );
        store
            .read()
            .download_tasks
            .iter()
            .find(|task| task.id == task_id)
            .cloned()
    };
    if let Some(task) = task_snapshot {
        let _ = db::download_tasks::upsert(&services.pool, &task).await;
    }
}

fn version_dir_for_task(task: &DownloadTaskRecord) -> PathBuf {
    let target = PathBuf::from(&task.target_path);
    match task.task_type.as_str() {
        "metadata_sync" | "client_download" => target.parent().unwrap_or(&target).to_path_buf(),
        "libraries_download" => target.parent().unwrap_or(&target).to_path_buf(),
        "core_download" => target,
        "assets_sync" => target
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .unwrap_or(&target)
            .to_path_buf(),
        "asset_objects_download" => target
            .parent()
            .and_then(|path| path.parent())
            .unwrap_or(&target)
            .to_path_buf(),
        _ => target,
    }
}

fn format_progress_percent(task: &DownloadTaskRecord) -> String {
    match task.total_bytes {
        Some(total) if total > 0 => {
            format!(
                "{:.1}%",
                (task.downloaded_bytes as f64 / total as f64) * 100.0
            )
        }
        _ => "-".to_string(),
    }
}

fn format_local_time(value: DateTime<Utc>) -> String {
    value
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

fn format_size(bytes: i64) -> String {
    if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
    }
}

fn format_speed(bytes_per_second: f64) -> String {
    if bytes_per_second < 1024.0 * 1024.0 {
        format!("{:.1} KB/s", bytes_per_second / 1024.0)
    } else {
        format!("{:.2} MB/s", bytes_per_second / 1024.0 / 1024.0)
    }
}

#[derive(Clone)]
struct DownloadDetailSnapshot {
    current_target: Option<String>,
    speed_text: String,
    overall_progress_text: String,
    overall_percent: f64,
    sections: Vec<DownloadDetailSection>,
}

#[derive(Clone)]
struct DownloadDetailSection {
    label: String,
    status: String,
    progress_text: String,
    percent: f64,
}

#[derive(Clone)]
struct DownloadJob {
    url: String,
    target: PathBuf,
    sha1: String,
    size: i64,
}

#[derive(Clone)]
enum SectionProgressReporter {
    Single {
        key: &'static str,
        total_bytes: i64,
    },
    Group {
        key: &'static str,
        progress: SectionGroupProgress,
    },
}

#[derive(Clone)]
struct AggregateProgress {
    inner: Arc<Mutex<AggregateProgressState>>,
}

#[derive(Clone)]
struct SectionGroupProgress {
    inner: Arc<Mutex<SectionGroupProgressState>>,
}

#[derive(Default)]
struct AggregateProgressState {
    total_bytes: i64,
    entries: HashMap<String, i64>,
}

#[derive(Default)]
struct SectionGroupProgressState {
    total_bytes: i64,
    total_items: i64,
    entries: HashMap<String, i64>,
    completed: HashMap<String, bool>,
}

impl AggregateProgress {
    fn new(total_bytes: i64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AggregateProgressState {
                total_bytes,
                entries: HashMap::new(),
            })),
        }
    }

    fn set_downloaded(&self, key: String, bytes: i64) -> (i64, Option<i64>) {
        let mut state = self.inner.lock().expect("aggregate progress poisoned");
        state.entries.insert(key, bytes.max(0));
        let downloaded = state.entries.values().copied().sum::<i64>();
        (downloaded, Some(state.total_bytes.max(downloaded)))
    }

    fn add_total(&self, bytes: i64) -> (i64, Option<i64>) {
        let mut state = self.inner.lock().expect("aggregate progress poisoned");
        state.total_bytes += bytes.max(0);
        let downloaded = state.entries.values().copied().sum::<i64>();
        (downloaded, Some(state.total_bytes.max(downloaded)))
    }

    fn snapshot(&self) -> (i64, Option<i64>) {
        let state = self.inner.lock().expect("aggregate progress poisoned");
        let downloaded = state.entries.values().copied().sum::<i64>();
        (downloaded, Some(state.total_bytes.max(downloaded)))
    }
}

impl SectionGroupProgress {
    fn new(total_bytes: i64, total_items: i64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SectionGroupProgressState {
                total_bytes,
                total_items,
                entries: HashMap::new(),
                completed: HashMap::new(),
            })),
        }
    }

    fn set_item(
        &self,
        key: String,
        downloaded_bytes: i64,
        expected_size: i64,
    ) -> (i64, i64, i64, i64) {
        let mut state = self.inner.lock().expect("section group poisoned");
        state.entries.insert(key.clone(), downloaded_bytes.max(0));
        state
            .completed
            .insert(key, downloaded_bytes >= expected_size);
        let downloaded = state.entries.values().copied().sum::<i64>();
        let completed = state.completed.values().filter(|value| **value).count() as i64;
        (
            downloaded,
            state.total_bytes.max(downloaded),
            completed,
            state.total_items,
        )
    }
}

impl SectionProgressReporter {
    fn single(key: &'static str, total_bytes: i64) -> Self {
        Self::Single { key, total_bytes }
    }

    fn group(key: &'static str, progress: SectionGroupProgress) -> Self {
        Self::Group { key, progress }
    }

    fn update(&self, task_id: &str, entry_key: &str, downloaded_bytes: i64, expected_size: i64) {
        match self {
            Self::Single { key, total_bytes } => {
                let state = if downloaded_bytes >= *total_bytes {
                    "ready"
                } else if downloaded_bytes > 0 {
                    "downloading"
                } else {
                    "pending"
                };
                publish_single_section(
                    task_id,
                    key,
                    downloaded_bytes.min(*total_bytes),
                    *total_bytes,
                    state,
                );
            }
            Self::Group { key, progress } => {
                let (downloaded, total, completed, count) =
                    progress.set_item(entry_key.to_string(), downloaded_bytes, expected_size);
                let state = if completed >= count && count > 0 {
                    "ready"
                } else if downloaded > 0 {
                    "downloading"
                } else {
                    "pending"
                };
                publish_group_section(task_id, key, downloaded, total, completed, count, state);
            }
        }
    }
}

fn publish_single_section(
    task_id: &str,
    key: &'static str,
    downloaded_bytes: i64,
    total_bytes: i64,
    state: &str,
) {
    task_runtime::set_section_snapshot(
        task_id,
        task_runtime::SectionRuntimeSnapshot {
            key: key.to_string(),
            downloaded_bytes,
            total_bytes,
            completed_items: if downloaded_bytes >= total_bytes {
                1
            } else {
                0
            },
            total_items: 1,
            state: state.to_string(),
        },
    );
}

fn publish_group_section(
    task_id: &str,
    key: &'static str,
    downloaded_bytes: i64,
    total_bytes: i64,
    completed_items: i64,
    total_items: i64,
    state: &str,
) {
    task_runtime::set_section_snapshot(
        task_id,
        task_runtime::SectionRuntimeSnapshot {
            key: key.to_string(),
            downloaded_bytes,
            total_bytes,
            completed_items,
            total_items,
            state: state.to_string(),
        },
    );
}

fn collect_download_detail_snapshot(
    services: &Arc<AppServices>,
    language: &str,
    task: &DownloadTaskRecord,
) -> Result<DownloadDetailSnapshot> {
    let runtime = task_runtime::snapshot(&task.id).unwrap_or_default();
    if !runtime.sections.is_empty() {
        let mut sections: Vec<DownloadDetailSection> = runtime
            .sections
            .iter()
            .map(|section| runtime_section_to_detail(services, language, section))
            .collect();
        sections.sort_by_key(|section| detail_section_order(&section.label, services, language));

        return Ok(DownloadDetailSnapshot {
            current_target: runtime.current_target,
            speed_text: runtime
                .speed_bps
                .map(format_speed)
                .unwrap_or_else(|| format_speed(0.0)),
            overall_progress_text: format_progress_percent(task),
            overall_percent: task
                .progress_ratio()
                .map(|value| (value as f64 * 100.0).clamp(0.0, 100.0))
                .unwrap_or_default(),
            sections,
        });
    }

    let version_dir = version_dir_for_task(task);
    let manifest_path = version_dir.join("manifest.json");

    let mut sections = vec![inspect_single_file_section(
        tr(services, language, "download.details.section.manifest"),
        tr(services, language, "download.details.state.ready"),
        tr(services, language, "download.details.state.downloading"),
        tr(services, language, "download.details.state.pending"),
        &manifest_path,
        None,
    )];

    if manifest_path.exists() {
        let bytes = fs::read(&manifest_path)?;
        let details: RemoteVersionDetails = serde_json::from_slice(&bytes)?;

        sections.push(inspect_single_file_section(
            tr(services, language, "download.details.section.client"),
            tr(services, language, "download.details.state.ready"),
            tr(services, language, "download.details.state.downloading"),
            tr(services, language, "download.details.state.pending"),
            &version_dir.join("client.jar"),
            Some(details.downloads.client.size),
        ));

        sections.push(inspect_libraries_section(
            services,
            language,
            &version_dir,
            &details,
        ));

        if let Some(asset_index) = details.asset_index.as_ref() {
            let index_path = version_dir
                .join("assets")
                .join("indexes")
                .join(format!("{}.json", asset_index.id));
            sections.push(inspect_single_file_section(
                tr(services, language, "download.details.section.asset_index"),
                tr(services, language, "download.details.state.ready"),
                tr(services, language, "download.details.state.downloading"),
                tr(services, language, "download.details.state.pending"),
                &index_path,
                Some(asset_index.size),
            ));

            if index_path.exists() {
                let index_bytes = fs::read(&index_path)?;
                let asset_index_file: AssetIndexFile = serde_json::from_slice(&index_bytes)?;
                sections.push(inspect_asset_objects_section(
                    services,
                    language,
                    &version_dir,
                    &asset_index_file,
                ));
            }
        }
    }

    Ok(DownloadDetailSnapshot {
        current_target: runtime
            .current_target
            .or_else(|| find_partial_target(&version_dir)),
        speed_text: runtime
            .speed_bps
            .map(format_speed)
            .unwrap_or_else(|| "-".to_string()),
        overall_progress_text: format_progress_percent(task),
        overall_percent: task
            .progress_ratio()
            .map(|value| (value as f64 * 100.0).clamp(0.0, 100.0))
            .unwrap_or_default(),
        sections,
    })
}

fn detail_section_order(label: &str, services: &Arc<AppServices>, language: &str) -> usize {
    let labels = [
        tr(services, language, "download.details.section.manifest"),
        tr(services, language, "download.details.section.client"),
        tr(services, language, "download.details.section.libraries"),
        tr(services, language, "download.details.section.asset_index"),
        tr(services, language, "download.details.section.asset_objects"),
    ];

    labels
        .iter()
        .position(|item| item == label)
        .unwrap_or(labels.len())
}

fn runtime_section_to_detail(
    services: &Arc<AppServices>,
    language: &str,
    section: &task_runtime::SectionRuntimeSnapshot,
) -> DownloadDetailSection {
    let label = match section.key.as_str() {
        SECTION_MANIFEST => tr(services, language, "download.details.section.manifest"),
        SECTION_CLIENT => tr(services, language, "download.details.section.client"),
        SECTION_LIBRARIES => tr(services, language, "download.details.section.libraries"),
        SECTION_ASSET_INDEX => tr(services, language, "download.details.section.asset_index"),
        SECTION_ASSET_OBJECTS => tr(services, language, "download.details.section.asset_objects"),
        _ => section.key.clone(),
    };
    let status = match section.state.as_str() {
        "ready" => tr(services, language, "download.details.state.ready"),
        "downloading" => tr(services, language, "download.details.state.downloading"),
        _ => tr(services, language, "download.details.state.pending"),
    };
    let progress_text = if section.total_items > 1 {
        format!(
            "{} / {} · {}/{} {}",
            format_size(section.downloaded_bytes),
            format_size(section.total_bytes),
            section.completed_items,
            section.total_items,
            tr(services, language, "download.details.files_suffix")
        )
    } else {
        format!(
            "{} / {}",
            format_size(section.downloaded_bytes),
            format_size(section.total_bytes.max(1))
        )
    };
    let percent = if section.total_bytes > 0 {
        ((section.downloaded_bytes as f64 / section.total_bytes as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    DownloadDetailSection {
        label,
        status,
        progress_text,
        percent,
    }
}

fn inspect_single_file_section(
    label: String,
    ready_label: String,
    downloading_label: String,
    pending_label: String,
    path: &Path,
    expected_size: Option<i64>,
) -> DownloadDetailSection {
    let part_path = append_suffix(path, ".part");
    let downloaded = fs::metadata(path)
        .map(|meta| meta.len() as i64)
        .or_else(|_| fs::metadata(&part_path).map(|meta| meta.len() as i64))
        .unwrap_or(0);
    let total = expected_size.unwrap_or(downloaded.max(1));
    let percent = ((downloaded as f64 / total as f64) * 100.0).clamp(0.0, 100.0);
    let status = if path.exists() {
        ready_label
    } else if part_path.exists() {
        downloading_label
    } else {
        pending_label
    };

    DownloadDetailSection {
        label,
        status,
        progress_text: format!("{} / {}", format_size(downloaded), format_size(total)),
        percent,
    }
}

fn inspect_libraries_section(
    services: &Arc<AppServices>,
    language: &str,
    version_dir: &Path,
    details: &RemoteVersionDetails,
) -> DownloadDetailSection {
    let mut downloaded = 0_i64;
    let mut total = 0_i64;
    let mut done = 0_i64;
    let mut count = 0_i64;

    for artifact in details
        .libraries
        .iter()
        .filter_map(|library| library.downloads.as_ref()?.artifact.as_ref())
    {
        count += 1;
        total += artifact.size;
        let target = version_dir.join("libraries").join(&artifact.path);
        let part = append_suffix(&target, ".part");
        if target.exists() {
            downloaded += artifact.size;
            done += 1;
        } else if let Ok(meta) = fs::metadata(&part) {
            downloaded += meta.len() as i64;
        }
    }

    let percent = if total > 0 {
        ((downloaded as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    DownloadDetailSection {
        label: tr(services, language, "download.details.section.libraries"),
        status: format!(
            "{done}/{count} {}",
            tr(services, language, "download.details.files_suffix")
        ),
        progress_text: format!("{} / {}", format_size(downloaded), format_size(total)),
        percent,
    }
}

fn inspect_asset_objects_section(
    services: &Arc<AppServices>,
    language: &str,
    version_dir: &Path,
    asset_index_file: &AssetIndexFile,
) -> DownloadDetailSection {
    let mut downloaded = 0_i64;
    let mut total = 0_i64;
    let mut done = 0_i64;
    let count = asset_index_file.objects.len() as i64;

    for object in asset_index_file.objects.values() {
        total += object.size;
        let prefix = &object.hash[..2];
        let target = version_dir
            .join("assets")
            .join("objects")
            .join(prefix)
            .join(&object.hash);
        let part = append_suffix(&target, ".part");
        if target.exists() {
            downloaded += object.size;
            done += 1;
        } else if let Ok(meta) = fs::metadata(&part) {
            downloaded += meta.len() as i64;
        }
    }

    let percent = if total > 0 {
        ((downloaded as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    DownloadDetailSection {
        label: tr(services, language, "download.details.section.asset_objects"),
        status: format!(
            "{done}/{count} {}",
            tr(services, language, "download.details.files_suffix")
        ),
        progress_text: format!("{} / {}", format_size(downloaded), format_size(total)),
        percent,
    }
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut file_name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    file_name.push_str(suffix);
    path.with_file_name(file_name)
}

fn find_partial_target(version_dir: &Path) -> Option<String> {
    let mut latest: Option<(std::time::SystemTime, String)> = None;
    let mut stack = vec![version_dir.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            let Some(name) = path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
            else {
                continue;
            };

            if !name.ends_with(".part") {
                continue;
            }

            let modified = entry
                .metadata()
                .and_then(|meta| meta.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let display_name = name.trim_end_matches(".part").to_string();

            match &latest {
                Some((latest_time, _)) if &modified <= latest_time => {}
                _ => latest = Some((modified, display_name)),
            }
        }
    }

    latest.map(|(_, name)| name)
}

fn task_type_label(services: &Arc<AppServices>, language: &str, task_type: &str) -> String {
    match task_type {
        "core_download" => tr(services, language, "download.tasks.type.core"),
        "metadata_sync" => tr(services, language, "download.tasks.type.metadata"),
        "client_download" => tr(services, language, "download.details.section.client"),
        "libraries_download" => tr(services, language, "download.details.section.libraries"),
        "assets_sync" => tr(services, language, "download.details.section.asset_index"),
        "asset_objects_download" => {
            tr(services, language, "download.details.section.asset_objects")
        }
        _ => task_type.to_string(),
    }
}

fn status_label(services: &Arc<AppServices>, language: &str, status: &DownloadStatus) -> String {
    let key = match status {
        DownloadStatus::Pending => "download.tasks.status.pending",
        DownloadStatus::Running => "download.tasks.status.running",
        DownloadStatus::Paused => "download.tasks.status.paused",
        DownloadStatus::Completed => "download.tasks.status.completed",
        DownloadStatus::Failed => "download.tasks.status.failed",
        DownloadStatus::Cancelled => "download.tasks.status.cancelled",
    };
    tr(services, language, key)
}

fn integrity_label(services: &Arc<AppServices>, language: &str, status: &str) -> String {
    let key = match status {
        "metadata_only" => "download.remote.integrity.metadata_only",
        "client_only" => "download.remote.integrity.client_only",
        "libraries_only" => "download.remote.integrity.libraries_only",
        "complete" => "download.remote.integrity.complete",
        "corrupted" => "download.remote.integrity.corrupted",
        _ => "download.remote.integrity.not_ready",
    };
    tr(services, language, key)
}

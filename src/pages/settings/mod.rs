use dioxus::prelude::*;
use dioxus_router::{Navigator, use_navigator};
use rfd::AsyncFileDialog;
use std::collections::BTreeSet;
use tokio::fs;

use crate::{
    app::{bootstrap::AppServices, root::tr, routes::Route},
    components::{
        confirm_modal::ConfirmModal,
        side_menu::{SideMenu, SideMenuItem},
    },
    domain::settings::{MirrorSource, MirrorType},
    infrastructure::db,
    platform,
    state::AppStore,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum JavaAddTab {
    Path,
    Archive,
}

#[component]
pub fn SettingsPage(section: SettingsSection) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let navigator = use_navigator();
    let language = store.read().settings.language.clone();

    let items = vec![
        side_item(
            &services,
            &language,
            &navigator,
            section,
            SettingsSection::General,
            Route::SettingsGeneral {},
            "general",
            "settings.menu.general",
        ),
        side_item(
            &services,
            &language,
            &navigator,
            section,
            SettingsSection::Java,
            Route::SettingsJava {},
            "java",
            "settings.menu.java",
        ),
        side_item(
            &services,
            &language,
            &navigator,
            section,
            SettingsSection::Mirrors,
            Route::SettingsMirrors {},
            "mirrors",
            "settings.menu.mirrors",
        ),
        side_item(
            &services,
            &language,
            &navigator,
            section,
            SettingsSection::Help,
            Route::SettingsHelp {},
            "help",
            "settings.menu.help",
        ),
    ];

    rsx! {
        div { class: "two-pane",
            SideMenu { items }
            section { class: "content-pane",
                match section {
                    SettingsSection::General => rsx! { GeneralSettingsPanel { language: language.clone() } },
                    SettingsSection::Java => rsx! { JavaSettingsPanel { language: language.clone() } },
                    SettingsSection::Mirrors => rsx! { MirrorSourcesPanel { language: language.clone() } },
                    SettingsSection::Help => rsx! { HelpPanel { language: language.clone() } },
                }
            }
        }
    }
}

fn side_item(
    services: &std::sync::Arc<AppServices>,
    language: &str,
    navigator: &Navigator,
    current: SettingsSection,
    target: SettingsSection,
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
pub enum SettingsSection {
    General,
    Java,
    Mirrors,
    Help,
}

#[derive(Props, Clone, PartialEq)]
struct LanguageProps {
    language: String,
}

#[component]
fn GeneralSettingsPanel(props: LanguageProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let mut store = use_context::<Signal<AppStore>>();
    let mut font_families = platform::fonts::system_font_families();
    let current_font_family = store.read().settings.font_family.clone();
    if !font_families
        .iter()
        .any(|family| family == &current_font_family)
    {
        font_families.insert(0, current_font_family.clone());
    }

    rsx! {
        div { class: "detail-panel",
            h2 { {tr(&services, &props.language, "settings.general.title")} }
            div { class: "setting-grid",
                SettingRow {
                    label: tr(&services, &props.language, "settings.general.language"),
                    content: rsx! {
                        select {
                            value: store.read().settings.language.clone(),
                            onchange: {
                                let services = services.clone();
                                let mut store = store;
                                move |evt| {
                                let value = evt.value();
                                store.write().settings.language = value.clone();
                                let settings = store.read().settings.clone();
                                let pool = services.pool.clone();
                                spawn(async move {
                                    let _ = db::settings::save(&pool, &settings).await;
                                });
                            }},
                            option { value: "zh-CN", "简体中文" }
                            option { value: "en-US", "English" }
                        }
                    }
                }
                SettingRow {
                    label: tr(&services, &props.language, "settings.general.theme"),
                    content: rsx! {
                        input {
                            r#type: "color",
                            value: store.read().settings.theme_color.clone(),
                            onchange: {
                                let services = services.clone();
                                let mut store = store;
                                move |evt| {
                                    store.write().settings.theme_color = evt.value();
                                    let settings = store.read().settings.clone();
                                    let pool = services.pool.clone();
                                    spawn(async move {
                                        let _ = db::settings::save(&pool, &settings).await;
                                    });
                                }
                            }
                        }
                    }
                }
                SettingRow {
                    label: tr(&services, &props.language, "settings.general.font_family"),
                    content: rsx! {
                        select {
                            value: store.read().settings.font_family.clone(),
                            onchange: {
                                let services = services.clone();
                                let mut store = store;
                                move |evt| {
                                    let value = evt.value();
                                    store.write().settings.font_family = value;
                                    let settings = store.read().settings.clone();
                                    let pool = services.pool.clone();
                                    spawn(async move {
                                        let _ = db::settings::save(&pool, &settings).await;
                                    });
                                }
                            },
                            for family in font_families.clone() {
                                option { value: family.clone(), "{family}" }
                            }
                        }
                    }
                }
                SettingRow {
                    label: tr(&services, &props.language, "settings.general.font_size"),
                    content: rsx! {
                        input {
                            r#type: "number",
                            value: store.read().settings.font_size.to_string(),
                            oninput: move |evt| {
                                if let Ok(value) = evt.value().parse() {
                                    store.write().settings.font_size = value;
                                }
                            }
                        }
                    }
                }
                SettingRow {
                    label: tr(&services, &props.language, "settings.general.memory"),
                    content: rsx! {
                        input {
                            r#type: "number",
                            value: store.read().settings.game_memory_mb.to_string(),
                            oninput: move |evt| {
                                if let Ok(value) = evt.value().parse() {
                                    store.write().settings.game_memory_mb = value;
                                }
                            }
                        }
                    }
                }
                SettingRow {
                    label: tr(&services, &props.language, "settings.general.resolution"),
                    content: rsx! {
                        div { class: "split-inputs",
                            input {
                                r#type: "number",
                                value: store.read().settings.resolution_width.to_string(),
                                oninput: move |evt| {
                                    if let Ok(value) = evt.value().parse() {
                                        store.write().settings.resolution_width = value;
                                    }
                                }
                            }
                            input {
                                r#type: "number",
                                value: store.read().settings.resolution_height.to_string(),
                                oninput: move |evt| {
                                    if let Ok(value) = evt.value().parse() {
                                        store.write().settings.resolution_height = value;
                                    }
                                }
                            }
                        }
                    }
                }
                SettingRow {
                    label: tr(&services, &props.language, "settings.general.rotate_backgrounds"),
                    content: rsx! {
                        input {
                            r#type: "checkbox",
                            checked: store.read().settings.rotate_backgrounds,
                            onchange: move |_| {
                                let current = store.read().settings.rotate_backgrounds;
                                store.write().settings.rotate_backgrounds = !current;
                            }
                        }
                    }
                }
                SettingRow {
                    label: tr(&services, &props.language, "settings.general.microsoft_client_id"),
                    content: rsx! {
                        div { class: "settings-inline-stack",
                            input {
                                value: store.read().settings.microsoft_client_id.clone(),
                                placeholder: tr(&services, &props.language, "settings.general.microsoft_client_id_placeholder"),
                                oninput: move |evt| {
                                    store.write().settings.microsoft_client_id = evt.value();
                                }
                            }
                            p { class: "meta-text", {tr(&services, &props.language, "settings.general.microsoft_client_id_hint")} }
                        }
                    }
                }
                SettingRow {
                    label: tr(&services, &props.language, "settings.general.authlib_runtime"),
                    content: rsx! {
                        div { class: "settings-inline-stack",
                            input {
                                value: services.paths.authlib_injector_jar().display().to_string(),
                                readonly: true
                            }
                            p { class: "meta-text",
                                {if services.paths.authlib_injector_jar().exists() {
                                    tr(&services, &props.language, "settings.general.authlib_runtime_ready")
                                } else {
                                    tr(&services, &props.language, "settings.general.authlib_runtime_missing")
                                }}
                            }
                            button {
                                class: "mini-btn",
                                onclick: {
                                    let services = services.clone();
                                    let store = store;
                                    let language = props.language.clone();
                                    move |_| {
                                        let services = services.clone();
                                        let mut store = store;
                                        let language = language.clone();
                                        spawn(async move {
                                            let Some(file) = AsyncFileDialog::new()
                                                .add_filter("Java Agent", &["jar"])
                                                .pick_file()
                                                .await
                                            else {
                                                return;
                                            };

                                            let source_path = file.path().to_path_buf();
                                            let target_path = services.paths.authlib_injector_jar();
                                            let result = async {
                                                if let Some(parent) = target_path.parent() {
                                                    fs::create_dir_all(parent).await?;
                                                }
                                                fs::copy(&source_path, &target_path).await?;
                                                Ok::<(), anyhow::Error>(())
                                            }
                                            .await;

                                            match result {
                                                Ok(()) => {
                                                    store.write().notification = Some(tr(
                                                        &services,
                                                        &language,
                                                        "notice.authlib_runtime_saved",
                                                    ));
                                                }
                                                Err(error) => {
                                                    store.write().notification = Some(format!(
                                                        "{} {}",
                                                        tr(&services, &language, "notice.authlib_runtime_failed"),
                                                        error
                                                    ));
                                                }
                                            }
                                        });
                                    }
                                },
                                {tr(&services, &props.language, "settings.general.authlib_runtime_import")}
                            }
                        }
                    }
                }
            }
            button {
                class: "mini-btn active",
                onclick: {
                    let services = services.clone();
                    let mut store = store;
                    let language = props.language.clone();
                    move |_| {
                    let settings = store.read().settings.clone();
                    store.write().notification = Some(tr(&services, &language, "notice.settings_saved"));
                    let pool = services.pool.clone();
                    spawn(async move {
                        let _ = db::settings::save(&pool, &settings).await;
                    });
                }},
                {tr(&services, &props.language, "action.save")}
            }
        }
    }
}

#[component]
fn MirrorSourcesPanel(props: LanguageProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let mirror_name = use_signal(String::new);
    let mirror_url = use_signal(String::new);
    let mirror_type = use_signal(|| MirrorType::OfficialCompatible);
    let show_add_mirror_modal = use_signal(|| false);
    let pending_delete_mirror = use_signal(|| None::<MirrorSource>);
    let selected_mirror = store.read().settings.selected_mirror.clone();
    let mut mirrors = store.read().custom_mirrors.clone();
    mirrors.sort_by(|left, right| {
        let left_selected = left.base_url == selected_mirror;
        let right_selected = right.base_url == selected_mirror;
        right_selected
            .cmp(&left_selected)
            .then_with(|| right.is_builtin.cmp(&left.is_builtin))
            .then_with(|| left.name.cmp(&right.name))
    });

    rsx! {
        div { class: "detail-panel",
            div { class: "toolbar-row",
                h2 { {tr(&services, &props.language, "settings.mirrors.title")} }
                button {
                    class: "mini-btn active",
                    onclick: {
                        let mut show_add_mirror_modal = show_add_mirror_modal;
                        move |_| show_add_mirror_modal.set(true)
                    },
                    {tr(&services, &props.language, "settings.general.add_mirror")}
                }
            }
            SettingRow {
                label: tr(&services, &props.language, "settings.general.mirror"),
                content: rsx! {
                    select {
                        value: store.read().settings.selected_mirror.clone(),
                        onchange: {
                            let services = services.clone();
                            let mut store = store;
                            move |evt| {
                                let next_value = evt.value();
                                if store.read().settings.selected_mirror == next_value {
                                    return;
                                }
                                store.write().settings.selected_mirror = next_value;
                                let settings = store.read().settings.clone();
                                let pool = services.pool.clone();
                                spawn(async move {
                                    let _ = db::settings::save(&pool, &settings).await;
                                });
                            }
                        },
                        for mirror in mirrors.clone() {
                            option { value: mirror.base_url.clone(), "{mirror.name} - {mirror.base_url}" }
                        }
                    }
                }
            }
            if store.read().custom_mirrors.is_empty() {
                p { class: "empty-state", {tr(&services, &props.language, "settings.mirrors.empty")} }
            } else {
                div { class: "list-stack",
                    for mirror in mirrors {
                        div { class: "list-item",
                            div { class: "list-item-copy",
                                strong { "{mirror.name}" }
                                span { class: "meta-text", "{mirror.base_url}" }
                                span { class: "meta-text",
                                    {
                                        match mirror.mirror_type {
                                            MirrorType::OfficialCompatible => tr(&services, &props.language, "settings.mirrors.type.official"),
                                            MirrorType::BmclApiCompatible => tr(&services, &props.language, "settings.mirrors.type.bmclapi"),
                                        }
                                    }
                                }
                            }
                            div { class: "item-actions mirror-item-actions",
                                if store.read().settings.selected_mirror == mirror.base_url {
                                    span { class: "status-pill", {tr(&services, &props.language, "settings.mirrors.current")} }
                                }
                                if mirror.is_builtin {
                                    span { class: "todo-pill", {tr(&services, &props.language, "settings.mirrors.builtin")} }
                                }
                                button {
                                    class: if store.read().settings.selected_mirror == mirror.base_url { "mini-btn active" } else { "mini-btn" },
                                    onclick: {
                                        let services = services.clone();
                                        let mut store = store;
                                        let base_url = mirror.base_url.clone();
                                        move |_| {
                                            if store.read().settings.selected_mirror == base_url {
                                                return;
                                            }
                                            store.write().settings.selected_mirror = base_url.clone();
                                            let settings = store.read().settings.clone();
                                            let pool = services.pool.clone();
                                            spawn(async move {
                                                let _ = db::settings::save(&pool, &settings).await;
                                            });
                                        }
                                    },
                                    {tr(&services, &props.language, "action.select")}
                                }
                                if !mirror.is_builtin {
                                    button {
                                        class: "mini-btn danger",
                                        onclick: {
                                            let mut pending_delete_mirror = pending_delete_mirror;
                                            let mirror = mirror.clone();
                                            move |_| pending_delete_mirror.set(Some(mirror.clone()))
                                        },
                                        {tr(&services, &props.language, "settings.general.delete_mirror")}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if show_add_mirror_modal() {
            MirrorAddModal {
                language: props.language.clone(),
                mirror_name,
                mirror_url,
                mirror_type,
                on_close: EventHandler::new({
                    let mut show_add_mirror_modal = show_add_mirror_modal;
                    move |()| show_add_mirror_modal.set(false)
                }),
                on_confirm: EventHandler::new({
                    let services = services.clone();
                    let mut store = store;
                    let mut mirror_name = mirror_name;
                    let mut mirror_url = mirror_url;
                    let mut mirror_type = mirror_type;
                    let mut show_add_mirror_modal = show_add_mirror_modal;
                    let language = props.language.clone();
                    move |()| {
                        let name = mirror_name().trim().to_string();
                        let url = mirror_url().trim().to_string();
                        if name.is_empty() || url.is_empty() {
                            return;
                        }

                        let mirror = MirrorSource {
                            id: uuid::Uuid::new_v4().to_string(),
                            name,
                            base_url: url.clone(),
                            is_builtin: false,
                            mirror_type: mirror_type(),
                        };
                        store.write().settings.selected_mirror = url;
                        store.write().add_custom_mirror(mirror.clone());
                        store.write().notification = Some(tr(&services, &language, "notice.mirror_added"));
                        mirror_name.set(String::new());
                        mirror_url.set(String::new());
                        mirror_type.set(MirrorType::OfficialCompatible);
                        show_add_mirror_modal.set(false);

                        let pool = services.pool.clone();
                        let settings = store.read().settings.clone();
                        spawn(async move {
                            let _ = db::custom_mirrors::insert(&pool, &mirror).await;
                            let _ = db::settings::save(&pool, &settings).await;
                        });
                    }
                })
            }
        }
        if let Some(mirror) = pending_delete_mirror() {
            ConfirmModal {
                language: props.language.clone(),
                title: tr(&services, &props.language, "settings.general.delete_mirror"),
                message: format!("{}\n{}", mirror.name, tr(&services, &props.language, "settings.general.delete_mirror_confirm")),
                confirm_label: tr(&services, &props.language, "action.delete"),
                on_close: EventHandler::new({
                    let mut pending_delete_mirror = pending_delete_mirror;
                    move |()| pending_delete_mirror.set(None)
                }),
                on_confirm: EventHandler::new({
                    let services = services.clone();
                    let mut store = store;
                    let language = props.language.clone();
                    let mut pending_delete_mirror = pending_delete_mirror;
                    move |()| {
                        store.write().remove_custom_mirror(&mirror.id);
                        if store.read().settings.selected_mirror == mirror.base_url {
                            let fallback_url = store.read().custom_mirrors.first().map(|mirror| mirror.base_url.clone());
                            if let Some(fallback_url) = fallback_url {
                                store.write().settings.selected_mirror = fallback_url;
                            }
                        }
                        let pool = services.pool.clone();
                        let settings = store.read().settings.clone();
                        let deleting_id = mirror.id.clone();
                        store.write().notification = Some(tr(&services, &language, "notice.mirror_deleted"));
                        pending_delete_mirror.set(None);
                        spawn(async move {
                            let _ = db::custom_mirrors::delete(&pool, &deleting_id).await;
                            let _ = db::settings::save(&pool, &settings).await;
                        });
                    }
                })
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct MirrorAddModalProps {
    language: String,
    mirror_name: Signal<String>,
    mirror_url: Signal<String>,
    mirror_type: Signal<MirrorType>,
    on_close: EventHandler<()>,
    on_confirm: EventHandler<()>,
}

#[component]
fn MirrorAddModal(props: MirrorAddModalProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let mut mirror_name = props.mirror_name;
    let mut mirror_url = props.mirror_url;
    let mut mirror_type = props.mirror_type;

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card",
                div { class: "modal-header",
                    h3 { {tr(&services, &props.language, "settings.general.add_mirror")} }
                    button {
                        class: "toast-close",
                        onclick: move |_| props.on_close.call(()),
                        "×"
                    }
                }
                div { class: "inline-form",
                    input {
                        value: mirror_name(),
                        placeholder: tr(&services, &props.language, "settings.general.add_mirror_name"),
                        oninput: move |evt| mirror_name.set(evt.value())
                    }
                    input {
                        value: mirror_url(),
                        placeholder: tr(&services, &props.language, "settings.general.add_mirror_url"),
                        oninput: move |evt| mirror_url.set(evt.value())
                    }
                    select {
                        value: mirror_type().as_str().to_string(),
                        onchange: move |evt| mirror_type.set(MirrorType::from_str(&evt.value())),
                        option {
                            value: MirrorType::OfficialCompatible.as_str(),
                            {tr(&services, &props.language, "settings.mirrors.type.official")}
                        }
                        option {
                            value: MirrorType::BmclApiCompatible.as_str(),
                            {tr(&services, &props.language, "settings.mirrors.type.bmclapi")}
                        }
                    }
                }
                div { class: "pill-row",
                    button {
                        class: "mini-btn",
                        onclick: move |_| props.on_close.call(()),
                        {tr(&services, &props.language, "action.cancel")}
                    }
                    button {
                        class: "mini-btn active",
                        onclick: move |_| props.on_confirm.call(()),
                        {tr(&services, &props.language, "action.save")}
                    }
                }
            }
        }
    }
}

#[component]
fn JavaSettingsPanel(props: LanguageProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let show_add_modal = use_signal(|| false);
    let show_download_modal = use_signal(|| false);
    let show_scan_modal = use_signal(|| false);
    let scan_loading = use_signal(|| false);
    let scan_candidates = use_signal(Vec::<crate::domain::java::JavaRuntime>::new);
    let scan_selected_paths = use_signal(BTreeSet::<String>::new);
    let mut runtimes = store.read().java_runtimes.clone();
    runtimes.sort_by(|left, right| {
        right
            .is_default
            .cmp(&left.is_default)
            .then_with(|| left.created_at.cmp(&right.created_at))
    });

    rsx! {
        div { class: "detail-panel",
            div { class: "toolbar-row",
                h2 { {tr(&services, &props.language, "settings.java.title")} }
                div { class: "item-actions",
                    button {
                        class: "mini-btn active",
                        onclick: {
                            let mut show_add_modal = show_add_modal;
                            move |_| show_add_modal.set(true)
                        },
                        {tr(&services, &props.language, "settings.java.add")}
                    }
                    button {
                        class: "mini-btn",
                        onclick: {
                            let mut show_download_modal = show_download_modal;
                            move |_| show_download_modal.set(true)
                        },
                        {tr(&services, &props.language, "settings.java.download")}
                    }
                    button {
                        class: "mini-btn",
                        onclick: {
                            let mut show_scan_modal = show_scan_modal;
                            let mut scan_loading = scan_loading;
                            let mut scan_candidates = scan_candidates;
                            let mut scan_selected_paths = scan_selected_paths;
                            let store = store;
                            move |_| {
                                let existing_paths: BTreeSet<String> = store
                                    .read()
                                    .java_runtimes
                                    .iter()
                                    .map(|runtime| runtime.path.clone())
                                    .collect();
                                show_scan_modal.set(true);
                                scan_loading.set(true);
                                scan_candidates.set(Vec::new());
                                scan_selected_paths.set(BTreeSet::new());

                                spawn(async move {
                                    let runtimes = platform::java::scan_java_runtimes().await;
                                    let selected_paths = runtimes
                                        .iter()
                                        .filter(|runtime| !existing_paths.contains(&runtime.path))
                                        .map(|runtime| runtime.path.clone())
                                        .collect::<BTreeSet<_>>();
                                    scan_candidates.set(runtimes);
                                    scan_selected_paths.set(selected_paths);
                                    scan_loading.set(false);
                                });
                            }
                        },
                        {tr(&services, &props.language, "settings.java.refresh")}
                    }
                }
            }
            if store.read().java_runtimes.is_empty() {
                p { class: "empty-state", {tr(&services, &props.language, "settings.java.empty")} }
            } else {
                div { class: "list-stack",
                    for runtime in runtimes {
                        div { class: "list-item",
                            div {
                                strong { {runtime.name} }
                                span { class: "meta-text", {runtime.path} }
                            }
                            button {
                                class: if runtime.is_default { "mini-btn active" } else { "mini-btn" },
                                onclick: {
                                    let runtime_id = runtime.id.clone();
                                    let services = services.clone();
                                    let mut store = store;
                                    move |_| {
                                        if !store.write().set_default_java(&runtime_id) {
                                            return;
                                        }
                                        let pool = services.pool.clone();
                                        let selected_id = runtime_id.clone();
                                        spawn(async move {
                                            let _ = db::java::set_default(&pool, &selected_id).await;
                                        });
                                    }
                                },
                                {tr(&services, &props.language, "settings.java.make_default")}
                            }
                        }
                    }
                }
            }
        }
        if show_add_modal() {
            JavaAddModal {
                language: props.language.clone(),
                on_close: EventHandler::new({
                    let mut show_add_modal = show_add_modal;
                    move |()| show_add_modal.set(false)
                })
            }
        }
        if show_download_modal() {
            JavaDownloadModal {
                language: props.language.clone(),
                on_close: EventHandler::new({
                    let mut show_download_modal = show_download_modal;
                    move |()| show_download_modal.set(false)
                })
            }
        }
        if show_scan_modal() {
            JavaScanModal {
                language: props.language.clone(),
                loading: scan_loading(),
                candidates: scan_candidates(),
                selected_paths: scan_selected_paths,
                on_close: EventHandler::new({
                    let mut show_scan_modal = show_scan_modal;
                    move |()| show_scan_modal.set(false)
                }),
                on_confirm: EventHandler::new({
                    let services = services.clone();
                    let mut store = store;
                    let mut show_scan_modal = show_scan_modal;
                    let selected_paths = scan_selected_paths;
                    let candidates = scan_candidates;
                    let language = props.language.clone();
                    move |()| {
                        let selected_paths = selected_paths();
                        let mut merged = store.read().java_runtimes.clone();
                        let had_default = merged.iter().any(|runtime| runtime.is_default);
                        let existing_paths = merged
                            .iter()
                            .map(|runtime| runtime.path.clone())
                            .collect::<BTreeSet<_>>();

                        let mut additions = candidates()
                            .into_iter()
                            .filter(|runtime| selected_paths.contains(&runtime.path))
                            .filter(|runtime| !existing_paths.contains(&runtime.path))
                            .collect::<Vec<_>>();

                        if additions.is_empty() {
                            show_scan_modal.set(false);
                            return;
                        }

                        if !had_default {
                            if let Some(first) = additions.first_mut() {
                                first.is_default = true;
                            }
                        }

                        merged.extend(additions.clone());
                        merged.sort_by(|left, right| {
                            right
                                .is_default
                                .cmp(&left.is_default)
                                .then_with(|| left.created_at.cmp(&right.created_at))
                        });
                        store.write().replace_java_runtimes(merged);
                        store.write().notification = Some(format!(
                            "{} {}",
                            tr(&services, &language, "notice.java_scan_added"),
                            additions.len()
                        ));
                        show_scan_modal.set(false);

                        let pool = services.pool.clone();
                        spawn(async move {
                            for runtime in &additions {
                                let _ = db::java::insert(&pool, runtime).await;
                            }
                            if let Some(default_runtime) = additions.iter().find(|runtime| runtime.is_default) {
                                let _ = db::java::set_default(&pool, &default_runtime.id).await;
                            }
                        });
                    }
                })
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct JavaScanModalProps {
    language: String,
    loading: bool,
    candidates: Vec<crate::domain::java::JavaRuntime>,
    selected_paths: Signal<BTreeSet<String>>,
    on_close: EventHandler<()>,
    on_confirm: EventHandler<()>,
}

#[component]
fn JavaScanModal(props: JavaScanModalProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let existing_paths = store
        .read()
        .java_runtimes
        .iter()
        .map(|runtime| runtime.path.clone())
        .collect::<BTreeSet<_>>();

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card modal-card-wide",
                div { class: "modal-header",
                    h3 { {tr(&services, &props.language, "settings.java.scan_title")} }
                    button {
                        class: "toast-close",
                        onclick: move |_| props.on_close.call(()),
                        "×"
                    }
                }
                if props.loading {
                    p { class: "meta-text", {tr(&services, &props.language, "settings.java.scan_loading")} }
                } else if props.candidates.is_empty() {
                    p { class: "empty-state", {tr(&services, &props.language, "settings.java.scan_empty")} }
                } else {
                    p { class: "meta-text", {tr(&services, &props.language, "settings.java.scan_hint")} }
                    div { class: "list-stack",
                        for runtime in props.candidates.clone() {
                            div { class: "list-item",
                                div { class: "list-item-copy",
                                    strong { {format!("{} ({})", runtime.name, runtime.version)} }
                                    span { class: "meta-text", {runtime.path.clone()} }
                                }
                                label { class: "meta-text",
                                    input {
                                        r#type: "checkbox",
                                        checked: existing_paths.contains(&runtime.path) || props.selected_paths.read().contains(&runtime.path),
                                        disabled: existing_paths.contains(&runtime.path),
                                        onchange: {
                                            let mut selected_paths = props.selected_paths;
                                            let path = runtime.path.clone();
                                            move |_| {
                                                let mut next = selected_paths();
                                                if next.contains(&path) {
                                                    next.remove(&path);
                                                } else {
                                                    next.insert(path.clone());
                                                }
                                                selected_paths.set(next);
                                            }
                                        }
                                    }
                                    {
                                        if existing_paths.contains(&runtime.path) {
                                            tr(&services, &props.language, "settings.java.scan_exists")
                                        } else {
                                            tr(&services, &props.language, "action.select")
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "pill-row",
                    button {
                        class: "mini-btn",
                        onclick: move |_| props.on_close.call(()),
                        {tr(&services, &props.language, "action.cancel")}
                    }
                    if !props.loading && !props.candidates.is_empty() {
                        button {
                            class: "mini-btn active",
                            onclick: move |_| props.on_confirm.call(()),
                            {tr(&services, &props.language, "settings.java.scan_confirm")}
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct JavaDownloadModalProps {
    language: String,
    on_close: EventHandler<()>,
}

#[component]
fn JavaDownloadModal(props: JavaDownloadModalProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();

    let links = [
        (
            tr(&services, &props.language, "settings.java.download.oracle"),
            "https://www.oracle.com/cn/java/technologies/downloads/",
        ),
        (
            tr(
                &services,
                &props.language,
                "settings.java.download.adoptopenjdk",
            ),
            "https://adoptopenjdk.net/releases.html",
        ),
        (
            tr(&services, &props.language, "settings.java.download.azul"),
            "https://www.azul.com/downloads/?package=jdk#zulu",
        ),
    ];

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card",
                div { class: "modal-header",
                    h3 { {tr(&services, &props.language, "settings.java.download")} }
                    button {
                        class: "toast-close",
                        onclick: move |_| props.on_close.call(()),
                        "×"
                    }
                }
                div { class: "list-stack",
                    for (label, href) in links {
                        button {
                            class: "mini-btn",
                            onclick: {
                                let services = services.clone();
                                let mut store = store;
                                let language = props.language.clone();
                                let href = href.to_string();
                                move |_| {
                                    if platform::java::open_url(&href).is_err() {
                                        store.write().notification = Some(tr(&services, &language, "settings.java.open_download_failed"));
                                    }
                                }
                            },
                            "{label}"
                        }
                    }
                }
                div { class: "pill-row",
                    button {
                        class: "mini-btn",
                        onclick: move |_| props.on_close.call(()),
                        {tr(&services, &props.language, "action.cancel")}
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct JavaAddModalProps {
    language: String,
    on_close: EventHandler<()>,
}

#[component]
fn JavaAddModal(props: JavaAddModalProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let active_tab = use_signal(|| JavaAddTab::Path);
    let import_path = use_signal(String::new);
    let archive_path = use_signal(String::new);

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card",
                div { class: "modal-header",
                    h3 { {tr(&services, &props.language, "settings.java.add")} }
                    button {
                        class: "toast-close",
                        onclick: move |_| props.on_close.call(()),
                        "×"
                    }
                }
                div { class: "pill-row",
                    button {
                        class: if active_tab() == JavaAddTab::Path { "mini-btn active" } else { "mini-btn" },
                        onclick: {
                            let mut active_tab = active_tab;
                            move |_| active_tab.set(JavaAddTab::Path)
                        },
                        {tr(&services, &props.language, "settings.java.tab_path")}
                    }
                    button {
                        class: if active_tab() == JavaAddTab::Archive { "mini-btn active" } else { "mini-btn" },
                        onclick: {
                            let mut active_tab = active_tab;
                            move |_| active_tab.set(JavaAddTab::Archive)
                        },
                        {tr(&services, &props.language, "settings.java.tab_archive")}
                    }
                }
                if active_tab() == JavaAddTab::Path {
                    div { class: "inline-form",
                        input {
                            value: import_path(),
                            readonly: true,
                            placeholder: tr(&services, &props.language, "settings.java.import_placeholder"),
                        }
                        div { class: "pill-row",
                            button {
                                class: "mini-btn",
                                onclick: {
                                    let mut import_path = import_path;
                                    move |_| {
                                        spawn(async move {
                                            if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
                                                import_path.set(folder.path().display().to_string());
                                            }
                                        });
                                    }
                                },
                                {tr(&services, &props.language, "settings.java.choose_folder")}
                            }
                            button {
                                class: "mini-btn",
                                onclick: {
                                    let mut import_path = import_path;
                                    move |_| {
                                        spawn(async move {
                                            if let Some(file) = AsyncFileDialog::new().pick_file().await {
                                                import_path.set(file.path().display().to_string());
                                            }
                                        });
                                    }
                                },
                                {tr(&services, &props.language, "settings.java.choose_file")}
                            }
                        }
                    }
                } else {
                    div { class: "inline-form",
                        input {
                            value: archive_path(),
                            readonly: true,
                            placeholder: tr(&services, &props.language, "settings.java.import_archive_placeholder"),
                        }
                        button {
                            class: "mini-btn",
                            onclick: {
                                let mut archive_path = archive_path;
                                move |_| {
                                    spawn(async move {
                                        if let Some(file) = AsyncFileDialog::new()
                                            .add_filter("Archive", &["zip", "jar"])
                                            .pick_file()
                                            .await
                                        {
                                            archive_path.set(file.path().display().to_string());
                                        }
                                    });
                                }
                            },
                            {tr(&services, &props.language, "settings.java.choose_archive")}
                        }
                    }
                }
                div { class: "pill-row",
                    button {
                        class: "mini-btn",
                        onclick: move |_| props.on_close.call(()),
                        {tr(&services, &props.language, "action.cancel")}
                    }
                    button {
                        class: "mini-btn active",
                        onclick: {
                            let services = services.clone();
                            let store = store;
                            let import_path = import_path;
                            let archive_path = archive_path;
                            let active_tab = active_tab;
                            let language = props.language.clone();
                            let on_close = props.on_close.clone();
                            move |_| {
                                let selected = active_tab();
                                let source_value = match selected {
                                    JavaAddTab::Path => import_path().trim().to_string(),
                                    JavaAddTab::Archive => archive_path().trim().to_string(),
                                };
                                if source_value.is_empty() {
                                    return;
                                }

                                let services = services.clone();
                                let mut store = store;
                                let language = language.clone();
                                let on_close = on_close.clone();
                                spawn(async move {
                                    match selected {
                                        JavaAddTab::Path => {
                                            let path = std::path::PathBuf::from(source_value);
                                            if let Some(runtime) = platform::java::import_java_from_path(&path).await {
                                                let _ = db::java::insert(&services.pool, &runtime).await;
                                                let mut runtimes = store.read().java_runtimes.clone();
                                                runtimes.push(runtime);
                                                store.write().replace_java_runtimes(runtimes);
                                                store.write().notification = Some(tr(&services, &language, "notice.java_imported"));
                                                on_close.call(());
                                            } else {
                                                store.write().notification = Some(tr(&services, &language, "settings.java.import_invalid"));
                                            }
                                        }
                                        JavaAddTab::Archive => {
                                            let java_dir = services.paths.java_dir().to_path_buf();
                                            let archive = std::path::PathBuf::from(source_value);
                                            match platform::java::import_java_from_archive(&archive, &java_dir).await {
                                                Ok(Some(runtime)) => {
                                                    let _ = db::java::insert(&services.pool, &runtime).await;
                                                    let mut runtimes = store.read().java_runtimes.clone();
                                                    runtimes.push(runtime);
                                                    store.write().replace_java_runtimes(runtimes);
                                                    store.write().notification = Some(tr(&services, &language, "notice.java_archive_imported"));
                                                    on_close.call(());
                                                }
                                                Ok(None) => {
                                                    store.write().notification = Some(tr(&services, &language, "settings.java.archive_invalid"));
                                                }
                                                Err(_) => {
                                                    store.write().notification = Some(tr(&services, &language, "settings.java.archive_import_failed"));
                                                }
                                            }
                                        }
                                    }
                                });
                            }
                        },
                        {tr(&services, &props.language, "settings.java.confirm_add")}
                    }
                }
            }
        }
    }
}

#[component]
fn HelpPanel(props: LanguageProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();

    let help_sections = if props.language == "zh-CN" {
        vec![
            (
                "下载",
                vec![
                    "先到「下载」页刷新 Mojang 官方版本清单。",
                    "筛选正式版、快照版或已下载版本后，点击目标版本的下载按钮。",
                    "下载完成后，再回到主页选择该版本启动。",
                ],
            ),
            (
                "Java",
                vec![
                    "首次启动会自动扫描本机 Java，并选中第一个可用运行时。",
                    "如果已有本地 Java 目录或压缩包，也可以在 Java 设置页手动导入。",
                    "版本不兼容时，会先提示你切换 Java，再继续启动。",
                ],
            ),
            (
                "启动",
                vec![
                    "主页需要先选账号、版本和 Java，再点击启动游戏。",
                    "如果任一条件缺失，页面会给出明确的阻断提示。",
                    "启动日志会写入本地日志目录，便于后续排查。",
                ],
            ),
        ]
    } else {
        vec![
            (
                "Download",
                vec![
                    "Open the Download page and refresh the Mojang version manifest first.",
                    "Filter release, snapshot, or installed versions, then download the target version.",
                    "After the download completes, return to Home and select that version.",
                ],
            ),
            (
                "Java",
                vec![
                    "The first launch scans local Java runtimes and selects the first usable one.",
                    "You can also import a local Java folder or archive from the Java settings page.",
                    "When Java is incompatible, the app will prompt you before continuing.",
                ],
            ),
            (
                "Launch",
                vec![
                    "Home requires a selected account, version, and Java runtime before launching.",
                    "Missing requirements will be blocked with a clear message.",
                    "Launch logs are written to the local logs directory for troubleshooting.",
                ],
            ),
        ]
    };

    rsx! {
        div { class: "detail-panel",
            h2 { {tr(&services, &props.language, "settings.menu.help")} }
            p { class: "meta-text", { if props.language == "zh-CN" { "这里整理了启动器的最小使用路径，按这个顺序走基本不会卡住。" } else { "This page summarizes the shortest path through the launcher so you can get started quickly." } } }
            div { class: "list-stack",
                for (title, items) in help_sections {
                    div { class: "summary-card",
                        span { class: "summary-label", {title} }
                        div { class: "help-list",
                            for item in items {
                                p { class: "meta-text", {item} }
                            }
                        }
                    }
                }
            }
            p { class: "empty-state", {tr(&services, &store.read().settings.language, "settings.help.content")} }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SettingRowProps {
    label: String,
    content: Element,
}

#[component]
fn SettingRow(props: SettingRowProps) -> Element {
    rsx! {
        div { class: "setting-row",
            label { {props.label} }
            {props.content}
        }
    }
}

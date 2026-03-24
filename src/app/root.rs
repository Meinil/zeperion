use std::sync::Arc;
use std::time::Duration;

use chrono::{Local, Utc};
use dioxus::{
    desktop::{
        AssetRequest, RequestAsyncResponder, use_asset_handler, use_window,
        wry::http::{Response, StatusCode, header::CONTENT_TYPE},
    },
    prelude::*,
};
use dioxus_router::{Outlet, Router};

use crate::{
    app::{
        bootstrap::{self, AppServices},
        routes::Route,
    },
    components::{confirm_modal::ConfirmModal, top_nav::TopNav},
    infrastructure::db,
    state::AppMessage,
};

const MAIN_CSS_INLINE: &str = include_str!("../../assets/main.css");
const INPUT_SHORTCUT_GUARD_SCRIPT: &str = r#"
(() => {
  if (window.__zeperionInputShortcutGuardInstalled) return;
  window.__zeperionInputShortcutGuardInstalled = true;

  const textLikeInputTypes = new Set([
    "text", "password", "number", "url", "search", "email", "tel"
  ]);

  const isTextEditableTarget = (target) => {
    if (!target) return false;
    if (target instanceof HTMLTextAreaElement) return true;
    if (!(target instanceof HTMLInputElement)) return false;
    return textLikeInputTypes.has((target.type || "text").toLowerCase());
  };

  document.addEventListener("keydown", (event) => {
    const target = document.activeElement;
    if (!isTextEditableTarget(target)) return;
    if (!(event.ctrlKey || event.metaKey)) return;
    const key = event.key.toLowerCase();
    if (key === "a" || key === "c" || key === "v" || key === "x") {
      event.stopPropagation();
    }
  }, true);
})();
"#;

#[component]
pub fn App() -> Element {
    let boot = bootstrap::get_bootstrap();
    let services: Arc<AppServices> = boot.services.clone();
    let initial_store = boot.initial_store.clone();
    let _window = use_window();

    use_context_provider(|| services.clone());
    let store = use_context_provider(|| Signal::new(initial_store));
    let mut active_message = use_signal(|| None::<AppMessage>);

    let app_font_family = store.read().settings.font_family.clone();
    let app_font_size = store.read().settings.font_size;
    let app_theme_color = store.read().settings.theme_color.clone();
    let active_background_id = store
        .read()
        .backgrounds
        .iter()
        .find(|item| item.is_active)
        .map(|item| item.id.clone());
    use_asset_handler("background-image", {
        let store = store;
        move |request: AssetRequest, responder: RequestAsyncResponder| {
            let requested_id = request
                .uri()
                .path()
                .trim_matches('/')
                .split('/')
                .nth(1)
                .map(str::to_string);
            let requested_path = requested_id.as_deref().and_then(|id| {
                store
                    .read()
                    .backgrounds
                    .iter()
                    .find(|item| item.id == id)
                    .map(|item| item.local_path.clone())
            });

            tokio::spawn(async move {
                let response = match requested_path {
                    Some(path) => serve_background_asset(&path).await,
                    None => Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Vec::new())
                        .unwrap(),
                };
                responder.respond(response);
            });
        }
    });

    use_effect(move || {
        let notification = store.read().notification.clone();
        if let Some(message) = notification {
            let mut store = store;
            let stored_message = store.write().push_message(message);
            active_message.set(Some(stored_message.clone()));
            store.write().notification = None;
            let pool = services.pool.clone();
            let message_to_save = stored_message.clone();
            spawn(async move {
                let _ = db::messages::upsert(&pool, &message_to_save).await;
            });
            let mut active_message = active_message;
            spawn(async move {
                tokio::time::sleep(Duration::from_secs(4)).await;
                active_message.set(None);
            });
        }
    });

    rsx! {
        document::Style { {MAIN_CSS_INLINE} }
        document::Script {
            {INPUT_SHORTCUT_GUARD_SCRIPT}
        }

        div {
            class: "app-shell",
            style: app_shell_style(
                active_background_id.as_deref(),
                &app_font_family,
                app_font_size,
                &app_theme_color,
            ),
            Router::<Route> {}
            MessageFab {}
            ToastHost { active_message }
            MessageDrawer {}
        }
    }
}

fn app_shell_style(
    active_background_id: Option<&str>,
    font_family: &str,
    font_size: u16,
    theme_color: &str,
) -> String {
    let primary = normalize_hex_color(theme_color).unwrap_or("#F97316");
    let primary_soft = soften_hex_color(primary, 0.82);
    let mut style = format!(
        "--app-font-family: '{}'; --app-font-size: {}px; --primary: {}; --primary-soft: {};",
        font_family.replace('\'', "\\'"),
        font_size,
        primary,
        primary_soft
    );

    if let Some(id) = active_background_id {
        style.push_str(&format!(
            " background-image: url('{}'); background-size: contain; background-position: center; background-repeat: no-repeat;",
            background_asset_url(id)
        ));
    }

    style
}

fn normalize_hex_color(value: &str) -> Option<&str> {
    let bytes = value.as_bytes();
    if bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(u8::is_ascii_hexdigit) {
        Some(value)
    } else {
        None
    }
}

fn soften_hex_color(hex: &str, mix: f32) -> String {
    let parse = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    let (Some(r), Some(g), Some(b)) = (parse(1..3), parse(3..5), parse(5..7)) else {
        return "#ffedd5".to_string();
    };

    let blend = |channel: u8| -> u8 {
        let channel = channel as f32;
        let value = channel + (255.0 - channel) * mix;
        value.round().clamp(0.0, 255.0) as u8
    };

    format!("#{:02x}{:02x}{:02x}", blend(r), blend(g), blend(b))
}

fn background_asset_url(id: &str) -> String {
    format!("/background-image/{id}")
}

async fn serve_background_asset(path: &str) -> Response<Vec<u8>> {
    match tokio::fs::read(path).await {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, background_content_type(path))
            .body(bytes)
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Vec::new())
            .unwrap(),
    }
}

fn background_content_type(path: &str) -> &'static str {
    match std::path::Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

pub fn tr(services: &Arc<AppServices>, locale: &str, key: &str) -> String {
    services.i18n.translate(locale, key)
}

#[component]
pub fn AppLayout() -> Element {
    let services = use_context::<Arc<AppServices>>();
    let store = use_context::<Signal<crate::state::AppStore>>();
    let status_message = {
        let state = store.read();
        tr(&services, &state.settings.language, "status.initialized")
    };

    rsx! {
        div { class: "app-frame",
            TopNav {}
            div { class: "page-main",
                Outlet::<Route> {}
            }
            footer { class: "status-bar",
                span { "{status_message}" }
            }
        }
    }
}

#[component]
fn MessageFab() -> Element {
    let store = use_context::<Signal<crate::state::AppStore>>();
    let services = use_context::<Arc<AppServices>>();
    let language = store.read().settings.language.clone();
    let message_count = store.read().unread_message_count();

    rsx! {
        button {
            class: "message-fab",
            title: tr(&services, &language, "messages.title"),
            aria_label: tr(&services, &language, "messages.title"),
            onclick: {
                let mut store = store;
                move |_| {
                    let next = !store.read().message_drawer_open;
                    store.write().message_drawer_open = next;
                }
            },
            span { class: "message-btn-glyph", "◌" }
            if message_count > 0 {
                span { class: "message-badge", "{message_count}" }
            }
        }
    }
}

#[component]
fn ToastHost(active_message: Signal<Option<AppMessage>>) -> Element {
    let Some(message) = active_message() else {
        return rsx! {};
    };
    let store = use_context::<Signal<crate::state::AppStore>>();
    let services = use_context::<Arc<AppServices>>();
    let language = store.read().settings.language.clone();
    let message_id = message.id.clone();
    let message_content = message.content.clone();

    rsx! {
        div { class: "toast-host",
            div { class: "toast-banner",
                div { class: "toast-copy", {message_content} }
                button {
                    class: "toast-close",
                    aria_label: tr(&services, &language, "messages.close"),
                    onclick: {
                        let mut active_message = active_message;
                        let mut store = store;
                        let pool = services.pool.clone();
                        move |_| {
                            let should_persist = store.write().mark_message_read(&message_id);
                            active_message.set(None);
                            if should_persist {
                                let pool = pool.clone();
                                let message_id = message_id.clone();
                                spawn(async move {
                                    let _ = db::messages::mark_read(&pool, &message_id).await;
                                });
                            }
                        }
                    },
                    "×"
                }
            }
        }
    }
}

#[component]
fn MessageDrawer() -> Element {
    let store = use_context::<Signal<crate::state::AppStore>>();
    let services = use_context::<Arc<AppServices>>();
    let language = store.read().settings.language.clone();
    let is_open = store.read().message_drawer_open;
    let messages = store.read().messages.clone();
    let pending_delete_id = use_signal(|| None::<String>);
    let pending_clear_all = use_signal(|| false);

    if !is_open {
        return rsx! {};
    }

    rsx! {
        div { class: "drawer-backdrop",
            onclick: {
                let mut store = store;
                move |_| store.write().message_drawer_open = false
            },
            aside {
                class: "message-drawer",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "message-drawer-header",
                    h3 { {tr(&services, &language, "messages.title")} }
                    div { class: "item-actions",
                        button {
                            class: "mini-btn",
                            onclick: {
                                let mut pending_clear_all = pending_clear_all;
                                move |_| pending_clear_all.set(true)
                            },
                            {tr(&services, &language, "messages.clear_all")}
                        }
                        button {
                            class: "toast-close",
                            aria_label: tr(&services, &language, "messages.close"),
                            onclick: {
                                let mut store = store;
                                move |_| store.write().message_drawer_open = false
                            },
                            "×"
                        }
                    }
                }
                if messages.is_empty() {
                    p { class: "empty-state", {tr(&services, &language, "messages.empty")} }
                } else {
                    div { class: "message-list",
                        for message in messages {
                            div {
                                class: if message.is_read { "message-item" } else { "message-item unread" },
                                onclick: {
                                    let is_read = message.is_read;
                                    let message_id = message.id.clone();
                                    let mut store = store;
                                    let pool = services.pool.clone();
                                    move |_| {
                                        if is_read {
                                            return;
                                        }
                                        let should_persist = store.write().mark_message_read(&message_id);
                                        if should_persist {
                                            let pool = pool.clone();
                                            let message_id = message_id.clone();
                                            spawn(async move {
                                                let _ = db::messages::mark_read(&pool, &message_id).await;
                                            });
                                        }
                                    }
                                },
                                if !message.is_read {
                                    span { class: "message-unread-dot" }
                                }
                                div { class: "list-item-copy",
                                    strong { {format_message_time(message.created_at)} }
                                    span { class: "meta-text", {message.content.clone()} }
                                }
                                div { class: "item-actions",
                                    button {
                                        class: "mini-btn danger",
                                        onclick: {
                                            let message_id = message.id.clone();
                                            let mut pending_delete_id = pending_delete_id;
                                            move |evt| {
                                                evt.stop_propagation();
                                                pending_delete_id.set(Some(message_id.clone()))
                                            }
                                        },
                                        {tr(&services, &language, "messages.delete")}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(message_id) = pending_delete_id() {
            ConfirmModal {
                language: language.clone(),
                title: tr(&services, &language, "messages.delete"),
                message: tr(&services, &language, "messages.delete_confirm"),
                confirm_label: tr(&services, &language, "action.delete"),
                on_close: EventHandler::new({
                    let mut pending_delete_id = pending_delete_id;
                    move |()| pending_delete_id.set(None)
                }),
                on_confirm: EventHandler::new({
                    let mut store = store;
                    let mut pending_delete_id = pending_delete_id;
                    let pool = services.pool.clone();
                    move |()| {
                        store.write().remove_message(&message_id);
                        pending_delete_id.set(None);
                        let pool = pool.clone();
                        let message_id = message_id.clone();
                        spawn(async move {
                            let _ = db::messages::delete(&pool, &message_id).await;
                        });
                    }
                })
            }
        }
        if pending_clear_all() {
            ConfirmModal {
                language: language.clone(),
                title: tr(&services, &language, "messages.clear_all"),
                message: tr(&services, &language, "messages.clear_all_confirm"),
                confirm_label: tr(&services, &language, "messages.clear_all"),
                on_close: EventHandler::new({
                    let mut pending_clear_all = pending_clear_all;
                    move |()| pending_clear_all.set(false)
                }),
                on_confirm: EventHandler::new({
                    let mut store = store;
                    let mut pending_clear_all = pending_clear_all;
                    let pool = services.pool.clone();
                    move |()| {
                        store.write().clear_messages();
                        pending_clear_all.set(false);
                        let pool = pool.clone();
                        spawn(async move {
                            let _ = db::messages::clear(&pool).await;
                        });
                    }
                })
            }
        }
    }
}

fn format_message_time(value: chrono::DateTime<Utc>) -> String {
    value
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

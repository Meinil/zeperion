use dioxus::prelude::*;
use dioxus_router::{use_navigator, use_route};

use crate::{
    app::{
        bootstrap::AppServices,
        root::tr,
        routes::{Route, TopLevelRoute},
    },
    state::AppStore,
};

#[component]
pub fn TopNav() -> Element {
    let desktop = dioxus::desktop::use_window();
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<AppStore>>();
    let navigator = use_navigator();
    let route: Route = use_route();
    let language = store.read().settings.language.clone();
    let controls_on_leading = cfg!(target_os = "macos");
    let brand_drag = desktop.clone();
    let brand_toggle = desktop.clone();
    let spacer_drag = desktop.clone();
    let spacer_toggle = desktop.clone();
    let minimize_window = desktop.clone();
    let toggle_window = desktop.clone();
    let close_window = desktop.clone();

    let items = [
        (
            TopLevelRoute::Home,
            Route::HomeAccounts {},
            tr(&services, &language, "tab.home"),
        ),
        (
            TopLevelRoute::Download,
            Route::DownloadCore {},
            tr(&services, &language, "tab.download"),
        ),
        (
            TopLevelRoute::Settings,
            Route::SettingsGeneral {},
            tr(&services, &language, "tab.settings"),
        ),
        (
            TopLevelRoute::More,
            Route::More {},
            tr(&services, &language, "tab.more"),
        ),
    ];

    rsx! {
        header { class: "top-nav",
            div {
                class: "titlebar-side titlebar-left titlebar-drag-zone",
                onmousedown: move |_| brand_drag.drag(),
                ondoubleclick: move |_| brand_toggle.toggle_maximized(),
                if controls_on_leading {
                    WindowControls { language: language.clone(), minimize_window: minimize_window.clone(), toggle_window: toggle_window.clone(), close_window: close_window.clone(), leading: true }
                }
            }
            div { class: "titlebar-center",
                nav { class: "top-nav-links",
                    for (tab, target_route, label) in items {
                        button {
                            class: if route.top_level() == tab { "nav-btn active" } else { "nav-btn" },
                            onclick: {
                                let navigator = navigator.clone();
                                move |_| {
                                    navigator.push(target_route.clone());
                                }
                            },
                            "{label}"
                        }
                    }
                }
            }
            div {
                class: "titlebar-side titlebar-right titlebar-drag-zone",
                onmousedown: move |_| spacer_drag.drag(),
                ondoubleclick: move |_| spacer_toggle.toggle_maximized(),
                if !controls_on_leading {
                    WindowControls { language: language.clone(), minimize_window: minimize_window.clone(), toggle_window: toggle_window.clone(), close_window: close_window.clone(), leading: false }
                }
                div { class: "brand-mark", aria_hidden: "true",
                    span { class: "brand-glyph", "Z" }
                }
            }
        }
    }
}

#[derive(Props, Clone)]
struct WindowControlsProps {
    language: String,
    minimize_window: dioxus::desktop::DesktopContext,
    toggle_window: dioxus::desktop::DesktopContext,
    close_window: dioxus::desktop::DesktopContext,
    leading: bool,
}

impl PartialEq for WindowControlsProps {
    fn eq(&self, other: &Self) -> bool {
        self.language == other.language && self.leading == other.leading
    }
}

#[component]
fn WindowControls(props: WindowControlsProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let maximize_title = tr(&services, &props.language, "window.maximize_restore");

    let minimize = rsx! {
        button {
            class: if props.leading { "window-control-btn mac minimize" } else { "window-control-btn" },
            title: tr(&services, &props.language, "window.minimize"),
            aria_label: tr(&services, &props.language, "window.minimize"),
            onclick: move |_| props.minimize_window.set_minimized(true),
            if props.leading { "" } else { "-" }
        }
    };

    let maximize = rsx! {
        button {
            class: if props.leading { "window-control-btn mac maximize" } else { "window-control-btn" },
            title: maximize_title.clone(),
            aria_label: maximize_title.clone(),
            onclick: move |_| props.toggle_window.toggle_maximized(),
            if props.leading {
                ""
            } else {
                "□"
            }
        }
    };

    let close = rsx! {
        button {
            class: if props.leading { "window-control-btn mac close" } else { "window-control-btn close" },
            title: tr(&services, &props.language, "window.close"),
            aria_label: tr(&services, &props.language, "window.close"),
            onclick: move |_| props.close_window.close(),
            if props.leading { "" } else { "x" }
        }
    };

    rsx! {
        div { class: if props.leading { "window-controls mac-leading" } else { "window-controls" },
            if props.leading {
                {close}
                {minimize}
                {maximize}
            } else {
                {minimize}
                {maximize}
                {close}
            }
        }
    }
}

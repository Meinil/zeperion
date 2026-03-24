use dioxus::prelude::*;

use crate::app::{bootstrap::AppServices, root::tr};

#[derive(Props, PartialEq, Clone)]
pub struct ConfirmModalProps {
    pub language: String,
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub on_close: EventHandler<()>,
    pub on_confirm: EventHandler<()>,
}

#[component]
pub fn ConfirmModal(props: ConfirmModalProps) -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-card",
                div { class: "modal-header",
                    h3 { {props.title.clone()} }
                    button {
                        class: "toast-close",
                        onclick: move |_| props.on_close.call(()),
                        "×"
                    }
                }
                p { class: "meta-text", {props.message.clone()} }
                div { class: "pill-row",
                    button {
                        class: "mini-btn",
                        onclick: move |_| props.on_close.call(()),
                        {tr(&services, &props.language, "action.cancel")}
                    }
                    button {
                        class: "mini-btn danger active",
                        onclick: move |_| props.on_confirm.call(()),
                        {props.confirm_label.clone()}
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct SideMenuProps {
    pub items: Vec<SideMenuItem>,
}

#[derive(Clone, PartialEq)]
pub struct SideMenuItem {
    pub key: String,
    pub label: String,
    pub active: bool,
    pub on_click: EventHandler<MouseEvent>,
}

#[component]
pub fn SideMenu(props: SideMenuProps) -> Element {
    rsx! {
        aside { class: "side-menu",
            for item in props.items {
                button {
                    key: "{item.key}",
                    class: if item.active { "side-link active" } else { "side-link" },
                    onclick: move |evt| item.on_click.call(evt),
                    "{item.label}"
                }
            }
        }
    }
}

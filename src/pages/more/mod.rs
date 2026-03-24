use dioxus::prelude::*;

use crate::app::{bootstrap::AppServices, root::tr};

#[component]
pub fn MorePage() -> Element {
    let services = use_context::<std::sync::Arc<AppServices>>();
    let store = use_context::<Signal<crate::state::AppStore>>();
    let language = store.read().settings.language.clone();
    let is_zh = language == "zh-CN";

    let links = [
        ("GitHub", "https://github.com/"),
        ("Telegram", "https://telegram.org/"),
        ("QQ", "https://im.qq.com/"),
        ("X", "https://x.com/"),
        ("Facebook", "https://facebook.com/"),
    ];

    let open_source_title = if is_zh {
        "开源软件栈"
    } else {
        "Open Source Stack"
    };
    let resource_title = if is_zh {
        "资源出处"
    } else {
        "Resource Credits"
    };
    let acknowledgements_title = if is_zh { "鸣谢" } else { "Acknowledgements" };
    let about_title = if is_zh { "关于" } else { "About" };
    let about_summary = if is_zh {
        "一款基于 Rust 与 Dioxus 构建的 Minecraft 启动器。"
    } else {
        "A Minecraft launcher built with Rust and Dioxus."
    };

    let open_source_stack = if is_zh {
        vec![
            ("Rust", "承载启动器核心能力的系统编程语言。"),
            ("Dioxus", "用于构建桌面应用壳层的 UI 框架。"),
            ("Tokio", "负责网络请求、进程控制与定时任务的异步运行时。"),
            ("SQLite + sqlx", "用于设置、账号、任务等本地数据持久化。"),
            (
                "Reqwest",
                "用于获取 Mojang 元数据和下载文件的 HTTP 客户端。",
            ),
        ]
    } else {
        vec![
            (
                "Rust",
                "Systems programming language powering the launcher core.",
            ),
            (
                "Dioxus",
                "Desktop UI framework used for the application shell.",
            ),
            (
                "Tokio",
                "Async runtime used for networking, process control and timers.",
            ),
            (
                "SQLite + sqlx",
                "Local persistence layer for settings, accounts and tasks.",
            ),
            (
                "Reqwest",
                "HTTP client used for Mojang metadata and download requests.",
            ),
        ]
    };

    let resource_credits = if is_zh {
        vec![
            ("Mojang 官方接口", "用于版本清单、资源索引与下载元数据。"),
            ("Minecraft Wiki", "用于补充启动与资源组织方式的行为参考。"),
            ("HMCL 相关实现思路", "用于架构设计阶段的启动器流程参考。"),
            ("社区链接", "本页展示的社交入口与社区触点。"),
        ]
    } else {
        vec![
            (
                "Mojang official endpoints",
                "Version manifests, assets and download metadata.",
            ),
            ("Minecraft Wiki", "Launch and packaging behavior reference."),
            (
                "HMCL implementation ideas",
                "Launcher workflow reference during architecture design.",
            ),
            ("Community links", "Social entry points shown in this page."),
        ]
    };

    let acknowledgements = if is_zh {
        vec![
            "感谢 Rust 与开源生态，让这个启动器栈既紧凑又可靠。",
            "感谢 Dioxus 项目，提供了现代桌面 UI 的基础能力。",
            "感谢社区贡献的设计参考与实现经验，帮助我们更快完成落地。",
        ]
    } else {
        vec![
            "Thanks to the Rust and open source ecosystem for making the launcher stack compact and reliable.",
            "Thanks to the Dioxus project for the modern desktop UI foundation.",
            "Thanks to the community for launcher design references and implementation ideas.",
        ]
    };

    rsx! {
        section { class: "content-pane single-pane",
            div { class: "detail-panel",
                h2 { {tr(&services, &language, "more.title")} }
                div { class: "info-grid",
                    div { class: "summary-card",
                        span { class: "summary-label", {tr(&services, &language, "more.author")} }
                        strong { "meinil" }
                    }
                    div { class: "summary-card",
                        span { class: "summary-label", {tr(&services, &language, "more.version")} }
                        strong { "0.1.0" }
                    }
                    div { class: "summary-card",
                        span { class: "summary-label", {tr(&services, &language, "more.platforms")} }
                        strong { {tr(&services, &language, "more.platforms_value")} }
                    }
                }
                div { class: "detail-panel",
                    h3 { "{about_title}" }
                    p { class: "meta-text", "{about_summary}" }
                }
                div { class: "detail-panel",
                    h3 { "{open_source_title}" }
                    div { class: "list-stack",
                        for (name, description) in open_source_stack {
                            div { class: "list-item",
                                div {
                                    strong { {name} }
                                    span { class: "meta-text", {description} }
                                }
                            }
                        }
                    }
                }
                div { class: "detail-panel",
                    h3 { "{resource_title}" }
                    div { class: "list-stack",
                        for (name, description) in resource_credits {
                            div { class: "list-item",
                                div {
                                    strong { {name} }
                                    span { class: "meta-text", {description} }
                                }
                            }
                        }
                    }
                }
                div { class: "detail-panel",
                    h3 { "{acknowledgements_title}" }
                    div { class: "list-stack",
                        for message in acknowledgements {
                            div { class: "list-item",
                                span { class: "meta-text", {message} }
                            }
                        }
                    }
                }
                h3 { {tr(&services, &language, "more.community")} }
                div { class: "pill-row",
                    for (label, href) in links {
                        a { class: "todo-pill", href: "{href}", target: "_blank", "{label}" }
                    }
                }
            }
        }
    }
}

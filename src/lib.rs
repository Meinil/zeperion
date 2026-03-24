pub mod app;
pub mod components;
pub mod domain;
pub mod infrastructure;
pub mod pages;
pub mod platform;
pub mod state;
pub mod tasks;

use app::bootstrap::{bootstrap, set_bootstrap};
use dioxus::{
    LaunchBuilder,
    desktop::{Config, LogicalPosition, LogicalSize, WindowBuilder},
};

pub use app::root::App;

pub fn launch_desktop() {
    let runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let bootstrap = runtime
        .block_on(bootstrap())
        .expect("failed to bootstrap Zeperion Launcher");

    let window_prefs = bootstrap.window_prefs.clone();
    set_bootstrap(bootstrap);
    let mut window = WindowBuilder::new()
        .with_title("Zeperion Launcher")
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(window_prefs.width, window_prefs.height))
        .with_min_inner_size(LogicalSize::new(970.0, 630.0))
        .with_maximized(window_prefs.is_maximized);

    if !window_prefs.is_maximized {
        if let (Some(x), Some(y)) = (window_prefs.x, window_prefs.y) {
            window = window.with_position(LogicalPosition::new(x, y));
        }
    }

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(window)
                .with_background_color((0, 0, 0, 0)),
        )
        .launch(App);
}

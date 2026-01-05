//! NLA AI Video Creator
//! 
//! A local-first, AI-native Non-Linear Animation editor for generative video production.

mod app;
mod state;
mod timeline;

use dioxus::desktop::{Config, WindowBuilder, LogicalSize};

fn main() {
    // Configure the window
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("NLA AI Video Creator")
                .with_inner_size(LogicalSize::new(1280.0, 800.0))
                .with_resizable(true)
        )
        .with_menu(None); // Disable default menu bar

    // Launch the Dioxus desktop application
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app::App);
}

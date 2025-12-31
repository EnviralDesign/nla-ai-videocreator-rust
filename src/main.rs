//! NLA AI Video Creator
//! 
//! A local-first, AI-native Non-Linear Animation editor for generative video production.

mod app;

fn main() {
    // Launch the Dioxus desktop application
    dioxus::launch(app::App);
}

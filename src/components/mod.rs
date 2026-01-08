//! UI components grouped by feature domain.
pub mod common;
pub mod assets;
pub mod attributes;

mod startup_modal;
mod title_bar;
mod side_panel;
mod status_bar;
mod preview_panel;
mod providers_modal;
mod new_project_modal;
mod track_context_menu;

pub use startup_modal::StartupModal;
pub use title_bar::TitleBar;
pub use side_panel::SidePanel;
pub use status_bar::StatusBar;
pub use preview_panel::PreviewPanel;
pub use providers_modal::ProvidersModal;
pub use new_project_modal::NewProjectModal;
pub use track_context_menu::TrackContextMenu;

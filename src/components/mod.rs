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
mod providers_modal_v2;
mod provider_json_editor_modal;
mod provider_builder_modal_v2;
mod new_project_modal;
mod track_context_menu;
mod generation_queue_panel;

pub use startup_modal::{StartupModal, StartupModalMode};
pub use title_bar::TitleBar;
pub use side_panel::SidePanel;
pub use status_bar::StatusBar;
pub use preview_panel::PreviewPanel;
pub use providers_modal_v2::ProvidersModalV2;
pub use provider_json_editor_modal::ProviderJsonEditorModal;
pub use provider_builder_modal_v2::ProviderBuilderModalV2;
pub use new_project_modal::NewProjectModal;
pub use track_context_menu::TrackContextMenu;
pub use generation_queue_panel::GenerationQueuePanel;

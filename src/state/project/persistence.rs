use std::fs;
use std::io;
use std::path::Path;
use uuid::Uuid;

use crate::state::{AssetKind, GenerativeConfig};
use super::{Project, ProjectSettings};

impl Project {
// =========================================================================
    // Save/Load
    // =========================================================================

    /// Save the project to its folder
    #[allow(dead_code)]
    pub fn save(&self) -> io::Result<()> {
        let path = self.project_path.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Project path not set")
        })?;
        self.save_to(path)
    }

    /// Save the project to a specific folder
    pub fn save_to(&self, folder: &Path) -> io::Result<()> {
        // Create the project folder structure
        fs::create_dir_all(folder)?;
        fs::create_dir_all(folder.join("audio"))?;
        fs::create_dir_all(folder.join("images"))?;
        fs::create_dir_all(folder.join("video"))?;
        fs::create_dir_all(folder.join("generated"))?;
        fs::create_dir_all(folder.join("generated/video"))?;
        fs::create_dir_all(folder.join("generated/image"))?;
        fs::create_dir_all(folder.join("generated/audio"))?;
        fs::create_dir_all(folder.join("exports"))?;

        // Write project.json
        let json = serde_json::to_string_pretty(self)?;
        fs::write(folder.join("project.json"), json)?;

        Ok(())
    }

    /// Load a project from a folder
    pub fn load(folder: &Path) -> io::Result<Self> {
        let project_file = folder.join("project.json");
        let json = fs::read_to_string(&project_file)?;
        let mut project: Project = serde_json::from_str(&json)?;
        project.project_path = Some(folder.to_path_buf());
        project.sync_generative_configs();
        Ok(project)
    }

    /// Create a new project in a folder
    #[allow(dead_code)]
    pub fn create_in(folder: &Path, name: impl Into<String>) -> io::Result<Self> {
        let mut project = Project::new(name);
        project.project_path = Some(folder.to_path_buf());
        project.save_to(folder)?;
        Ok(project)
    }

    /// Create a new project in a folder with explicit settings
    pub fn create_in_with_settings(
        folder: &Path,
        name: impl Into<String>,
        settings: ProjectSettings,
    ) -> io::Result<Self> {
        let mut project = Project::new(name);
        project.settings = settings;
        project.project_path = Some(folder.to_path_buf());
        project.save_to(folder)?;
        Ok(project)
    }

    /// Save the current project to a new folder (initializing it)
    #[allow(dead_code)]
    pub fn save_project_as(&mut self, folder: &Path, name: impl Into<String>) -> io::Result<()> {
        self.name = name.into();
        self.project_path = Some(folder.to_path_buf());
        self.save_to(folder)?;
        Ok(())
    }

    pub fn set_generative_active_version(
        &mut self,
        asset_id: Uuid,
        version: Option<String>,
    ) {
        if let Some(asset) = self.assets.iter_mut().find(|asset| asset.id == asset_id) {
            match &mut asset.kind {
                AssetKind::GenerativeVideo { active_version, .. }
                | AssetKind::GenerativeImage { active_version, .. }
                | AssetKind::GenerativeAudio { active_version, .. } => {
                    *active_version = version;
                }
                _ => {}
            }
        }
    }

    fn sync_generative_configs(&mut self) {
        let Some(project_root) = self.project_path.clone() else {
            return;
        };

        for asset in self.assets.iter_mut() {
            let (folder, active_version, provider_id) = match &mut asset.kind {
                AssetKind::GenerativeVideo {
                    folder,
                    active_version,
                    provider_id,
                }
                | AssetKind::GenerativeImage {
                    folder,
                    active_version,
                    provider_id,
                }
                | AssetKind::GenerativeAudio {
                    folder,
                    active_version,
                    provider_id,
                } => (folder, active_version, provider_id),
                _ => continue,
            };

            let folder_path = project_root.join(folder);
            let config_path = folder_path.join("config.json");
            let mut config = GenerativeConfig::load(&folder_path).unwrap_or_default();
            let mut changed = !config_path.exists();

            if config.active_version.is_none() {
                if let Some(existing) = active_version.clone() {
                    config.active_version = Some(existing);
                    changed = true;
                }
            }

            if config.provider_id.is_none() {
                if let Some(existing) = provider_id.clone() {
                    config.provider_id = Some(existing);
                    changed = true;
                }
            }

            if config.active_version != *active_version {
                *active_version = config.active_version.clone();
            }

            if config.provider_id != *provider_id {
                *provider_id = config.provider_id;
            }

            if changed {
                let _ = config.save(&folder_path);
            }
        }
    }

    pub fn set_generative_provider_id(
        &mut self,
        asset_id: Uuid,
        provider_id: Option<Uuid>,
    ) {
        if let Some(asset) = self.assets.iter_mut().find(|asset| asset.id == asset_id) {
            match &mut asset.kind {
                AssetKind::GenerativeVideo { provider_id: stored, .. }
                | AssetKind::GenerativeImage { provider_id: stored, .. }
                | AssetKind::GenerativeAudio { provider_id: stored, .. } => {
                    *stored = provider_id;
                }
                _ => {}
            }
        }
    }
}

#![allow(dead_code)]
//! Generative asset config model and persistence helpers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Input value bound to a provider field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputValue {
    AssetRef { asset_id: Uuid },
    Literal { value: serde_json::Value },
}

/// A single generation record for a generative asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationRecord {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub provider_id: Uuid,
    pub inputs_snapshot: HashMap<String, InputValue>,
}

/// Persistent config stored in `generated/.../config.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerativeConfig {
    #[serde(default)]
    pub provider_id: Option<Uuid>,
    #[serde(default)]
    pub inputs: HashMap<String, InputValue>,
    #[serde(default)]
    pub versions: Vec<GenerationRecord>,
    #[serde(default)]
    pub active_version: Option<String>,
}

impl Default for GenerativeConfig {
    fn default() -> Self {
        Self {
            provider_id: None,
            inputs: HashMap::new(),
            versions: Vec::new(),
            active_version: None,
        }
    }
}

impl GenerativeConfig {
    pub fn load(folder: &Path) -> io::Result<Self> {
        let path = config_path(folder);
        let json = match fs::read_to_string(&path) {
            Ok(json) => json,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(err) => return Err(err),
        };
        let config = serde_json::from_str(&json)?;
        Ok(config)
    }

    pub fn save(&self, folder: &Path) -> io::Result<()> {
        fs::create_dir_all(folder)?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(config_path(folder), json)?;
        Ok(())
    }
}

fn config_path(folder: &Path) -> PathBuf {
    folder.join("config.json")
}

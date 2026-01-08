#![allow(dead_code)]
//! Provider configuration data model.
//!
//! Providers describe external generation backends (ComfyUI, APIs, etc.).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The output media type produced by a provider entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderOutputType {
    Image,
    Video,
    Audio,
}

/// Input types supported by provider schemas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderInputType {
    Image,
    Video,
    Audio,
    Text,
    Number,
    Integer,
    Boolean,
    Enum { options: Vec<String> },
}

/// Schema field describing a single provider input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderInputField {
    pub name: String,
    pub label: String,
    pub input_type: ProviderInputType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

/// Connection configuration for a provider entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderConnection {
    ComfyUi {
        base_url: String,
        #[serde(default)]
        workflow_path: Option<String>,
    },
    CustomHttp { base_url: String, api_key: Option<String> },
}

/// A configured provider entry stored on disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderEntry {
    pub id: Uuid,
    pub name: String,
    pub output_type: ProviderOutputType,
    #[serde(default)]
    pub inputs: Vec<ProviderInputField>,
    pub connection: ProviderConnection,
}

impl ProviderEntry {
    pub fn new(
        name: impl Into<String>,
        output_type: ProviderOutputType,
        connection: ProviderConnection,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            output_type,
            inputs: Vec::new(),
            connection,
        }
    }
}

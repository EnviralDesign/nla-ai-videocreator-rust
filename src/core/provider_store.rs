#![allow(dead_code)]
//! Provider storage helpers for `.providers/` configs.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::state::ProviderEntry;

pub fn load_provider_entries(project_root: &Path) -> io::Result<Vec<ProviderEntry>> {
    let root = providers_root(project_root);
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !is_json_file(&path) {
            continue;
        }
        let json = fs::read_to_string(&path)?;
        let provider: ProviderEntry = serde_json::from_str(&json)?;
        entries.push(provider);
    }

    Ok(entries)
}

pub fn save_provider_entry(project_root: &Path, entry: &ProviderEntry) -> io::Result<PathBuf> {
    let root = providers_root(project_root);
    fs::create_dir_all(&root)?;
    let path = root.join(format!("{}.json", entry.id));
    let json = serde_json::to_string_pretty(entry)?;
    fs::write(&path, json)?;
    Ok(path)
}

fn providers_root(project_root: &Path) -> PathBuf {
    project_root.join(".providers")
}

fn is_json_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
}

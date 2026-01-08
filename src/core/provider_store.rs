#![allow(dead_code)]
//! Provider storage helpers for provider configs.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::state::ProviderEntry;

pub fn load_provider_entries(project_root: &Path) -> io::Result<Vec<ProviderEntry>> {
    load_provider_entries_from(&providers_root(project_root))
}

pub fn load_global_provider_entries() -> io::Result<Vec<ProviderEntry>> {
    load_provider_entries_from(&global_providers_root())
}

pub fn load_global_provider_entries_or_empty() -> Vec<ProviderEntry> {
    match load_global_provider_entries() {
        Ok(entries) => entries,
        Err(err) => {
            println!("Failed to load provider entries: {}", err);
            Vec::new()
        }
    }
}

pub fn save_provider_entry(project_root: &Path, entry: &ProviderEntry) -> io::Result<PathBuf> {
    save_provider_entry_to(&providers_root(project_root), entry)
}

pub fn save_global_provider_entry(entry: &ProviderEntry) -> io::Result<PathBuf> {
    save_provider_entry_to(&global_providers_root(), entry)
}

pub fn global_providers_root() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("APPDATA"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    base.join("NLA-AI-VideoCreator").join("providers")
}

pub fn list_global_provider_files() -> Vec<PathBuf> {
    let root = global_providers_root();
    let mut files = Vec::new();
    let read_dir = match fs::read_dir(&root) {
        Ok(read_dir) => read_dir,
        Err(_) => return files,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if is_json_file(&path) {
            files.push(path);
        }
    }
    files.sort();
    files
}

pub fn read_provider_file(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

pub fn write_provider_file(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

pub fn provider_path_for_entry(entry: &ProviderEntry) -> PathBuf {
    global_providers_root().join(format!("{}.json", entry.id))
}

pub fn default_provider_entry() -> ProviderEntry {
    let mut entry = ProviderEntry::new(
        "New Provider",
        crate::state::ProviderOutputType::Image,
        crate::state::ProviderConnection::ComfyUi {
            base_url: "http://127.0.0.1:8188".to_string(),
            workflow_path: Some("workflows/sdxl_simple_example_API.json".to_string()),
        },
    );
    entry.inputs = Vec::new();
    entry
}

fn providers_root(project_root: &Path) -> PathBuf {
    project_root.join(".providers")
}

fn load_provider_entries_from(root: &Path) -> io::Result<Vec<ProviderEntry>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                println!("Failed to read provider entry: {}", err);
                continue;
            }
        };
        let path = entry.path();
        if !is_json_file(&path) {
            continue;
        }
        let json = match fs::read_to_string(&path) {
            Ok(json) => json,
            Err(err) => {
                println!("Failed to read provider config {:?}: {}", path, err);
                continue;
            }
        };
        let provider: ProviderEntry = match serde_json::from_str(&json) {
            Ok(provider) => provider,
            Err(err) => {
                println!("Failed to parse provider config {:?}: {}", path, err);
                continue;
            }
        };
        entries.push(provider);
    }

    Ok(entries)
}

fn save_provider_entry_to(root: &Path, entry: &ProviderEntry) -> io::Result<PathBuf> {
    fs::create_dir_all(root)?;
    let path = root.join(format!("{}.json", entry.id));
    let json = serde_json::to_string_pretty(entry)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    fs::write(&path, json)?;
    Ok(path)
}

fn is_json_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
}

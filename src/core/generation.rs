use std::collections::HashMap;

use serde_json::Value;

use crate::state::{GenerativeConfig, InputValue, ProviderEntry};

#[derive(Debug, Clone)]
pub struct ResolvedInputs {
    pub values: HashMap<String, Value>,
    pub snapshot: HashMap<String, InputValue>,
    pub missing_required: Vec<String>,
}

pub fn resolve_provider_inputs(
    provider: &ProviderEntry,
    config: &GenerativeConfig,
) -> ResolvedInputs {
    let mut values = HashMap::new();
    let mut snapshot = HashMap::new();
    let mut missing_required = Vec::new();

    for input in provider.inputs.iter() {
        let value = literal_input_value(config, &input.name)
            .or_else(|| input.default.clone());

        if let Some(value) = value {
            values.insert(input.name.clone(), value.clone());
            snapshot.insert(
                input.name.clone(),
                InputValue::Literal { value },
            );
        } else if input.required {
            missing_required.push(input.name.clone());
        }
    }

    ResolvedInputs {
        values,
        snapshot,
        missing_required,
    }
}

pub fn next_version_label(config: &GenerativeConfig) -> String {
    let mut max_version = 0u32;
    for record in config.versions.iter() {
        if let Some(number) = parse_version_number(&record.version) {
            max_version = max_version.max(number);
        }
    }
    if let Some(active) = config.active_version.as_ref() {
        if let Some(number) = parse_version_number(active) {
            max_version = max_version.max(number);
        }
    }
    format!("v{}", max_version + 1)
}

fn literal_input_value(config: &GenerativeConfig, name: &str) -> Option<Value> {
    config.inputs.get(name).and_then(|input| match input {
        InputValue::Literal { value } => Some(value.clone()),
        _ => None,
    })
}

fn parse_version_number(version: &str) -> Option<u32> {
    let trimmed = version.trim();
    let numeric = trimmed.strip_prefix('v').or_else(|| trimmed.strip_prefix('V'))?;
    numeric.parse::<u32>().ok()
}

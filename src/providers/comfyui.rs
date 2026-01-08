use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

const DEFAULT_WORKFLOW_PATH: &str = "workflows/sdxl_simple_example_API.json";
const OUTPUT_NODE_ID: &str = "53";

#[derive(Debug, Clone)]
pub struct ComfyUiImage {
    pub bytes: Vec<u8>,
    pub extension: String,
}

#[derive(Debug, Clone, Copy)]
enum InputCoercion {
    String,
    Integer,
    Float,
}

struct WorkflowInputBinding {
    name: &'static str,
    node_id: &'static str,
    input_key: &'static str,
    coercion: InputCoercion,
}

const WORKFLOW_INPUTS: &[WorkflowInputBinding] = &[
    WorkflowInputBinding {
        name: "prompt",
        node_id: "6",
        input_key: "text",
        coercion: InputCoercion::String,
    },
    WorkflowInputBinding {
        name: "negative_prompt",
        node_id: "7",
        input_key: "text",
        coercion: InputCoercion::String,
    },
    WorkflowInputBinding {
        name: "seed",
        node_id: "10",
        input_key: "noise_seed",
        coercion: InputCoercion::Integer,
    },
    WorkflowInputBinding {
        name: "steps",
        node_id: "10",
        input_key: "steps",
        coercion: InputCoercion::Integer,
    },
    WorkflowInputBinding {
        name: "cfg",
        node_id: "10",
        input_key: "cfg",
        coercion: InputCoercion::Float,
    },
    WorkflowInputBinding {
        name: "width",
        node_id: "5",
        input_key: "width",
        coercion: InputCoercion::Integer,
    },
    WorkflowInputBinding {
        name: "height",
        node_id: "5",
        input_key: "height",
        coercion: InputCoercion::Integer,
    },
    WorkflowInputBinding {
        name: "checkpoint",
        node_id: "4",
        input_key: "ckpt_name",
        coercion: InputCoercion::String,
    },
    WorkflowInputBinding {
        name: "sampler",
        node_id: "10",
        input_key: "sampler_name",
        coercion: InputCoercion::String,
    },
    WorkflowInputBinding {
        name: "scheduler",
        node_id: "10",
        input_key: "scheduler",
        coercion: InputCoercion::String,
    },
    WorkflowInputBinding {
        name: "start_step",
        node_id: "68",
        input_key: "value",
        coercion: InputCoercion::Integer,
    },
];

pub fn resolve_workflow_path(path: Option<&str>) -> PathBuf {
    let resolved = path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_WORKFLOW_PATH));
    if resolved.is_absolute() {
        resolved
    } else {
        let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let candidate = base.join(&resolved);
        if candidate.exists() {
            return candidate;
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let exe_candidate = parent.join(&resolved);
                if exe_candidate.exists() {
                    return exe_candidate;
                }
            }
        }
        base.join(resolved)
    }
}

pub async fn generate_image(
    base_url: &str,
    workflow_path: &Path,
    inputs: &HashMap<String, Value>,
) -> Result<ComfyUiImage, String> {
    let mut workflow = load_workflow(workflow_path)?;
    apply_inputs(&mut workflow, inputs)?;

    let client = reqwest::Client::new();
    let prompt_id = submit_prompt(&client, base_url, &workflow).await?;
    let outputs = poll_history(&client, base_url, &prompt_id).await?;
    let image_ref = find_image_output(&outputs)
        .ok_or_else(|| "ComfyUI history did not include image outputs.".to_string())?;
    let bytes = download_image(&client, base_url, &image_ref).await?;

    let extension = Path::new(&image_ref.filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("png")
        .to_string();

    Ok(ComfyUiImage { bytes, extension })
}

fn load_workflow(path: &Path) -> Result<Value, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|err| format!("Failed to read workflow: {}", err))?;
    serde_json::from_str(&json).map_err(|err| format!("Invalid workflow JSON: {}", err))
}

fn apply_inputs(workflow: &mut Value, inputs: &HashMap<String, Value>) -> Result<(), String> {
    for binding in WORKFLOW_INPUTS.iter() {
        let Some(value) = inputs.get(binding.name) else {
            continue;
        };
        let coerced = coerce_value(value, binding.coercion)
            .map_err(|err| format!("Input {}: {}", binding.name, err))?;
        set_workflow_input(workflow, binding.node_id, binding.input_key, coerced)?;
    }
    Ok(())
}

fn set_workflow_input(
    workflow: &mut Value,
    node_id: &str,
    input_key: &str,
    value: Value,
) -> Result<(), String> {
    let Some(node) = workflow.get_mut(node_id) else {
        return Err(format!("Workflow missing node {}", node_id));
    };
    let Some(inputs) = node.get_mut("inputs") else {
        return Err(format!("Workflow node {} missing inputs", node_id));
    };
    let Some(inputs) = inputs.as_object_mut() else {
        return Err(format!("Workflow node {} inputs not an object", node_id));
    };
    inputs.insert(input_key.to_string(), value);
    Ok(())
}

fn coerce_value(value: &Value, kind: InputCoercion) -> Result<Value, String> {
    match kind {
        InputCoercion::String => Ok(Value::String(match value {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        })),
        InputCoercion::Integer => {
            let number = if let Some(num) = value.as_i64() {
                serde_json::Number::from(num)
            } else if let Some(num) = value.as_u64() {
                serde_json::Number::from(num)
            } else if let Some(num) = value.as_f64() {
                serde_json::Number::from(num.round() as i64)
            } else if let Some(text) = value.as_str() {
                let parsed = text
                    .trim()
                    .parse::<i64>()
                    .map_err(|_| format!("Expected integer, got {}", text))?;
                serde_json::Number::from(parsed)
            } else {
                return Err("Expected integer value".to_string());
            };
            Ok(Value::Number(number))
        }
        InputCoercion::Float => {
            let number = if let Some(num) = value.as_f64() {
                serde_json::Number::from_f64(num)
            } else if let Some(text) = value.as_str() {
                let parsed = text
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| format!("Expected float, got {}", text))?;
                serde_json::Number::from_f64(parsed)
            } else {
                None
            }
            .ok_or_else(|| "Expected float value".to_string())?;
            Ok(Value::Number(number))
        }
    }
}

async fn submit_prompt(
    client: &reqwest::Client,
    base_url: &str,
    workflow: &Value,
) -> Result<String, String> {
    let url = format!("{}/prompt", base_url.trim_end_matches('/'));
    let response = client
        .post(url)
        .json(&serde_json::json!({ "prompt": workflow }))
        .send()
        .await
        .map_err(|err| format!("Failed to submit prompt: {}", err))?;
    let status = response.status();
    let payload: Value = response
        .json()
        .await
        .map_err(|err| format!("Failed to parse prompt response: {}", err))?;
    if !status.is_success() {
        return Err(format!(
            "ComfyUI rejected prompt ({}): {}",
            status,
            payload
        ));
    }
    payload
        .get("prompt_id")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| "ComfyUI response missing prompt_id".to_string())
}

async fn poll_history(
    client: &reqwest::Client,
    base_url: &str,
    prompt_id: &str,
) -> Result<Value, String> {
    let url = format!(
        "{}/history/{}",
        base_url.trim_end_matches('/'),
        prompt_id
    );
    for _ in 0..240 {
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|err| format!("Failed to query history: {}", err))?;
        let payload: Value = response
            .json()
            .await
            .map_err(|err| format!("Failed to parse history: {}", err))?;

        if let Some(outputs) = extract_outputs(&payload, prompt_id) {
            return Ok(outputs.clone());
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Err("Timed out waiting for ComfyUI output.".to_string())
}

fn extract_outputs<'a>(payload: &'a Value, prompt_id: &str) -> Option<&'a Value> {
    if let Some(outputs) = payload.get("outputs") {
        return Some(outputs);
    }
    payload.get(prompt_id)?.get("outputs")
}

struct ImageRef {
    filename: String,
    subfolder: String,
    kind: String,
}

fn find_image_output(outputs: &Value) -> Option<ImageRef> {
    if let Some(output) = outputs.get(OUTPUT_NODE_ID) {
        if let Some(image) = extract_first_image(output) {
            return Some(image);
        }
    }

    outputs
        .as_object()
        .and_then(|map| {
            map.values()
                .find_map(|value| extract_first_image(value))
        })
}

fn extract_first_image(output: &Value) -> Option<ImageRef> {
    let images = output.get("images")?.as_array()?;
    let first = images.first()?;
    let filename = first.get("filename")?.as_str()?.to_string();
    let subfolder = first
        .get("subfolder")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
    let kind = first
        .get("type")
        .and_then(|value| value.as_str())
        .unwrap_or("output")
        .to_string();
    Some(ImageRef {
        filename,
        subfolder,
        kind,
    })
}

async fn download_image(
    client: &reqwest::Client,
    base_url: &str,
    image: &ImageRef,
) -> Result<Vec<u8>, String> {
    let url = format!(
        "{}/view?filename={}&subfolder={}&type={}",
        base_url.trim_end_matches('/'),
        urlencoding::encode(&image.filename),
        urlencoding::encode(&image.subfolder),
        urlencoding::encode(&image.kind),
    );
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|err| format!("Failed to download image: {}", err))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("ComfyUI image download failed: {}", status));
    }
    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|err| format!("Failed to read image bytes: {}", err))
}

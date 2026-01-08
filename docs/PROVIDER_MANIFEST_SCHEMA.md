# Provider Manifest Schema (Draft)

This document defines a versioned manifest format that bridges a provider's
internal workflow/API details and the clean input UI shown in NLA.

The manifest is **adapter-specific** but shares a common header so the app can
load it consistently.

## Common Header

```json
{
  "schema_version": 1,
  "adapter_type": "comfyui",
  "name": "SDXL Simple",
  "output_type": "image",
  "workflow": "... adapter specific ...",
  "inputs": [ "... adapter specific ..." ],
  "output": { "... adapter specific ..." }
}
```

### Fields

- `schema_version`: Integer. Enables migrations later.
- `adapter_type`: `comfyui`, `custom_http`, `fal`, etc.
- `name`: Display name for the provider entry (optional; UI can override).
- `output_type`: `image`, `video`, or `audio`.
- `workflow`: Adapter-specific payload (workflow path or endpoint config).
- `inputs`: Array of exposed inputs for the editor UI.
- `output`: Adapter-specific output selection.

## ComfyUI Manifest (Adapter: `comfyui`)

### Required fields

```json
{
  "schema_version": 1,
  "adapter_type": "comfyui",
  "name": "SDXL Simple",
  "output_type": "image",
  "workflow": {
    "workflow_path": "workflows/sdxl_simple_example_API.json",
    "workflow_hash": "sha256:..."
  },
  "inputs": [ ... ],
  "output": {
    "selector": { "class_type": "PreviewImage", "tag": "final_output" },
    "index": 0
  }
}
```

### Input schema

```json
{
  "name": "prompt",
  "label": "Prompt",
  "input_type": "text",
  "required": true,
  "default": null,
  "ui": {
    "placeholder": "Describe the scene...",
    "multiline": true,
    "group": "Prompt",
    "advanced": false
  },
  "bind": {
    "selector": {
      "tag": "prompt_text",
      "class_type": "CLIPTextEncode",
      "input_key": "text",
      "title": "CLIP Text Encode (Prompt)"
    },
    "transform": null
  }
}
```

### Selector rules

Selectors resolve without node IDs:

1. Match by `tag` if present.
2. Otherwise match by `class_type + input_key + title`.
3. If ambiguous, mark provider **Needs Rebind**.

### UI hints

The `ui` block influences editor widgets only:

- `min` / `max` / `step` for numeric fields
- `placeholder`, `multiline` for text fields
- `group`, `advanced` for display grouping
- `unit` for display (e.g., "px", "s")

## Custom HTTP Manifest (Adapter: `custom_http`)

This is a **future** adapter example. It lets the builder map inputs into a
REST-style request without custom code.

```json
{
  "schema_version": 1,
  "adapter_type": "custom_http",
  "name": "My API Image Gen",
  "output_type": "image",
  "workflow": {
    "base_url": "https://api.example.com",
    "path": "/v1/generate",
    "method": "POST",
    "headers": {
      "Authorization": "Bearer ${API_KEY}"
    },
    "body_format": "json",
    "response_path": "data.image_url"
  },
  "inputs": [
    {
      "name": "prompt",
      "label": "Prompt",
      "input_type": "text",
      "required": true,
      "bind": { "json_path": "prompt" }
    },
    {
      "name": "seed",
      "label": "Seed",
      "input_type": "integer",
      "bind": { "json_path": "seed" }
    }
  ],
  "output": {
    "download": true,
    "url_path": "data.image_url",
    "bytes_path": null
  }
}
```

### Custom HTTP bindings

- `json_path`: Dot-path to place the input in the request body.
- `response_path` or `url_path`: Where to find the result in the response.

This keeps the manifest extensible without assuming ComfyUI.

## Example: ComfyUI Manifest for `sdxl_simple_example_API.json`

```json
{
  "schema_version": 1,
  "adapter_type": "comfyui",
  "name": "SDXL Simple (Example)",
  "output_type": "image",
  "workflow": {
    "workflow_path": "workflows/sdxl_simple_example_API.json",
    "workflow_hash": "sha256:REPLACE_ME"
  },
  "inputs": [
    {
      "name": "prompt",
      "label": "Prompt",
      "input_type": "text",
      "required": true,
      "ui": { "multiline": true, "group": "Prompt" },
      "bind": {
        "selector": {
          "tag": "prompt_text",
          "class_type": "CLIPTextEncode",
          "input_key": "text",
          "title": "CLIP Text Encode (Prompt)"
        }
      }
    },
    {
      "name": "negative_prompt",
      "label": "Negative Prompt",
      "input_type": "text",
      "ui": { "multiline": true, "group": "Prompt", "advanced": true },
      "bind": {
        "selector": {
          "tag": "negative_text",
          "class_type": "CLIPTextEncode",
          "input_key": "text",
          "title": "CLIP Text Encode (Prompt)"
        }
      }
    },
    {
      "name": "steps",
      "label": "Steps",
      "input_type": "integer",
      "default": 20,
      "ui": { "min": 1, "max": 100, "step": 1, "group": "Sampling" },
      "bind": {
        "selector": {
          "tag": "sampler_steps",
          "class_type": "KSamplerAdvanced",
          "input_key": "steps",
          "title": "KSampler (Advanced) - BASE"
        }
      }
    },
    {
      "name": "cfg",
      "label": "CFG",
      "input_type": "number",
      "default": 5.0,
      "ui": { "min": 1.0, "max": 20.0, "step": 0.5, "group": "Sampling" },
      "bind": {
        "selector": {
          "tag": "sampler_cfg",
          "class_type": "KSamplerAdvanced",
          "input_key": "cfg",
          "title": "KSampler (Advanced) - BASE"
        }
      }
    },
    {
      "name": "width",
      "label": "Width",
      "input_type": "integer",
      "default": 1024,
      "ui": { "min": 64, "max": 2048, "step": 64, "unit": "px", "group": "Size" },
      "bind": {
        "selector": {
          "tag": "latent_width",
          "class_type": "EmptyLatentImage",
          "input_key": "width",
          "title": "Empty Latent Image"
        }
      }
    },
    {
      "name": "height",
      "label": "Height",
      "input_type": "integer",
      "default": 1024,
      "ui": { "min": 64, "max": 2048, "step": 64, "unit": "px", "group": "Size" },
      "bind": {
        "selector": {
          "tag": "latent_height",
          "class_type": "EmptyLatentImage",
          "input_key": "height",
          "title": "Empty Latent Image"
        }
      }
    },
    {
      "name": "checkpoint",
      "label": "Checkpoint",
      "input_type": "text",
      "ui": { "group": "Model", "advanced": true },
      "bind": {
        "selector": {
          "tag": "base_checkpoint",
          "class_type": "CheckpointLoaderSimple",
          "input_key": "ckpt_name",
          "title": "Load Checkpoint - BASE"
        }
      }
    }
  ],
  "output": {
    "selector": {
      "tag": "final_output",
      "class_type": "PreviewImage",
      "input_key": "images",
      "title": "Preview Image"
    },
    "index": 0
  }
}
```

This example mirrors the ComfyUI workflow in `workflows/sdxl_simple_example_API.json`
and exposes a minimal, curated UI.

An example file is included at `workflows/sdxl_simple_example_manifest.json`.

# Provider Setup Guide (MVP + Planned)

This guide covers the current MVP setup and the planned Provider Builder flow.
ComfyUI is the primary open-source path today; other adapter styles are planned.

## Provider Types (Roadmap)

- **ComfyUI (current MVP)**: Local workflows (API JSON).
- **Custom HTTP (planned)**: Generic REST APIs with input mapping.
- **Hosted Adapters (planned)**: fal.ai, Replicate, Veo, etc.

## Quick Start (Current MVP - ComfyUI JSON)

1. Start ComfyUI and confirm it responds at `http://127.0.0.1:8188`.
2. Export an API workflow JSON from ComfyUI.
3. Put the JSON somewhere stable (recommended: `workflows/` in this repo).
4. In the app, open `Settings > AI Providers...` and click `New`.
5. Edit the JSON to point at your `base_url` and `workflow_path`, then `Save`.
6. Select that provider on a generative image clip and click **Generate**.

## Quick Start (Builder - ComfyUI)

The Provider Builder UI is now available for ComfyUI workflows:

1. `Settings > AI Providers...` -> **Build**.
2. Pick a ComfyUI API workflow JSON file.
3. Use search + dropdowns to select node inputs to expose.
4. Choose a single output node (image/video/audio).
5. Save: the builder writes a manifest and creates the provider entry.

No node ID editing required. The builder uses selectors/tags under the hood.

## Where Provider Files Live

Provider entries are stored globally (not per project) as JSON files:

```
%LOCALAPPDATA%\NLA-AI-VideoCreator\providers\
```

Each provider file is named after its UUID: `<provider-id>.json`.

Workflow manifests live alongside the workflow JSON:

```
workflows/
├── my_workflow_API.json
└── my_workflow_manifest.json
```

## Creating a Provider Entry

Open the Providers dialog:

- `Settings > AI Providers...`
- Click `New` to create a draft provider JSON (manual).
- Click **Build** to use the builder UI (recommended).

### Provider JSON Example (ComfyUI Image Gen - MVP)

```json
{
  "id": "d7c1f4a0-9db8-4d7e-a4e8-7b7e0c5a9c21",
  "name": "ComfyUI SDXL (Local)",
  "output_type": "image",
  "inputs": [
    { "name": "prompt", "label": "Prompt", "input_type": "text", "required": true },
    { "name": "negative_prompt", "label": "Negative Prompt", "input_type": "text" },
    { "name": "seed", "label": "Seed", "input_type": "integer" },
    { "name": "steps", "label": "Steps", "input_type": "integer", "default": 20 },
    { "name": "cfg", "label": "CFG", "input_type": "number", "default": 5.0 },
    { "name": "width", "label": "Width", "input_type": "integer", "default": 1024 },
    { "name": "height", "label": "Height", "input_type": "integer", "default": 1024 },
    { "name": "checkpoint", "label": "Checkpoint", "input_type": "text" },
    { "name": "sampler", "label": "Sampler", "input_type": "text" },
    { "name": "scheduler", "label": "Scheduler", "input_type": "text" },
    { "name": "start_step", "label": "Start Step", "input_type": "integer", "default": 0 }
  ],
  "connection": {
    "type": "comfy_ui",
    "base_url": "http://127.0.0.1:8188",
    "workflow_path": "workflows/sdxl_simple_example_API.json",
    "manifest_path": "workflows/sdxl_simple_example_manifest.json"
  }
}
```

### Field Notes

- `id`: Stable UUID for this provider. Keep it the same once assets depend on it.
- `output_type`: `image`, `video`, or `audio`. ComfyUI image workflows use `image`.
- `inputs`: Drives the Attributes panel UI. Required fields must be filled before Generate.
- `connection.type`: Use `comfy_ui` for the current MVP. Other adapters are planned.
- `workflow_path`: Optional. If omitted, the app uses the default
  `workflows/sdxl_simple_example_API.json`.
- `manifest_path`: Optional but recommended. When provided, the adapter binds
  inputs/outputs via selectors instead of legacy node IDs.

## ComfyUI Workflow Setup

The app expects a **ComfyUI API workflow JSON** (not a PNG or UI save).

Recommended flow:

1. Open ComfyUI.
2. Load `workflows/sdxl_simple_example_API.json`.
3. Make your edits (swap model, sampler, etc.).
4. Export as **API** JSON and save over your file.

This preserves the workflow structure and node titles that selector matching
uses (tags are optional but recommended for stability).

### Input Mapping (Builder / Manifest)

The ComfyUI adapter now reads the **manifest** (if present) and binds inputs by
selector instead of node ID. Each exposed input maps to:

```
selector: { tag?, class_type, input_key, title? }
```

Selector matching behavior:

- `tag` (if present) must match `_meta.nla_tag` inside the workflow JSON.
- `class_type + input_key` must match a node input.
- `title` is used to disambiguate when multiple nodes match.

If you don't provide a manifest (or omit `manifest_path` in the provider entry),
the adapter falls back to the legacy node-ID bindings in the SDXL example.

### Output Expectations

- With a manifest: the output selector identifies the node and the output key.
- Without a manifest: the adapter looks for an image output on node `53`
  (PreviewImage) and falls back to the first image output it can find.
- Only the first image output (or the `index` if specified) is used.

### Builder Binding (Current)

The builder lets you bind inputs by **selector** (class type + input key + optional tag),
so node IDs are no longer required. See `docs/PROVIDER_MANIFEST_SCHEMA.md`.
Tags are optional and not exposed in the builder UI yet (TODO: auto-tagging).

## Using Your Provider in the App

1. Create a **Generative Image** asset.
2. Drag it onto a Video track.
3. Select the clip to open the Attributes panel.
4. Pick your provider from the dropdown.
5. Fill in inputs and click **Generate**.

## Pitfalls and Current Constraints

- **Asset inputs are not wired yet.** Image/video/audio inputs show a placeholder.
- **ComfyUI only (for now).** Other adapter types are planned.
- **Relative workflow paths** are resolved from the app working directory first,
  then from the executable directory. Use absolute paths if in doubt.
- **Provider ID changes** will break existing generative assets that reference it.
- **Manual JSON without a manifest** uses the legacy node ID bindings.
- **Manifest-based binding** requires selector matches; mismatches will error.

## Troubleshooting

- "Missing inputs: ..." -> Required fields are not set in the Attributes panel.
- "No workflow node matched selector (...)" -> The manifest selector doesn't
  match your workflow. Check `_meta.nla_tag`, class type, input key, and title.
- "Multiple workflow nodes matched selector (...)" -> Add a tag or title to
  narrow the match.
- "ComfyUI rejected prompt ..." -> Base URL is wrong or ComfyUI is not running.
- "Timed out waiting for ComfyUI output." -> Workflow stalled or produced no image.
- "ComfyUI history did not include image outputs." -> Ensure your workflow
  ends with an image output node.

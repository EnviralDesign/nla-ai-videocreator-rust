# Provider Setup Guide (MVP)

This guide is written for first-time users who want to plug in their own
providers (ComfyUI workflows today, API providers later) and start generating
content inside NLA AI Video Creator.

## Quick Start (ComfyUI)

1. Start ComfyUI locally and confirm it responds at `http://127.0.0.1:8188`.
2. Export an API workflow JSON from ComfyUI.
3. Place the JSON somewhere stable (recommended: `workflows/` in this repo).
4. In the app, open `Settings > AI Providers...` and create a new provider.
5. Select that provider on a generative image clip and click Generate.

## Where Provider Files Live

Provider entries are stored globally (not per project) as JSON files:

```
%LOCALAPPDATA%\NLA-AI-VideoCreator\providers\
```

Each provider file is named after its UUID: `<provider-id>.json`.

## Creating a Provider Entry

Open the Providers dialog:

- `Settings > AI Providers...`
- Click `New` to create a draft provider JSON.
- Edit and save the JSON in the right-side editor.

### Provider JSON Example (ComfyUI Image Gen)

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
    "workflow_path": "workflows/sdxl_simple_example_API.json"
  }
}
```

### Field Notes

- `id`: Stable UUID for this provider. Keep it the same once assets depend on it.
- `output_type`: `image`, `video`, or `audio`. ComfyUI image workflows use `image`.
- `inputs`: Drives the Attributes panel UI. Required fields must be filled before Generate.
- `connection.type`: Use `comfy_ui` for the current MVP. `custom_http` exists but is not wired yet.
- `workflow_path`: Optional. If omitted, the app uses the default
  `workflows/sdxl_simple_example_API.json`.

## ComfyUI Workflow Setup

The app expects a **ComfyUI API workflow JSON** (not a PNG or UI save).

Recommended flow:

1. Open ComfyUI.
2. Load `workflows/sdxl_simple_example_API.json`.
3. Make your edits (swap model, sampler, etc.).
4. Export as **API** JSON and save over your file.

This preserves node IDs that the current adapter expects.

### Input Mapping (Important)

The current ComfyUI adapter is hardwired to specific node IDs and input keys.
Your workflow must contain these nodes (or the adapter will error).

```
prompt          -> node 6  : inputs.text
negative_prompt -> node 7  : inputs.text
seed            -> node 10 : inputs.noise_seed
steps           -> node 10 : inputs.steps
cfg             -> node 10 : inputs.cfg
width           -> node 5  : inputs.width
height          -> node 5  : inputs.height
checkpoint      -> node 4  : inputs.ckpt_name
sampler         -> node 10 : inputs.sampler_name
scheduler       -> node 10 : inputs.scheduler
start_step      -> node 68 : inputs.value
```

If you delete or replace these nodes, update your workflow by reusing the
template or be prepared to change the adapter code.

### Output Expectations

- The adapter looks for an image output on node `53` (PreviewImage).
- If node `53` is missing, it falls back to the first image output it can find.
- Only the first image output is used.

## Using Your Provider in the App

1. Create a **Generative Image** asset.
2. Drag it onto a Video track.
3. Select the clip to open the Attributes panel.
4. Pick your provider from the dropdown.
5. Fill in inputs and click **Generate**.

## Pitfalls and Current Constraints

- **Asset inputs are not wired yet.** Image/video/audio inputs show a placeholder.
- **ComfyUI only.** `custom_http` providers will return "not supported yet."
- **Relative workflow paths** are resolved from the app working directory first,
  then from the executable directory. Use absolute paths if in doubt.
- **Provider ID changes** will break existing generative assets that reference it.

## Troubleshooting

- "Missing inputs: ..." -> Required fields are not set in the Attributes panel.
- "Workflow missing node ..." -> Your API workflow JSON does not contain the
  expected node IDs listed above.
- "ComfyUI rejected prompt ..." -> Base URL is wrong or ComfyUI is not running.
- "Timed out waiting for ComfyUI output." -> Workflow stalled or produced no image.
- "ComfyUI history did not include image outputs." -> Ensure your workflow
  ends with an image output node.

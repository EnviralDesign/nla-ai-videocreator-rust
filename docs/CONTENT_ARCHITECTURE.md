# Content & Generation Architecture

> **The philosophy and design decisions for how content flows through NLA AI Video Creator.**

This document captures the architecture for:
- How assets and generative objects are represented
- How the timeline relates to generation inputs
- How providers declare their capabilities
- The user workflow for building AI-generated video projects

---

## 🎯 Core Philosophy

### Familiar Where Expected, Innovative Where There's Vacuum

The timeline and clip representation follows **established NLE conventions** (DaVinci, Premiere, etc.). Users shouldn't have to relearn how a timeline works.

The **generative layer** is where we innovate. This is new territory with no established patterns, so we design for:
- Low friction
- Logical defaults
- Smart automation that doesn't require explicit wiring

### The Timeline as Implicit Wiring

When configuring a generative asset's inputs, the system **auto-surfaces assets that overlap in time**. The timeline itself becomes the wiring diagram—if you want an image to be the input for a generative video, place them overlapping temporally. No explicit linking required.

---

## 📦 Asset Types

All content in a project is an **Asset**. Assets live in the project folder and are managed via the Assets panel.

### Standard Assets

| Type | How Created | File Location |
|------|-------------|---------------|
| **Video** | Import file (drag or menu) | `video/` |
| **Image** | Import file (drag or menu) | `images/` |
| **Audio** | Import file (drag or menu) | `audio/` |

Standard assets are simple file references. They have no generation history or inputs.

### Generative Assets

| Type | How Created | File Location |
|------|-------------|---------------|
| **Generative Video** | Explicit UI action ("+ New Generative Video") | `generated/video/{id}/` |
| **Generative Image** | Explicit UI action ("+ New Generative Image") | `generated/image/{id}/` |
| **Generative Audio** | Explicit UI action ("+ New Generative Audio") | `generated/audio/{id}/` |

Generative assets are **explicit, intentional creations**. They start "hollow" (no media) and become populated through generation.

UI note: generative assets use human-friendly sequential names (e.g., "Gen Image 1") while their folders remain UUID-based.

#### Generative Asset Structure

Each generative asset has its own folder:

```
generated/video/gen_001/
├── config.json      # Generation parameters, provider, inputs, history
├── v1.mp4           # First generation
├── v2.mp4           # Second generation (different seed, etc.)
└── v3.mp4           # ...
```

The `config.json` stores:
- Provider ID
- Input bindings (references to other assets)
- Provider-specific parameters (prompt, seed, etc.)
- Generation history (timestamps, which inputs were used)

#### Active Version

Each generative asset has an **active version**—the one currently displayed on the timeline and used when this asset is an input to another generative asset.

When a generative video references a generative image:
1. It binds to the image asset (not a specific version)
2. At generation time, it resolves to the image's **current active version**
3. If the user switches the image to a different version and regenerates the video, the video automatically uses the new image

This creates a **reactive cascade** without explicit re-linking.

---

## 🎬 Timeline Representation

### Track Types

| Track Type | Holds | Default Count |
|------------|-------|---------------|
| **Video** | Video clips, image clips (stills with duration), generative video/image clips | 1 ("Video 1") |
| **Audio** | Audio clips, generative audio clips | 1 ("Audio 1") |
| **Markers** | Point-in-time markers (no duration) | 1 (non-duplicatable) |

Users can add additional Video and Audio tracks. Markers is a single, special-purpose track.

### Clip Representation

All clips on Video/Audio tracks are **range-based** (have start time and duration). This includes:
- Standard video/audio clips
- Still images (displayed for their duration)
- Generative assets (sized to desired output duration)

**There is no "point-based" visual on video/audio tracks.** Everything has temporal extent.

### Markers

Markers are **point-based** annotations:
- Single point in time (no duration)
- Can be stacked (multiple markers at same time)
- Have optional label and metadata
- Used for: beat markers, scene breaks, notes, cue points

Markers are organizational aids, not content.

### Generative Clips on Timeline

When a generative asset is placed on the timeline:
- **Before generation:** Shows placeholder (dashed border, "⚙️ Pending" indicator)
- **After generation:** Shows thumbnail/preview of active version
- **Multiple versions:** User selects active version via Attributes panel

The clip's duration on the timeline serves as the **target duration** for generation (for providers that support duration control).

---

## 🔌 Provider System (Revised)

### What Users Need

1. Build and test **any** ComfyUI workflow in isolation.
2. Attach that workflow as a provider without hand-editing JSON.
3. Expose only a **small, curated set** of inputs in the editor UI.
4. Keep providers stable even if ComfyUI **node IDs change**.

### Provider Entry (Global)

A **Provider Entry** is the user-facing configuration stored in the app. It holds:
- **Output Type**: What it produces (Image | Video | Audio)
- **Connection Info**: How to reach it (ComfyUI URL, API key, etc.)
- **Workflow Package**: A workflow JSON plus a manifest that declares what to expose

Provider entries stay global and are referenced by `provider_id` in each generative asset's `config.json`.

### Workflow Package (ComfyUI)

For ComfyUI, a provider points at two files:

```
workflows/
├── my_workflow_API.json        # The ComfyUI API workflow
└── my_workflow_manifest.json   # The provider manifest (generated by the app)
```

The manifest is the bridge between "anything in ComfyUI" and "a clean UI in NLA."

### Provider Manifest (Proposed)

The manifest lists only the inputs you want to expose and how each input maps
into the workflow. Node IDs are not the source of truth.

```json
{
  "workflow": "workflows/sdxl_simple_example_API.json",
  "output": {
    "type": "image",
    "selector": { "class_type": "PreviewImage" },
    "index": 0
  },
  "inputs": [
    {
      "name": "prompt",
      "label": "Prompt",
      "input_type": "text",
      "required": true,
      "bind": {
        "selector": {
          "tag": "prompt_text",
          "class_type": "CLIPTextEncode",
          "input_key": "text"
        }
      }
    },
    {
      "name": "negative_prompt",
      "label": "Negative Prompt",
      "input_type": "text",
      "bind": {
        "selector": {
          "tag": "negative_text",
          "class_type": "CLIPTextEncode",
          "input_key": "text"
        }
      }
    },
    {
      "name": "steps",
      "label": "Steps",
      "input_type": "integer",
      "default": 20,
      "ui": { "min": 1, "max": 100, "step": 1 },
      "bind": {
        "selector": {
          "class_type": "KSamplerAdvanced",
          "title": "KSampler (Advanced) - BASE",
          "input_key": "steps"
        }
      }
    },
    {
      "name": "width",
      "label": "Width",
      "input_type": "integer",
      "default": 1024,
      "ui": { "min": 64, "max": 2048, "step": 64 },
      "bind": {
        "selector": {
          "class_type": "EmptyLatentImage",
          "title": "Empty Latent Image",
          "input_key": "width"
        }
      }
    }
  ]
}
```

Notes:
- The manifest is generated by a **Provider Builder UI** (see below).
- Users can still edit JSON manually, but that should be optional.
- Tags (like `prompt_text`) are written by the builder to disambiguate similar nodes.

### Binding Resolution (No Node IDs)

Bindings resolve workflow inputs using a **selector**, not raw node IDs.
Selectors are intentionally flexible:

- `class_type` (required)
- `input_key` (required)
- `title` (optional, from `_meta.title`)
- `tag` (optional, a stable identifier written into workflow metadata)

Resolution strategy (proposed):
1. Match by `tag` if present (most stable).
2. Otherwise match by `class_type + input_key + title`.
3. If multiple matches or no match, mark the provider "Needs Rebind" and open a UI wizard.

The app may store a **last-seen node ID** for speed, but it is never the source of truth.

### Tagging Nodes (Optional but Recommended)

The Provider Builder can optionally write a stable tag into workflow metadata
(`_meta.nla_tag`) for any exposed input. This avoids brittle title matching and
survives node ID changes. Users never have to hand-edit JSON.

### Comfy Provider Builder UI (Planned)

Instead of editing JSON by hand, users get a guided, ComfyUI-specific flow
that separates **workflow authoring** from **provider setup**.

Core UX elements:

- **Workflow file picker**: Browse and select a ComfyUI API JSON file.
- **Node browser**: Search + filter by `class_type`, title, or input key.
- **Input inspector**: Click a node to view its inputs and outputs.
- **Expose panel**: Add selected inputs to the provider UI with labels,
  defaults, ranges, and required flags.
- **Output selector**: Choose which node output to treat as the provider output.

Provider editor integration:
- The existing Providers modal gains a **Build from Workflow** button.
- Clicking it opens the builder and pre-fills provider name/output type.

Suggested flow:

1. Pick a workflow JSON (API format).
2. The builder parses nodes and builds an indexed list.
3. Select a node input, click **Expose Input**.
4. Name the field, set default/required/range, optionally mark as Advanced.
5. Select an output node for `image`/`video`/`audio` output.
6. Save: the builder writes a manifest and creates the provider entry.

The builder can optionally write `_meta.nla_tag` into selected nodes/inputs to
make bindings stable across workflow edits.

### Node Linking UI (Dropdown + Search)

The binding UI should let users connect exposed fields without ever touching
node IDs:

- **Search bar**: Filter by class type, title, or input key.
- **Node dropdown**: Pick a node from filtered results.
- **Input dropdown**: Pick one of the node's input keys.
- **Conflict indicator**: Warn if multiple nodes match the selector.
- **Resolution hint**: Suggest adding a tag if the selector is ambiguous.

This keeps provider setup as a clean, deliberate step after a workflow is
already working in ComfyUI.

### Workflow Drift & Rebind

Because workflows evolve, the manifest should store a lightweight fingerprint
(hash of node class types + input keys + titles) for change detection. If the
hash changes, the provider is marked **Needs Rebind** and opens the builder
with the previous selections preloaded for quick repair.

### Output Type is Primary

Providers are still grouped by output type. We do not hardcode "I2V" or "T2V";
the input schema **is** the type.

Common patterns:
| Output | Common Input Patterns |
|--------|----------------------|
| **Video** | Image (I2V), Text only (T2V), Video (V2V), Image+Audio |
| **Image** | Text (T2I), Image (I2I) |
| **Audio** | Text (T2A), Video (V2A) |

### Input Types (Exposed Inputs)

| Type | UI Widget | Source |
|------|-----------|--------|
| `image` | Asset picker (dropdown/browse) | Project asset |
| `video` | Asset picker | Project asset |
| `audio` | Asset picker | Project asset |
| `text` | Text area | Literal |
| `number` | Number input / slider | Literal |
| `integer` | Integer input | Literal |
| `boolean` | Checkbox | Literal |
| `enum` | Dropdown | Literal |

### Input UI Metadata (Optional)

Exposed inputs can include UI hints for better controls:

- `min` / `max` / `step` for numeric fields
- `placeholder` and `multiline` for text fields
- `group` and `advanced` to keep the UI clean
- `unit` (e.g., "px", "s") for display

These do not affect the workflow binding, only the editor UI.

### MVP Reality Check

For now, only **literal inputs** are wired in the app. Asset inputs (image/video/audio)
are part of the design but not implemented yet.

Current MVP implementation details:
- Provider editing is a **raw JSON modal** (no builder UI yet).
- ComfyUI bindings are **hardcoded to node IDs** in the adapter.
- Output selection uses a fixed node ID first, then falls back to first image output.

The sections above describe the **intended** architecture once the Provider
Builder and selector-based bindings are implemented.

---

## 🧠 Smart Input Suggestions

Once asset inputs are wired, the **Attributes panel** can auto-populate suggestions
for image/video/audio fields.

### Temporal Overlap Priority

For inputs that reference assets (image, video, audio), the picker shows:

1. **Overlapping Assets** — Assets whose clips overlap the generative clip's time range on the timeline
2. **Other Project Assets** — Everything else in the project

```
┌─────────────────────────────────────────┐
│ Start Image                             │
│ ┌─────────────────────────────────────┐ │
│ │ 📍 In Time Range:                   │ │
│ │    [intro_scene.png] ✓              │ │
│ │    [bg_layer.png]                   │ │
│ │ ─────────────────────────────────── │ │
│ │ 📁 Other Assets:                    │ │
│ │    [outro_scene.png]                │ │
│ │    [unused_ref.png]                 │ │
│ └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

This makes the workflow intuitive:
1. Lay out reference images on a Video track
2. Lay out generative video clips overlapping those images
3. Open a generative clip's Attributes—the overlapping images are right there

### Duration Inference

If the generative provider supports a `duration` input:
- Default to the clip's duration on the timeline
- User can override in Attributes panel

---

## 📁 Project Folder Structure

```
my-project/
├── project.json              # Project state, track layout, clip placements
├── audio/                    # Imported audio files
│   └── soundtrack.mp3
├── images/                   # Imported images
│   └── reference_art.png
├── video/                    # Imported video files
│   └── b_roll_clip.mp4
├── generated/                # All generated content
│   ├── video/
│   │   ├── gen_001/          # One folder per generative video asset
│   │   │   ├── config.json
│   │   │   ├── v1.mp4
│   │   │   ├── v2.mp4
│   │   │   └── ...
│   │   └── gen_002/
│   │       └── ...
│   ├── image/
│   │   └── gen_img_001/
│   │       ├── config.json
│   │       └── v1.png
│   └── audio/
│       └── ...
├── exports/                  # Final rendered outputs
│   └── final_v1.mp4
```

Global providers (MVP, Windows):
```
%LOCALAPPDATA%\NLA-AI-VideoCreator\providers\
├── <provider-id>.json
└── ...
```

Workflow templates (repo):
```
workflows/
└── sdxl_simple_example_API.json
```

Manifests will live alongside workflows once the Provider Builder is in place
(for example: `sdxl_simple_example_manifest.json`).

### In-Project Only (Strict MVP)

For MVP, **all assets MUST be inside the project folder.**

- **No external linking:** Dragging a file into the app **implicitly copies** it into the project folder.
- **No prompt:** The copy happens automatically to ensure project portability.
- **"Project = Folder":** The project is fully self-contained. You can zip the folder and send it to another machine.

---

## 🔄 Generation Workflow

### Creating a Generative Asset

1. **User action:** "+ New Generative Video" button in Assets panel
2. **System:** Creates empty generative asset, adds to Assets list
3. **User:** Drags asset to timeline, sizes it
4. **User:** Selects clip, opens Attributes panel
5. **System:** Shows provider picker, input fields (with overlapping assets promoted)
6. **User:** Configures inputs, clicks "Generate"
7. **System:** Queues generation job, shows progress
8. **On complete:** Asset folder now contains `v1.mp4`, clip thumbnail updates

### Regeneration

1. **User:** Selects existing generative clip
2. **User:** Tweaks inputs (different prompt, seed, etc.)
3. **User:** Clicks "Generate" (or "Add Variation")
4. **System:** Generates `v2.mp4`, adds to versions list
5. **User:** Can switch active version in Attributes panel

### Batch Variations

For future consideration:
- "Generate 5 variations" queues multiple jobs with different seeds
- Results populate as they complete
- User picks favorite as active

---

## 🎨 UI Implications

### Assets Panel

Shows all project assets:
- Imported assets (simple list/grid)
- Generative assets (distinguished visually—maybe a ⚙️ badge)

Actions:
- Import file
- + New Generative Video
- + New Generative Image
- + New Generative Audio

### Attributes Panel (for Generative Clips)

When a generative clip is selected:
- **Provider selection** (dropdown of compatible providers)
- **Input fields** (dynamic, based on provider schema)
- **Version selector** (if multiple versions exist)
- **Generate / Regenerate button**
- **Generation status** (pending, in progress, complete)

### Timeline Clips

Visual distinction:
- **Standard clips:** Solid, show content thumbnail
- **Generative clips (pending):** Dashed border, placeholder icon
- **Generative clips (generated):** Solid, show thumbnail, maybe subtle badge indicating it's generative

---

## 🗂️ Data Model Sketch

```rust
// Asset types
enum AssetKind {
    Video { path: PathBuf },
    Image { path: PathBuf },
    Audio { path: PathBuf },
    GenerativeVideo { id: Uuid, folder: PathBuf, active_version: Option<String> },
    GenerativeImage { id: Uuid, folder: PathBuf, active_version: Option<String> },
    GenerativeAudio { id: Uuid, folder: PathBuf, active_version: Option<String> },
}

struct Asset {
    id: Uuid,
    name: String, // User-facing label
    kind: AssetKind,
}

// Track types
enum TrackType { Video, Audio, Marker }

struct Track {
    id: Uuid,
    name: String,
    track_type: TrackType,
}

// Clips (range-based, on Video/Audio tracks)
struct Clip {
    id: Uuid,
    asset_id: Uuid,      // Reference to asset
    track_id: Uuid,
    start_time: f64,     // Seconds
    duration: f64,       // Seconds
    // trim_in, trim_out for future
}

// Markers (point-based, on Marker track)
struct Marker {
    id: Uuid,
    time: f64,           // Seconds
    label: Option<String>,
    color: Option<String>,
}

// Provider
enum OutputType { Video, Image, Audio }
enum ProviderConnection { ComfyUi { base_url: String }, CustomHttp { base_url: String } }

struct ProviderEntry {
    id: Uuid,
    name: String,
    output_type: OutputType,
    connection: ProviderConnection,
    workflow: WorkflowPackage,
}

struct WorkflowPackage {
    workflow_path: PathBuf,
    manifest_path: PathBuf,
}

struct ProviderManifest {
    workflow: PathBuf,
    workflow_hash: Option<String>,
    output: OutputSelector,
    inputs: Vec<ManifestInput>,
}

struct ManifestInput {
    name: String,
    label: String,
    input_type: InputType, // Image, Video, Audio, Text, Number, etc.
    required: bool,
    default: Option<Value>,
    ui: Option<InputUi>,
    bind: InputBinding,
}

struct InputUi {
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    placeholder: Option<String>,
    multiline: bool,
    group: Option<String>,
    advanced: bool,
    unit: Option<String>,
}

struct InputBinding {
    selector: NodeSelector,
    transform: Option<BindingTransform>,
}

struct NodeSelector {
    tag: Option<String>,
    class_type: String,
    input_key: String,
    title: Option<String>,
}

struct OutputSelector {
    output_type: OutputType,
    selector: NodeSelector,
    index: Option<u32>,
}

enum BindingTransform {
    Clamp { min: f64, max: f64 },
    Scale { factor: f64 },
}

// Generative config (stored in config.json)
struct GenerativeConfig {
    provider_id: Uuid,
    inputs: HashMap<String, InputValue>, // name -> bound value or asset ref
    versions: Vec<GenerationRecord>,
    active_version: Option<String>,
}

enum InputValue {
    AssetRef(Uuid),           // Reference to another asset
    Literal(serde_json::Value), // Inline value (text, number, etc.)
}

struct GenerationRecord {
    version: String,          // "v1", "v2", etc.
    timestamp: DateTime<Utc>,
    provider_id: Uuid,
    inputs_snapshot: HashMap<String, InputValue>,
}
```

---

## 📋 Decision Summary

| Topic | Decision |
|-------|----------|
| Timeline representation | Standard clips with duration (familiar NLE model) |
| Images on timeline | Treated as stills with duration, on Video tracks |
| Generative assets | Explicit creation, version history, active version |
| Input wiring | Timeline overlap = smart suggestions, no explicit linking |
| Provider taxonomy | Grouped by output type; input schema is dynamic |
| Provider bindings | Resolve via selectors/tags, not node IDs |
| Provider builder UI | Workflow picker + node browser for binding exposed inputs |
| Track types | Video, Audio, Markers |
| Track duplication | Video/Audio can be duplicated; Markers is singular |
| Cascading versions | Dependent generative assets use active version of inputs |
| Asset storage | In-project only (MVP) |
| Folder structure | Generative assets get their own folder with versions |

---

*Last updated: 2026-01-08*

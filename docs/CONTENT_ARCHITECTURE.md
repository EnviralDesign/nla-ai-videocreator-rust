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
| **Generative Image** | Explicit UI action ("+ New Generative Image") | `generated/images/{id}/` |
| **Generative Audio** | Explicit UI action ("+ New Generative Audio") | `generated/audio/{id}/` |

Generative assets are **explicit, intentional creations**. They start "hollow" (no media) and become populated through generation.

#### Generative Asset Structure

Each generative asset has its own folder:

```
generated/video/gen_001/
├── config.json      # Generation parameters, provider, inputs
├── v1.mp4           # First generation
├── v2.mp4           # Second generation (different seed, etc.)
├── v3.mp4           # ...
└── active.txt       # Points to active version (e.g., "v2")
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

## 🔌 Provider System

### Provider Entries

A **Provider Entry** is a configured backend that can execute generation tasks. Users add entries via the Provider configuration UI.

Each entry declares:
- **Output Type**: What it produces (Video | Image | Audio)
- **Input Schema**: What inputs it requires (dynamic per provider)
- **Connection Info**: How to reach it (ComfyUI URL, API key, etc.)

### Output Type is Primary

Providers are grouped by **output type**. When creating a "Generative Video," the user picks from providers that output video. The input requirements vary per provider—that's determined by the schema.

Common patterns:
| Output | Common Input Patterns |
|--------|----------------------|
| **Video** | Image (I2V), Text only (T2V), Video (V2V), Image+Audio, etc. |
| **Image** | Text (T2I), Image (I2I), etc. |
| **Audio** | Text (T2A), Video (V2A), etc. |

We don't hardcode "I2V" or "T2V" as types—the input schema **is** the type.

### Input Schema

Each provider declares its inputs as a schema:

```json
{
  "inputs": [
    { "name": "start_image", "type": "image", "required": true, "label": "Start Frame" },
    { "name": "end_image", "type": "image", "required": false, "label": "End Frame" },
    { "name": "prompt", "type": "text", "required": true, "label": "Prompt" },
    { "name": "duration", "type": "number", "required": false, "default": 5, "label": "Duration (s)" },
    { "name": "seed", "type": "integer", "required": false, "label": "Seed" }
  ]
}
```

The Attributes panel dynamically renders input widgets based on this schema.

### Input Types

| Type | UI Widget | Asset Reference |
|------|-----------|-----------------|
| `image` | Asset picker (dropdown/browse) | References an image asset |
| `video` | Asset picker | References a video asset |
| `audio` | Asset picker | References an audio asset |
| `text` | Text area | Inline value |
| `number` | Number input / slider | Inline value |
| `integer` | Integer input | Inline value |
| `boolean` | Checkbox | Inline value |
| `enum` | Dropdown | Inline value |

---

## 🧠 Smart Input Suggestions

When configuring a generative asset's inputs, the **Attributes panel** auto-populates suggestions.

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
│   │   │   └── active.txt
│   │   └── gen_002/
│   │       └── ...
│   ├── images/
│   │   └── gen_img_001/
│   │       ├── config.json
│   │       ├── v1.png
│   │       └── active.txt
│   └── audio/
│       └── ...
├── exports/                  # Final rendered outputs
│   └── final_v1.mp4
└── .providers/               # Provider configurations for this project
    └── my_comfy_t2v.json
```

### In-Project Only (MVP)

For MVP, all assets must be inside the project folder. External file references are a future power-user feature.

When a user selects a file from outside the project, prompt:
> "Copy this file into your project folder?"

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

struct ProviderEntry {
    id: Uuid,
    name: String,
    output_type: OutputType,
    inputs: Vec<InputField>,
    // connection details...
}

struct InputField {
    name: String,
    label: String,
    input_type: InputType, // Image, Video, Audio, Text, Number, etc.
    required: bool,
    default: Option<Value>,
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
| Track types | Video, Audio, Markers |
| Track duplication | Video/Audio can be duplicated; Markers is singular |
| Cascading versions | Dependent generative assets use active version of inputs |
| Asset storage | In-project only (MVP) |
| Folder structure | Generative assets get their own folder with versions |

---

*Last updated: 2026-01-04*

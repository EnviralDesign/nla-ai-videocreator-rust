# NLA AI Video Creator

> **A local-first, AI-native Non-Linear Animation editor for generative video production.**

---

## 🎯 Vision

**NLA AI Video Creator** is a desktop application designed to bridge the gap between creative intent and AI-powered video generation. It provides filmmakers, animators, and content creators with an intuitive timeline-based environment to orchestrate AI-generated content—keyframe images, video segments, and audio—into cohesive short films.

The tool embraces a **"bring your own AI"** philosophy. Rather than locking users into a single provider or workflow, it offers a modular adapter architecture that lets creators plug in their preferred tools—whether that's commercial APIs like Veo3 and fal.ai, or custom ComfyUI workflows they've painstakingly crafted.

### The Problem

Creating AI-generated short films today is *tedious*:
- Switching between generation tools and video editors
- Manually downloading, renaming, and importing assets
- Losing creative flow while waiting for generations
- No unified timeline view of audio + keyframes + generated segments
- Difficulty coordinating keyframe images with beat markers in music

### The Solution

A purpose-built NLA editor that:
1. **Unifies the workflow** — Audio, keyframes, and video segments live in one timeline
2. **Integrates AI natively** — Generate images and videos directly from the editor
3. **Stays flexible** — Swap providers per-project or per-shot via adapters
4. **Works locally** — Your projects, your machine, your data (with optional cloud features later)

---

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         NLA AI Video Creator                        │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │
│  │   Timeline  │  │   Preview   │  │   Assets    │  │ Attribute  │  │
│  │    Editor   │  │    Window   │  │   Browser   │  │   Editor   │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│                         Core Engine (Rust)                          │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  App State  │  Selection  │  Asset Manager  │  Job Queue     │   │
│  └──────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│                      Provider Adapter Layer                         │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌─────────────────┐   │
│  │  ComfyUI   │ │   fal.ai   │ │   Veo3     │ │  Custom HTTP    │   │
│  │  Adapter   │ │  Adapter   │ │  Adapter   │ │    Adapter      │   │
│  └────────────┘ └────────────┘ └────────────┘ └─────────────────┘   │
│  └────────────┘ └────────────┘ └────────────┘ └─────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│                       Rendering Engine                              │
│  ┌───────────────┐   ┌────────────────┐   ┌─────────────────────┐   │
│  │ Thumbnailer   │   │  Compositor    │   │   Frame Server      │   │
│  └───────────────┘   └────────────────┘   └─────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **UI Framework** | [Dioxus 0.7](https://dioxuslabs.com/) | Rust-native, cross-platform, reactive, hot-patching |
| **Language** | Rust | Safety, performance, excellent FFI |
| **Video Processing** | FFmpeg (external) | Industry standard, battle-tested |
| **Async Runtime** | Tokio | De facto Rust async runtime |
| **Serialization** | Serde (JSON) | Provider configs, project files |
| **HTTP Client** | reqwest | Async HTTP for API providers |

### Target Platforms

| Platform | Priority | Status |
|----------|----------|--------|
| Windows 10/11 | **Primary** | Active development |
| macOS | Secondary | Future |
| Linux | Secondary | Future |

---

## 📐 Core Concepts

### 1. Project

A **Project** is the top-level container. It has:
- A name and save location (folder = project, KISS)
- Global settings (resolution, frame rate, export preferences)
- One or more **Tracks**
- Provider configuration (global for MVP)

### 2. App Settings

Global application settings (not per-project):
- **Projects folder location** — Where new projects are created by default
- UI preferences (theme, layout)
- Default providers / presets
- FFmpeg path override (if not on PATH)

### 3. Tracks

The timeline consists of layered tracks:

| Track Type | Purpose | Duplicatable |
|------------|---------|--------------|
| **Video Track** | Holds video clips, image clips (stills with duration), and generative visual content | Yes |
| **Audio Track** | Holds audio clips and generative audio content | Yes |
| **Marker Track** | Holds point-in-time markers (beat markers, scene breaks, notes) | No |

> **Note:** Images are placed on Video tracks as stills with duration, following standard NLE conventions. There is no separate "Keyframe" track—reference images for generation are simply clips that overlap generative clips in time.
> 
> See [CONTENT_ARCHITECTURE.md](./CONTENT_ARCHITECTURE.md) for the full content and generation architecture.

### 4. Markers / Keypoints

Markers are timestamp annotations that can:
- Be placed manually (MVP)
- Be auto-generated from audio analysis (beat detection, transients)
- Carry metadata (labels, colors, types)
- Trigger or guide generation tasks

### 5. Generation Tasks

A **Generation Task** is a request to an AI provider:
- **Image Generation** — Create a keyframe image from a prompt
- **Image-to-Video (I2V)** — Animate a keyframe into a video segment
- **Video-to-Video (V2V)** — Transform/stylize an existing video
- **Video Extension** — Extend a video segment forward or backward
- **Audio Generation** — *(Future)* Generate audio from video, or music from prompts
- **Audio Analysis** — *(Future)* Extract beats, segments, transcription

> Note: Audio isn't just an anchor—it might itself be generated content.

### 6. Provider Entries

Provider entries are the pluggable backends that execute generation tasks. Key principles:
- **Single-purpose** — Each entry does ONE thing (image gen, I2V, etc.). If a service supports multiple capabilities, the user adds separate entries for each.
- Configured via simple JSON/config
- Can be a commercial API, local ComfyUI instance, or custom HTTP endpoint
- ComfyUI entries reference an API workflow JSON via `workflow_path` (relative to repo/app or absolute path; MVP default: `workflows/sdxl_simple_example_API.json`)
- Details of the adapter interface will be discovered during implementation—we're keeping this intentionally vague until we experiment with real ComfyUI workflows.

---

> **Intentionally vague for now.** We'll discover the right abstractions when we actually integrate with ComfyUI and experiment with real workflows. Premature abstraction is the enemy of good design.

---

## 🎥 Rendering & Preview Strategy

### 1. Robust Compositor (Canvas-based)
We are skipping intermediate "DOM overlay" solutions to build a production-grade compositing engine from the start. This ensures support for pixel-perfect operations, affine transformations (scale, rotate, translate), and complex blending.

#### Architecture
1.  **Frame Server (Rust)**
    - Managed by a background thread (Tokio).
    - Responsible for fetching or decoding frame data for the current timestamp.
    - **Caching Strategy**: To ensure smooth scrubbing, we will employ a hybrid strategy:
        - **Images**: Loaded fast from disk/memory.
        - **Video**: decoded on demand or pre-cached as low-res proxy image sequences for performance.
2.  **Compositor (Rust)**
    - Takes raw frame buffers from the Frame Server.
    - Applies a "Render Graph" of operations:
        - **Transform**: Scale, Rotate, Translate (Project Canvas coordinates).
        - **Composite**: Layering using standard blending modes (Source Over).
    - Outputs a single raw RGBA buffer for the viewport.
3.  **Display (Frontend/Dioxus)**
    - A single `<canvas>` element in the Preview Panel.
    - Rust sends the composited RGBA buffer to the shared UI memory or via efficient binary transfer.
    - JavaScript draws the buffer using `putImageData` or WebGL texture upload.

### 2. Thumbnail Generation
Visual feedback on the timeline.
- **Mechanism**: Background FFmpeg task.
- **Output**: Cached JPEGs stored in `.project/cache/thumbnails/`.
- **UI**: CSS `background-image` sprite sheets for performance.

---

## 🔌 Provider System

### Design Goals

1. **Simplicity** — Adding a provider should be straightforward, not overwhelming
2. **Single-purpose entries** — Each provider entry does ONE thing. Want image gen AND I2V from the same service? Add two entries.
3. **Configurability** — JSON-based configuration (API URL, auth key, workflow path, etc.)
4. **Flexibility** — A project can mix providers freely (Provider A for images, Provider B for I2V)

### Entry Types

The types of things a provider entry can do:
- **Image Generation** — Text/prompt → Image
- **Image-to-Video (I2V)** — Image → Video segment
- **Video-to-Video (V2V)** — Video → Transformed video
- **Video Extension** — Video → Longer video
- *(Future: Audio analysis, beat detection, etc.)*

### Implementation Notes

> **Intentionally vague for now.** We'll discover the right abstractions when we actually integrate with ComfyUI and experiment with real workflows. Premature abstraction is the enemy of good design.

### Example Providers (Ideas)

| Provider | Type | Notes |
|----------|------|-------|
| ComfyUI workflow #1 | Image Gen | User's custom SDXL style workflow |
| ComfyUI workflow #2 | I2V | User's AnimateDiff or similar |
| fal.ai Kling | I2V | Commercial API |
| Veo3 | I2V | Commercial API |
| Replicate Model X | Image Gen | Commercial API |

Users can add as many entries as they need. The same underlying service (like ComfyUI) might have multiple entries for different workflows/purposes.

### Dynamic Provider UI (Sockets)

Providers have **bespoke input requirements**. Some need just text, others need text + image, etc. The UI should have a framework that:
- Allows providers to declare their input schema
- Dynamically renders appropriate input fields (text boxes, image pickers, sliders, etc.)
- Acts as "sockets" that can be plugged into from the predefined tools

> **Intentionally vague on implementation.** The goal is to avoid hardcoding provider UIs—instead, a harness that adapts based on what each provider needs.

---

## 🎬 User Workflow (MVP)

### Phase 1: Setup
### Phase 1: Setup
1. User opens the app → **Startup Modal** appears (New Project / Open Project)
2. **New Project**: User selects a folder/name. System immediately creates the folder structure on disk.
3. **Open Project**: User selects an existing `project.json` or folder.
4. App loads with the project active.
2. Sets project dimensions (1080p, 4K, etc.) and frame rate
3. Configures one or more providers in the Provider panel

### Phase 2: Audio & Planning
1. Imports an audio file (MP3/WAV) → appears on Audio Track
2. Plays through audio, manually places markers at key moments
3. Optionally labels markers ("intro", "beat drop", "climax")

### Phase 3: Keyframes
1. At each marker (or selected markers), user creates a keyframe slot
2. Either:
   - **Imports** an existing image
   - **Generates** via a configured image provider (types prompt, clicks "Generate")
3. Keyframe appears in the Keyframe Track at that timestamp

### Phase 4: Video Generation
1. User selects two adjacent keyframes
2. Chooses an I2V provider and parameters
3. Clicks "Generate Video Segment"
4. Generated video appears in the Video Track, spanning between keyframes

### Phase 5: Export
1. User arranges tracks as desired
2. Clicks "Export" → FFmpeg composites audio + video → final output
3. *(Future)* Option to "Export Parts" — individual clips with nice filenames for external editing

---

## 🎨 UI Principles

### Fluidity & Polish

The UI should feel **alive and responsive**:
- **Hover effects** on all interactive elements (buttons, clips, markers)
- **Smooth transitions** on state changes (selections, panel toggles)
- **Timeline smoothness** — scrubbing, zooming, and panning should feel buttery
- No jarring state jumps; prefer animated transitions where practical

### Attribute Editor

A context-sensitive panel that displays properties of the current selection:
- If **one item** is selected → show all editable properties
- If **multiple items of the same type** are selected → show common properties, edits apply to all
- If **mixed types** are selected → show only universally applicable actions (delete, etc.)

This panel adapts based on what's selected in the **timeline** or **asset browser**.

### Labels vs. Filenames

Every asset (clip, keyframe, audio file) has:
- **Filename** — The actual file on disk (auto-generated or imported)
- **Label** — A user-facing display name (optional, can be different from filename)

This supports:
- Friendly display in the UI ("Intro Scene" instead of `seg_001_002.mp4`)
- Future "Export Parts" feature where clips get nice descriptive names

---

## 🗂️ Project File Structure

```
my-project/
├── project.json          # Main project file
├── audio/                # Imported audio files
│   └── soundtrack.mp3
├── keyframes/            # Generated or imported images
│   ├── kf_001_intro.png
│   └── kf_002_beatdrop.png
├── video_segments/       # Generated video clips
│   ├── seg_001_002.mp4
│   └── seg_002_003.mp4
├── exports/              # Final rendered outputs
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

### `project.json` Schema (Simplified)

```json
{
  "version": "1.0",
  "name": "My Short Film",
  "settings": {
    "width": 1920,
    "height": 1080,
    "fps": 24
  },
  "tracks": {
    "audio": [...],
    "markers": [...],
    "keyframes": [...],
    "video": [...]
  },
  "provider_assignments": {
    "image_generation": "my_comfy_workflow",
    "image_to_video": "fal_kling"
  }
}
```

---

## 📦 MVP Feature Set

### Must Have (v0.1)

- [x] **UI Shell** ✓
  - Main application layout (title bar, panels, timeline, status bar)
  - Charcoal monochrome color scheme with functional accent colors
  - Consistent borders and typography
  - Panel headers with matching heights

- [x] **Panel System** ✓
  - Resizable side panels (drag edge, instant feedback)
  - Collapsible side panels (icon button → thin rail)
  - Collapsible timeline (icon button → bottom rail with controls visible)
  - Smooth animated collapse/expand transitions
  - Hover feedback on collapsed rails
  - Click anywhere on collapsed rail/header to expand
  - Drag state persists if mouse leaves window and returns

- [x] **Data Model & Project Management** (Phase 1) ✓
  - [x] Core data structures (Project, Track, Clip, Asset, Marker)
  - [x] Project save/load (JSON serialization)
  - [x] Project creation workflow (new project → folder)
  - [x] Project settings (resolution, fps)
  - [x] In-project asset storage (audio/, images/, video/, generated/)

- [x] **Timeline Editor** (Foundation) ✓
  - [x] Horizontal scrolling timeline (robust hierarchical structure)
  - [x] Zoom in/out (pixel-based scaling)
  - [x] Multiple track lanes (synced w/ headers)
  - [x] Frame-snapped playhead (60fps visual alignment)
  - [x] Click-to-scrub interaction (click/drag anywhere on ruler to seek)
  - [x] Playback/Seek controls (Play, Pause, Step Frame)
  - [x] Frame ticks on ruler (subtle, at high zoom levels)
  - [x] Timecode display (HH:MM:SS:FF format)
  - [x] Dynamic track list (from project data, not hardcoded)
  - [x] Add/remove tracks UI
  - [ ] Audio playback integration

- [x] **Track System** (Revised Architecture) ✓
  - [x] Video tracks — hold video clips, image clips (stills), generative clips
  - [x] Audio tracks — hold audio clips, generative audio clips
  - [x] Marker track — point-in-time markers (single, non-duplicatable)
  - [x] Default new project: Video 1, Audio 1, Markers
  - [x] User can add additional Video/Audio tracks

- [x] **Clip System**
  - [x] Render clips on timeline tracks (positioned by start_time, sized by duration)
  - [x] Visual distinction: standard clips vs generative clips (dashed border, ✨ prefix)
  - [x] Clip Interactions:
    - [x] Move clips (drag body to reposition, frame-snapped 60fps)
    - [x] Resize clips (drag left/right edges, min duration 0.1s)
    - [x] Delete clips (right-click custom context menu, native menu suppressed)
    - [x] Move clips between compatible tracks (context menu up/down)
  - [x] Clip Creation:
    - [x] "Add to Timeline" (context menu) — renders at playhead
    - [x] Drag & Drop from Asset Panel — renders at drop position
  - [x] Clip labels (per-instance display name) editable in Attributes panel
  - [ ] Clip thumbnail/waveform preview
    - [x] **Thumbnailer Service**: Background FFmpeg task to generate cache images
    - [x] **Timeline Rendering**: UI logic to display cached thumbnails on clips

- [x] **Asset System** (Phase 2A) ✓
  - [x] Assets panel shows project assets (imported + generative)
  - [x] Import files via native file dialog
  - [x] Visual distinction: standard assets vs generative assets (⚙️ badge, dashed border)
  - [x] Drag assets to timeline to create clips (with compatibility checks)
    - [x] Copy imported files to project folder
    - [x] **Import Logic**: Create `Project::import_file` to copy external files to `audio/`, `video/`, etc.
    - [x] **Path Normalization**: Ensure `Asset` stores relative paths for portability
    - [x] **Collision Handling**: Auto-rename files if they already exist in project folder

- [ ] **Generative Assets** (Core Innovation) — In Progress
  - [x] "+ New Generative Video/Image/Audio" buttons in Assets panel
  - [x] Generative asset folder structure (generated/{type}/{id}/)
  - [x] Placeholder display for un-generated assets (dashed border, ⚙️ icon)
  - [x] Generative config file (config.json)
    - [x] Create config.json on generative asset creation
    - [x] Persist provider id selection
    - [x] Persist input bindings + version history
  - [x] Version management (v1, v2, ... in asset folder)
  - [x] Active version selection (stored in config.json)
  - [x] Delete active version with inline confirmation
  - [x] Active version load/save on project open
  - [x] Thumbnail updates after generation completes

- [ ] **Markers**
  - [ ] Click to add marker at playhead position
  - [ ] Drag markers to reposition
  - [ ] Delete markers
  - [ ] Marker labels (optional)
  - [ ] Marker colors (optional)

- [ ] **Audio Track**
  - [ ] Import MP3/WAV
  - [ ] **Waveform visualization** (essential)
  - [ ] Basic playback controls (play, pause, seek)
  - [ ] **Audio scrubbing** — hear audio while dragging playhead (critical for usability)

- [ ] **Selection & Attribute Editor**
  - [x] Clip selection state (single)
  - [x] Attribute panel for clip transforms (position/scale/rotation/opacity)
  - [ ] Track selection state
  - [ ] Asset selection state
  - [ ] Multi-select support for same-type items
  - [x] For generative clips: provider picker
  - [x] For generative clips: generate button
  - [x] For generative clips: dynamic input fields (schema-driven)
  - [x] For generative clips: version selector (active version)
  - [x] For generative clips: status/progress line (queued/running/done)

- [ ] **Smart Input Suggestions** (Timeline as Implicit Wiring)
  - [ ] When configuring generative clip inputs, auto-surface overlapping assets
  - [ ] "In Time Range" section at top of asset picker
  - [ ] "Other Assets" section below
  - [ ] Duration defaults to clip duration on timeline

- [ ] **Provider System**
  - [x] Provider entry data model (output type, input schema, connection info)
  - [x] Global provider config storage under `%LOCALAPPDATA%\NLA-AI-VideoCreator\providers\`
  - [x] Provider configuration UI (JSON editor modal)
  - [x] Dynamic input schema rendering (text, number, boolean, enum)
  - [ ] Health check / connection test
  - [x] ComfyUI adapter (first provider)
    - [x] Minimal T2I workflow (prompt + seed)
    - [x] Image output download/save into generated/{type}/{id}/

- [ ] **Generation Pipeline**
  - [ ] Queue generation jobs (async, non-blocking)
  - [ ] Job state tracking (queued/running/succeeded/failed)
  - [x] Progress/status feedback in UI
  - [x] Save generated files to asset folder (v1.png / v1.mp4 / v1.wav)
  - [x] Update config.json + asset active version on completion
  - [x] Trigger thumbnail refresh after generation
  - [ ] Cascading: regenerating dependent uses active version of inputs

- [ ] **FFmpeg Integration**
  - [ ] Export final timeline to video file
  - [ ] Assume FFmpeg on PATH
  - [ ] Basic export settings

- [ ] **Preview Window** (Priority: High)
  - [x] Clip transform data model (position/scale/rotation/opacity)
  - [x] Preview render loop (playhead-driven frame requests)
  - [x] Frame server v0: load stills + in-process FFmpeg decode worker
  - [x] Compositor v0: layer stack with opacity + basic scale/translate
  - [x] Preview panel renders composited frame via direct RGBA canvas upload
  - [ ] Transform pipeline v1: rotation + anchor/pivot support
  - [x] Canvas compositor + direct buffer upload (replace PNG cache)
  - [x] Native preview surface (wgpu) integration
  - [x] Frame caching/prefetch for smooth scrubbing

### Nice to Have (v0.2+)

- [ ] I2V generation (image-to-video providers)
- [ ] V2V transformation (video-to-video providers)
- [ ] Video extension
- [ ] Batch variations ("Generate 5 variations with different seeds")
- [ ] Beat detection / auto-marker placement
- [ ] Undo/redo
- [ ] Provider presets library
- [ ] fal.ai provider
- [ ] Replicate provider
- [ ] Multiple audio tracks with mute/solo
- [ ] Multiple video tracks with visibility toggle
- [ ] Audio generation providers
- [x] Rename/relabel clips and assets
- [ ] Export Parts (individual clips with descriptive filenames)
- [ ] Keyboard shortcuts

### Future Vision (v1.0+)

- [ ] Bundled FFmpeg (no external dependency)
- [ ] macOS and Linux builds
- [ ] Cloud sync for projects
- [ ] Hosted provider hub (premium)
- [ ] Collaborative editing
- [ ] Plugin system for custom adapters
- [ ] LUT/color grading
- [ ] Transitions and effects
- [ ] Basic video transforms (translate, rotate, scale)
- [ ] External asset references (outside project folder)

> **Philosophy:** This is NOT meant to replace a full video editor. If users need fine-grained control, they export their timed/sequenced clips (nicely named!) and bring them into their editor of choice. We stay focused on the AI generation workflow.

---

## 💼 Business Model (Long-term Vision)

### Open Source Core (MIT License)

The desktop application is **open source under MIT**:
- Maximum adoption and community contributions
- Establishes trust with technical users
- Benefits from security and quality auditing

### Monetization Avenues

1. **Premium Hosted Providers**
   - Curated, optimized workflows as a service
   - Users pay for API credits or subscription
   - Zero config—just works

2. **Pro Features (Freemium Model)**
   - Base app free
   - Pro license unlocks: cloud sync, priority support, advanced export codecs

3. **Marketplace**
   - User-contributed workflows and presets
   - Revenue share for creators

4. **Enterprise**
   - Team features, SSO, audit logs
   - Custom provider development

---

## 🛠️ Development Setup

### Prerequisites

| Dependency | Version | Installation |
|------------|---------|--------------|
| Rust | 1.75+ | [rustup.rs](https://rustup.rs) |
| Dioxus CLI | Latest | `cargo install dioxus-cli` |
| FFmpeg | 6.0+ | [ffmpeg.org](https://ffmpeg.org/download.html) or `winget install ffmpeg` |

### Secrets / API Keys

Provider API keys and secrets are stored in a `.env` file at the project root (git-ignored). Users running locally manage their own `.env`.

```env
# Example .env
FAL_API_KEY=your_fal_key_here
REPLICATE_API_TOKEN=your_replicate_token_here
```

### Getting Started

```bash
# Clone the repo
git clone https://github.com/yourusername/nla-ai-videocreator-rust.git
cd nla-ai-videocreator-rust

# Run in development mode
dx serve

# Build for release
dx build --release
```

### Project Structure (Proposed)

```
nla-ai-videocreator-rust/
├── Cargo.toml
├── Dioxus.toml
├── src/
│   ├── main.rs              # App entry point
│   ├── app.rs               # Root component & state
│   ├── timeline.rs          # Timeline editor components
│   ├── components/          # (Future) UI components
│   │   ├── preview/         # Preview window
│   │   ├── panels/          # Side panels (assets, attributes)
│   │   └── common/          # Shared UI components
│   ├── state/               # App state management
│   │   ├── mod.rs           # State module root
│   │   ├── app_state.rs     # Global app state
│   │   ├── project.rs       # Project state
│   │   ├── selection.rs     # Selection state (shared across views)
│   │   └── providers.rs     # Provider state
│   ├── providers/           # Provider adapter implementations
│   │   ├── mod.rs           # Provider traits and types
│   │   ├── comfyui.rs       # ComfyUI adapter
│   │   └── fal.rs           # fal.ai adapter
│   ├── core/                # Core logic (non-UI)
│   │   ├── ffmpeg.rs        # FFmpeg wrapper
│   │   ├── audio.rs         # Audio processing
│   │   ├── project_io.rs    # Project save/load
│   │   └── job_queue.rs     # Background task queue
│   └── schema/              # JSON schemas for providers, project files
├── assets/                  # Static assets (icons, fonts)
├── workflows/               # Example ComfyUI workflows
└── docs/                    # Additional documentation
    └── CONTENT_ARCHITECTURE.md  # Content & generation architecture
```

### State Architecture

Inspired by **Blender's multi-view model**:
- **Shared core data** — The project, assets, timeline clips exist once in memory
- **View-specific state** — Each panel (asset browser, timeline, attribute editor) may have its own selection, scroll position, etc.
- **Selection is centralized** — A single selection state that multiple views can observe and modify
- **Modular and flat** — Avoid deep nesting; prefer distinct state slices that can be composed

This allows:
- Asset browser showing the same asset that's on the timeline
- Attribute editor responding to selections from either view
- Multiple views staying in sync without tight coupling

---

## 📋 Decision Log

| Decision | Rationale | Status |
|----------|-----------|--------|
| Use Dioxus Desktop (not Web/Tauri) | Full local experience, single DX language (RSX), simpler architecture | ✅ Decided |
| FFmpeg on PATH for MVP | Simplifies initial development; bundling is later optimization | ✅ Decided |
| JSON for provider configs | Machine-readable, toolable, familiar | ✅ Decided |
| Project-local asset folders | Portable, self-contained projects | ✅ Decided |
| Folder = Project (KISS) | Simple mental model, easy to backup/share | ✅ Decided |
| Async job queue for generations | Non-blocking UI while waiting for slow API calls | ✅ Decided |
| Single-purpose provider entries | Simpler mental model; add service twice if it does multiple things | ✅ Decided |
| MIT License | Maximum adoption, permissive, standard for tools | ✅ Decided |
| Secrets via .env | Simple, familiar, users manage their own keys | ✅ Decided |
| Lean development philosophy | Build custom, avoid dependency bloat, iterate with user | ✅ Decided |
| Modular state (Blender-inspired) | Multiple views can share/observe same data with their own view state | ✅ Decided |
| Labels separate from filenames | Enables friendly display names + future "Export Parts" feature | ✅ Decided |
| Audio scrubbing is essential | Without hearing audio while scrubbing, the tool is unusable for music-synced work | ✅ Decided |
| UI fluidity is non-negotiable | Hover effects, smooth transitions, polished feel from day one | ✅ Decided |
| Dioxus 0.7 (latest) | Hot-patching support, better signals, performance improvements | ✅ Decided |
| Transitions disabled during drag | Instant resize feedback; transitions only for collapse/expand | ✅ Decided |
| App-local Timeline State (Temp) | Kept state simple in app.rs until data model requirements mature | ✅ Decided |
| Scroll-synced Track Labels | CSS sticky positioning for rock-solid sync vs JS event listeners | ✅ Decided |
| Draggable Playhead | Real-time updating during drag for immediate feedback | ✅ Decided |
| 1-Second Step Buttons | Frame-stepping felt too slow; 1s steps preferred for navigating | ✅ Decided |
| Frame-snapped Playhead | All seeking snaps to 60fps frame boundaries for accurate positioning | ✅ Decided |
| Click-to-scrub Interaction | Click anywhere on ruler to seek; playhead follows cursor, not grabbed | ✅ Decided |
| Hierarchical Timeline Layout | Fixed left column + scrollable right column; no JS scroll sync needed | ✅ Decided |
| Playhead as Visual Indicator | Triangle handle is purely visual; interaction is on ruler bar | ✅ Decided |
| Generative Assets as Explicit Creation | Users explicitly create generative assets via UI; they start "hollow" and get populated through generation | ✅ Decided |
| Generative Assets Have Versions | Each generation creates a new version; user picks active version; dependent assets use active version | ✅ Decided |
| Timeline as Implicit Wiring | Overlapping assets auto-surface as input suggestions; no explicit linking required | ✅ Decided |
| Providers Grouped by Output Type | Video/Image/Audio; input requirements vary per provider via dynamic schema | ✅ Decided |
| No Separate Keyframe Track | Images are clips on Video tracks; "keyframes" are just overlapping reference images | ✅ Decided |
| In-Project Assets Only (MVP) | All assets must be in project folder; external refs are future enhancement | ✅ Decided |
| Canvas Compositor Strategy | Skip DOM overlay; build robust pixel-buffer compositing for transforms/blending immediately | ✅ Decided |

---

## 🤝 Contributing

*(To be expanded)*

This project welcomes contributions! Areas where help is especially appreciated:
- Provider adapters for new services
- UI/UX improvements
- Cross-platform testing (macOS, Linux)
- Documentation and tutorials

---

## 📜 License

**MIT License**

This project is open source under the MIT License. See [LICENSE](./LICENSE) for details.

---

## 🗺️ Roadmap

```
v0.1 - Foundation
├── Basic timeline UI
├── Audio import & playback
├── Manual marker placement
├── Keyframe import
├── ComfyUI image generation adapter
└── FFmpeg export

v0.2 - Generation Flow
├── I2V generation pipeline
├── Job queue with progress UI
├── Provider health checks
└── fal.ai adapter

v0.3 - Polish
├── Undo/redo
├── Improved waveform
├── Keyboard shortcuts
└── Marker auto-generation from beats

v0.4 - Multi-platform
├── macOS builds
├── Linux builds
└── Bundled FFmpeg

v1.0 - Public Release
├── Stable API for adapters
├── Documentation
├── Premium hosted providers (beta)
└── Community workflow library
```

---

## 📞 Contact

*(To be filled in)*

---

## 📊 Current Status (2026-01-04)

### Completed ✅
| Area | Status | Notes |
|------|--------|-------|
| **UI Shell** | ✅ Complete | Title bar, panels, timeline, status bar |
| **Panel System** | ✅ Complete | Resizable, collapsible, hover effects |
| **Data Model** | ✅ Complete | Project, Track, Clip, Asset, Marker structs |
| **Project Management** | ✅ Complete | New project dialog, create folder, save/load JSON |
| **Timeline Foundation** | ✅ Complete | Scroll, zoom, playhead, ruler, timecode |
| **Track System** | ✅ Complete | Video/Audio/Marker tracks, add/remove/reorder |
| **Context Menus** | ✅ Complete | Custom right-click menus (delete, move up/down) |
| **Window Config** | ✅ Complete | Custom title, no default menu bar |
| **Asset Panel** | ✅ Complete | Display assets, import files via native dialog |

### In Progress 🔄
| Area | Status | Next Steps |
|------|--------|------------|
| **Clip System** | � In Progress | Placing clips works, previews next |
| **Thumbnails** | ✅ Complete | Background generation & `nla://` protocol |
| **Preview Engine** | 🟨 In Progress | v0 frame server + compositor wired; next: canvas buffer + caching |
| **Audio Playback** | 🔲 Not Started | Waveform visualization, sync with timeline |
| **File Copy** | 🔲 Not Started | Copy imported files into project folder |

### Code Structure
```
src/
├── main.rs          # Entry point, window config
├── app.rs           # Main App component, UI shell, dialogs
├── constants.rs     # Shared UI constants (colors, sizing, scripts)
├── components/      # UI components (startup modal, panels, fields)
├── timeline.rs      # TimelinePanel, ruler, tracks, playback controls
├── hotkeys/         # Hotkey system (Registry, Actions, Context)
└── state/
    ├── mod.rs       # Module exports
    ├── asset.rs     # Asset, AssetKind (file & generative)
    └── project.rs   # Project, Track, Clip, Marker, save/load
```

### Recent Changes (Session Log)
- **2026-01-08:** Completed REFACTOR.md Step 1.11 by relocating remaining helper functions into `core/` and `state/` modules (no helper functions remain in `app.rs` or component files).
- **2026-01-08:** Extracted Attributes and Assets panels into `src/components/attributes/` and `src/components/assets/`; relocated provider/media/generative helpers into `core/` and `state/`, and moved timeline zoom bounds into the timeline module.
- **2026-01-08:** Began refactor plan by extracting shared UI constants, startup modal, and panel components into `src/constants.rs` and `src/components/`; moved shared input fields into `components/common/fields.rs`.
- **2026-01-08:** Generative thumbnail cache now clears when no active version exists (prevents stale thumbnails after deleting all versions)
- **2026-01-08:** Preview cache now invalidates generative asset folders on generate/delete so regenerated versions update immediately
- **2026-01-08:** Version dropdown now lists newest first; added inline delete confirmation that keeps selection position
- **2026-01-08:** Split Generative controls into two cards: Generative (version/provider/generate) and Provider Inputs (dynamic fields)
- **2026-01-08:** Asset context menus now clamp to the Assets panel width so they don't get hidden by the native preview overlay
- **2026-01-08:** Attributes panel now remounts on clip selection to refresh fields when switching clips
- **2026-01-08:** Added generative version selector in Attributes panel; changing active version refreshes thumbnails and preview
- **2026-01-08:** Added per-clip labels in Attributes panel; timeline labels now respect clip names and show active generative version
- **2026-01-08:** Generative assets now default to sequential names (Gen Image 1, Gen Video 2) and asset list titles include active version
- **2026-01-07:** Implemented centralized hotkey system (`src/hotkeys/`) with action-based architecture and context awareness
- **2026-01-07:** Added global hotkeys for Timeline Zoom (+/- on Numpad and standard keys)
- **2026-01-07:** Attributes panel now shows provider picker for generative clips and saves provider selection to config.json
- **2026-01-07:** Provider entries now load from the global providers folder and display their input schema
- **2026-01-07:** Added a global Providers JSON editor modal (top bar) for quick provider edits
- **2026-01-07:** Providers modal now renders at the app root and messaging reflects global-only config
- **2026-01-07:** Native preview overlay now hides while modal dialogs are open (prevents WGPU surface covering UI)
- **2026-01-07:** Added ComfyUI workflow template `workflows/sdxl_simple_example_API.json` and optional `workflow_path` on ComfyUI providers
- **2026-01-07:** ComfyUI adapter now submits API workflows, polls history, and downloads image outputs
- **2026-01-07:** Generative attributes now render dynamic input fields, generate button, and status line
- **2026-01-07:** Generative output now writes versioned files, updates config + active version, and refreshes thumbnails
- **2026-01-07:** Thumbnailer now supports generative image/video assets by resolving active versions
- **2026-01-07:** Still image thumbnails now render via the image crate (covers regular + generative images)
- **2026-01-07:** Project load now syncs generative asset config.json and active version state
- **2026-01-07:** Generative asset creation now writes a default config.json in the asset folder
- **2026-01-07:** Added provider + generative config data models and storage helpers (config.json + global providers)
- **2026-01-07:** Expanded the Generative/Provider/Generation TODOs into an atomic ComfyUI MVP plan
- **2026-01-07:** Defaulted timeline zoom to Fit on project open/create once the viewport width is known
- **2026-01-07:** Added timeline zoom bounds based on visible width, plus Fit/Frames buttons and adaptive minimum clip width
- **2026-01-07:** Distinguished preview plate vs background by clearing the GPU surface to the UI background, adding a black plate fill, and drawing a 1px plate border
- **2026-01-07:** Fixed native-size preview scaling by tracking source dimensions alongside cached frames
- **2026-01-07:** Preview now renders clips at native size (no auto-fit scaling to canvas)
- **2026-01-07:** Fixed GPU rotation skew by accounting for preview aspect ratio in the shader
- **2026-01-07:** Added CPU fallback rotation support using imageproc (GPU + CPU paths now respect clip rotation)
- **2026-01-07:** Added idle-time prefetch (800ms delay, 5s ahead + 1s behind) to warm the preview cache when not playing
- **2026-01-07:** GPU preview now applies clip rotation (rotation degrees respected during compositing)
- **2026-01-07:** Added a title-bar "HW Dec" toggle to force CPU decode for A/B comparisons
- **2026-01-07:** Defaulted preview stats toggle to off (still available via the header toggle)
- **2026-01-07:** Moved preview stats into a docked right-side column so they stay visible above the native wgpu surface
- **2026-01-07:** Expanded preview stats into a vertical overlay with detailed video decode breakdown (seek, packet, transfer, scale, copy)
- **2026-01-07:** Expanded playback prefetch window to 3 seconds to improve sustained playback responsiveness
- **2026-01-07:** Increased preview frame cache budget to 8GB for smoother scrubbing in larger regions
- **2026-01-07:** Added parallel decode scheduling (worker pool keyed by track lanes) to allow per-layer decoding concurrently
- **2026-01-07:** Added hardware-accelerated decode support (Windows D3D11VA/DXVA2) with automatic CPU fallback
- **2026-01-07:** Preview stats now include a `hwdec` percentage to indicate hardware decode usage
- **2026-01-07:** Added a `SHOW_CACHE_TICKS` toggle to enable/disable the timeline cache bucket overlay
- **2026-01-07:** Cache tick overlay now uses a per-asset frame index to mark buckets based on any cached frame in the clip range (stills fill all buckets once cached)
- **2026-01-07:** Added per-clip cache tick overlay to visualize cached frame buckets on the timeline
- **2026-01-07:** Capped in-process FFmpeg decoders with an LRU eviction policy (8 max) and added sequential playback decode mode
- **2026-01-07:** Increased playback prefetch window to 1s and forced a preview render after GPU init; native overlay stays hidden until first upload
- **2026-01-07:** Fixed WGPU preview shader uniform layout to prevent pipeline validation crashes
- **2026-01-07:** Switched native preview to upload per-layer textures and composite them in WGPU using per-layer transforms and opacity
- **2026-01-07:** Preview render loop now emits layer stacks for the GPU path and triggers native redraws when layers update
- **2026-01-07:** Reworked preview stats labeling to show scan time (excluding decode/still) for clearer left-to-right stage timing
- **2026-01-07:** Allowed the preview panel to shrink vertically to avoid the native surface overlapping the timeline in short windows
- **2026-01-07:** Added a small Windows-only native preview offset to compensate for WebView2 client-area inset
- **2026-01-07:** Kept preview header layout stable when stats are hidden and removed preview padding to align the native surface
- **2026-01-07:** Added a title-bar toggle for preview stats and anchored native preview bounds to a dedicated host rectangle
- **2026-01-07:** Aligned native preview bounds to the canvas element, moved stats into the preview header, and switched native letterbox bars to black
- **2026-01-07:** Fixed native preview positioning to use parent-relative coordinates (avoids double offset when moving the app window)
- **2026-01-07:** Adjusted native preview window positioning to use window-origin coordinates and raised the child window to the top of the z-order
- **2026-01-07:** Added wgpu upload timing to the preview performance overlay
- **2026-01-07:** WGPU preview now uploads RGBA frames to a texture and renders via a quad (canvas uploads suppressed once native preview is active)
- **2026-01-07:** Restored preview canvas visibility while the native host is active and fixed preview overlay stacking so stats stay visible
- **2026-01-06:** Added preview performance overlay (cache hit rate + per-stage timing) to guide optimization work
- **2026-01-06:** Served preview frames from in-memory PNG store to remove per-frame disk writes
- **2026-01-06:** Switched preview output to raw RGBA canvas uploads (removed PNG encode from the loop)
- **2026-01-06:** Added preview stats reference doc (overlay field definitions)
- **2026-01-06:** Added wgpu native preview surface spike (child window + bounds sync)
- **2026-01-06:** Throttled native preview init/update to avoid UI stalls (bounds change + redraw gating)
- **2026-01-06:** Updated ffmpeg-next to v8.0.0 to align with FFmpeg 7.x headers from vcpkg
- **2026-01-06:** Added in-process FFmpeg decode worker for preview frame extraction
- **2026-01-06:** Removed ffmpeg scale filter from preview decode to avoid empty frames; scaling happens in Rust after decode
- **2026-01-06:** Fixed preview latest-wins gating so in-flight renders don't get discarded when the render gate is busy
- **2026-01-06:** Added preview frame cache (2GB budget), latest-wins scheduling, and prefetch window for smoother scrubbing
- **2026-01-06:** Clip context menu now supports moving clips up/down to compatible tracks
- **2026-01-06:** Attribute editor numeric fields commit on blur/Enter to avoid input jitter
- **2026-01-06:** Added preview renderer v0 (ffmpeg frame extraction + compositing) and playhead-driven preview updates
- **2026-01-06:** Added clip transforms + single-clip selection with transform editing in Attributes panel
- **2026-01-06:** Startup modal now captures project resolution, FPS, and duration; location field moved to bottom with separator
- **2026-01-06:** Added project duration to settings and extended timeline ruler ticks across full duration
- **2026-01-06:** Fixed left-edge trim drift by anchoring to drag-start end time
- **2026-01-06:** Removed unused `mut` warnings from thumbnail tick signals and clip resize logic
- **2026-01-06:** Added clip trim-in state for left-edge trimming; timeline thumbnails now offset by trim-in and clip filmstrip is clipped to bounds
- **2026-01-06:** Fixed thumbnail refresh wiring for asset/timeline panels and duration probe helpers
- **2026-01-06:** Asset durations now cached via ffprobe for audio/video; clips use asset duration on drop/add and resizing is clamped to source length
- **2026-01-06:** Thumbnail URLs now cache-bust on refresh and missing files no longer render broken images
- **2026-01-06:** Asset panel shows first-frame thumbnails for visual assets; timeline thumbnails distribute across clips using 1s sampling with repeat-fill on zoom
- **2026-01-06:** Implemented robust custom protocol (`http://nla.localhost`) for serving local thumbnails
- **2026-01-06:** Added "Rendering & Preview Strategy" to docs
- **2026-01-06:** Promoted Preview Window and Thumbnails to MVP status based on user feedback
- **2026-01-06:** Added right-click context menu to delete projects from startup modal
- **2026-01-06:** Fixed project list layout (compact items, proper overflow handling, scrollable)
- **2026-01-06:** Improved Startup Modal: existing projects now listed automatically, file dialogs start from projects folder
- **2026-01-04:** Implemented custom context menus for track management
- **2026-01-04:** Added "Move Up/Down" track reordering via context menu
- **2026-01-04:** Fixed window title and removed default Win/Edit/Help menu bar
- **2026-01-04:** Added viewport-constrained context menu positioning
- **2026-01-04:** Implemented New Project modal dialog with folder creation
- **2026-01-04:** Added track add/remove functionality with UI buttons
- **2026-01-04:** Integrated Project data model with timeline (dynamic tracks)
- **2026-01-04:** Created core data structures (Project, Track, Clip, Asset, Marker)
- **2026-01-04:** Implemented timeline clip interactions (Move, Resize, Delete, Drag & Drop)
- **2026-01-04:** Refined resize handles and fixed context menus

---

## 🧭 Development Philosophy

> **"Tight, lean, focused."**

This project intentionally:
- **Avoids premature abstraction** — We discover the right patterns during implementation, not before
- **Minimizes external dependencies** — If we can build it simply, we do
- **Iterates with the user** — Frequent check-ins, test early, refine as we go
- **Stays in its lane** — AI video generation workflow, not a full-featured video editor
- **Values feel over features** — Every component should feel intentional and polished
- **Prioritizes fluidity** — Smooth hover effects, transitions, and scrubbing from the start

We start with the UI shell, dial in the look and feel, then layer in functionality. Style and UX decisions are made early to avoid refactoring across the codebase later.

---

*Last updated: 2026-01-08*


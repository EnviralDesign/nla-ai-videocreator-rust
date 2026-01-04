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
- Provider configuration for this project

### 2. App Settings

Global application settings (not per-project):
- **Projects folder location** — Where new projects are created by default
- UI preferences (theme, layout)
- Default providers / presets
- FFmpeg path override (if not on PATH)

### 3. Tracks

The timeline consists of layered tracks:

| Track Type | Purpose |
|------------|---------|
| **Audio Track** | Holds imported audio files (music, voiceover, SFX) |
| **Marker Track** | Holds keypoints/markers for timing (beat markers, scene breaks) |
| **Keyframe Track** | Holds keyframe images at specific points |
| **Video Track** | Holds generated or imported video segments |
| **Text/Prompt Track** | *(Future)* Holds prompt segments tied to timeline regions |

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
- Details of the adapter interface will be discovered during implementation—we're keeping this intentionally vague until we experiment with real ComfyUI workflows.

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
1. User opens the app, creates a new project
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
└── .providers/           # Provider configs for this project
    └── my_comfy_workflow.json
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

- [ ] **Project Management**
  - Create, open, save projects
  - Project settings (resolution, fps)

- [x] **Timeline Editor** (Foundation) ✓
  - [x] Horizontal scrolling timeline (with synced headers)
  - [x] Zoom in/out (pixel-based scaling)
  - [x] Multiple track lanes (Audio, Markers, Keyframes, Video, synced w/ headers)
  - [x] Draggable playhead with live timecode
  - [x] Playback/Seek controls (Play, Pause, Step Frame)
  - [ ] Audio playback integration

- [ ] **Audio Track**
  - Import MP3/WAV
  - **Waveform visualization** (essential)
  - Basic playback controls (play, pause, seek)
  - **Audio scrubbing** — hear audio while dragging playhead (critical for usability)

- [ ] **Marker Track**
  - Click to add marker at playhead position
  - Drag markers to reposition
  - Delete markers
  - Marker labels (optional)

- [ ] **Keyframe Track**
  - Add keyframe slots at marker positions
  - Import images into keyframe slots
  - Thumbnail preview in timeline

- [ ] **Video Track**
  - Import existing video segments
  - Display video segments with duration
  - Basic clip arrangement

- [ ] **Provider System**
  - ComfyUI adapter (generic workflow support)
  - Provider configuration UI
  - Health check / connection test

- [ ] **Image Generation**
  - Generate image via configured provider
  - Prompt input UI
  - Progress/status feedback
  - Auto-save to keyframes folder

- [ ] **FFmpeg Integration**
  - Export final timeline to video file
  - Assume FFmpeg on PATH
  - Basic export settings

- [ ] **Selection & Attribute Editor**
  - Track selection state (what's selected where)
  - Context-sensitive attribute panel
  - Multi-select support for same-type items

### Nice to Have (v0.2+)

- [ ] I2V generation from keyframes
- [ ] V2V transformation
- [ ] Video extension
- [ ] Beat detection / auto-marker placement
- [ ] Undo/redo
- [ ] Provider presets library
- [ ] fal.ai provider
- [ ] Replicate provider
- [ ] Preview window with real-time playback
- [ ] Multiple audio tracks with mute/solo
- [ ] Multiple video tracks with mute/visibility toggle
- [ ] Prompt track (prompts as timeline regions)
- [ ] Audio generation providers
- [ ] Rename/relabel clips and assets
- [ ] Export Parts (individual clips with descriptive filenames)

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

*Last updated: 2025-01-03*


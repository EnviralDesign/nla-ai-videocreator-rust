# NLA AI Video Creator — Codebase Refactor Plan

> **Purpose**: This document provides an exhaustive, step-by-step refactoring plan to break apart monolithic code files into a clean, organized module structure optimized for AI coding agents.

---

## Executive Summary

### Current State — The Monoliths

| File | Lines | Size | Primary Problem |
|------|-------|------|-----------------|
| [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) | 4,870 | 228KB | **Massive** — Contains 44+ components, all UI logic, dialogs, panels, utilities |
| [src/timeline.rs](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs) | 1,218 | 56KB | Single file for all timeline components |
| [src/core/preview.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs) | 1,242 | 39KB | Frame rendering + caching + compositing mixed |
| [src/core/preview_gpu.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs) | 1,039 | 37KB | GPU surface + shaders + rendering logic |
| [src/state/project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs) | 832 | 28KB | Data model + file I/O + business logic mixed |
| [src/core/video_decode.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/video_decode.rs) | 749 | 23KB | Acceptable but could be cleaner |

### Target State — AI-Optimized Structure

The refactored structure prioritizes:
1. **Single Responsibility** — Each file handles one concern
2. **Minimal Context Loads** — AI agents can work on one module without loading unrelated code
3. **Clear Entry Points** — [mod.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/mod.rs) files provide navigation and re-exports
4. **Predictable Naming** — File names match their primary export

---

## Proposed Directory Structure

```
src/
├── main.rs                      # Entry point only (unchanged, ~100 lines)
├── app.rs                       # Root App component only (~300 lines target)
│
├── components/                  # UI Components (NEW DIRECTORY)
│   ├── mod.rs                   # Re-exports all components
│   ├── startup_modal.rs         # Startup/project creation modal
│   ├── title_bar.rs             # Title bar component
│   ├── side_panel.rs            # Generic side panel wrapper
│   ├── status_bar.rs            # Bottom status bar
│   ├── preview_panel.rs         # Preview window component
│   │
│   ├── assets/                  # Assets panel components
│   │   ├── mod.rs
│   │   ├── assets_panel.rs      # Main assets panel
│   │   └── asset_item.rs        # Single asset item
│   │
│   ├── attributes/              # Attributes panel components
│   │   ├── mod.rs
│   │   ├── attributes_panel.rs  # Main attributes panel
│   │   ├── numeric_field.rs     # Numeric input field
│   │   ├── provider_fields.rs   # Provider-specific input fields
│   │   └── generative_controls.rs # Generative asset controls
│   │
│   └── common/                  # Shared UI components
│       ├── mod.rs
│       └── fields.rs            # Generic input field components
│
├── timeline/                    # Timeline module (PROMOTED FROM SINGLE FILE)
│   ├── mod.rs                   # Re-exports, constants
│   ├── panel.rs                 # TimelinePanel component
│   ├── ruler.rs                 # TimeRuler component
│   ├── track_label.rs           # TrackLabel component
│   ├── track_row.rs             # TrackRow component
│   ├── clip_element.rs          # ClipElement component
│   └── playback_controls.rs     # PlaybackBtn and controls
│
├── state/                       # State management (EXISTING, REORGANIZED)
│   ├── mod.rs                   # Re-exports
│   ├── project/                 # Project state (SPLIT FROM SINGLE FILE)
│   │   ├── mod.rs               # Re-exports Project, Track, Clip, Marker
│   │   ├── project.rs           # Project struct + core methods
│   │   ├── track.rs             # Track, TrackType
│   │   ├── clip.rs              # Clip, ClipTransform
│   │   ├── marker.rs            # Marker
│   │   ├── settings.rs          # ProjectSettings
│   │   └── persistence.rs       # Save/load/create operations
│   ├── asset.rs                 # Asset, AssetKind (keep as-is, well-sized)
│   ├── selection.rs             # SelectionState (keep as-is)
│   ├── providers.rs             # Provider types (keep as-is)
│   └── generative.rs            # GenerativeConfig (keep as-is)
│
├── core/                        # Core logic (EXISTING, REORGANIZED)
│   ├── mod.rs                   # Re-exports
│   ├── preview/                 # Preview rendering (SPLIT FROM 2 FILES)
│   │   ├── mod.rs               # Re-exports
│   │   ├── renderer.rs          # PreviewRenderer struct + main logic
│   │   ├── cache.rs             # FrameCache, PlateCache
│   │   ├── layers.rs            # PreviewLayer, compositing helpers
│   │   ├── types.rs             # PreviewStats, PreviewFrameInfo, etc.
│   │   └── utils.rs             # Utility functions (time conversion, etc.)
│   ├── preview_gpu/             # GPU rendering (SPLIT FROM SINGLE FILE)
│   │   ├── mod.rs               # Re-exports
│   │   ├── surface.rs           # PreviewGpuSurface
│   │   ├── shaders.rs           # PREVIEW_SHADER, BORDER_SHADER constants
│   │   ├── types.rs             # Vertex, LayerUniform, BorderUniform, etc.
│   │   └── layers.rs            # GpuLayer, layer creation/upload
│   ├── video_decode.rs          # Keep as single file (acceptable size)
│   ├── thumbnailer.rs           # Keep as single file (acceptable size)
│   ├── media.rs                 # Keep as single file (small)
│   ├── preview_store.rs         # Keep as single file (small)
│   ├── provider_store.rs        # Keep as single file (small)
│   └── generation.rs            # Keep as single file (small)
│
├── providers/                   # Provider adapters (KEEP AS-IS)
│   ├── mod.rs
│   └── comfyui.rs
│
├── hotkeys/                     # Hotkey system (KEEP AS-IS)
│   └── mod.rs
│
├── utils.rs                     # General utilities (KEEP AS-IS)
│
└── constants.rs                 # Shared constants (NEW FILE)
```

---

## Phase 1: Extract [app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) Components

> **Priority: HIGHEST**  
> **Current size**: 4,870 lines  
> **Target size**: ~300 lines  
> **Impact**: Most critical refactor — unlocks all other work

### Step 1.1: Create `src/constants.rs`

**Action**: Extract all color constants and UI constants from [app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 1-70.

**Source file**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 1-70  
**Target file**: `src/constants.rs` (NEW)

**Content to extract**:
```rust
// Lines 28-45 of app.rs - Color constants
pub const BG_BASE: &str = "#0a0a0b";
pub const BG_SURFACE: &str = "#141414";
pub const BG_ELEVATED: &str = "#1a1a1a";
pub const BG_HOVER: &str = "#262626";
pub const BORDER_SUBTLE: &str = "#1f1f1f";
pub const BORDER_DEFAULT: &str = "#27272a";
pub const BORDER_STRONG: &str = "#3f3f46";
pub const BORDER_ACCENT: &str = "#3b82f6";
pub const TEXT_PRIMARY: &str = "#fafafa";
pub const TEXT_SECONDARY: &str = "#a1a1aa";
pub const TEXT_MUTED: &str = "#71717a";
pub const TEXT_DIM: &str = "#52525b";
pub const ACCENT_AUDIO: &str = "#3b82f6";
pub const ACCENT_MARKER: &str = "#f97316";
pub const ACCENT_VIDEO: &str = "#22c55e";
// ... and all others through line ~70

// Preview/Timeline constants
pub const PREVIEW_PREFETCH_WINDOW_FRAMES: u32 = 30;
pub const PREVIEW_IDLE_PREFETCH_BEHIND_SECONDS: f64 = 1.0;
pub const SHOW_CACHE_TICKS: bool = false;
pub const TIMELINE_MIN_ZOOM_FLOOR: f64 = 0.1;
pub const TIMELINE_MAX_PX_PER_FRAME: f64 = 8.0;
```

**Update [app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs)**: Add `use crate::constants::*;`

**Update [main.rs](file:///c:/repos/nla-ai-videocreator-rust/src/main.rs)**: Add `mod constants;`

---

### Step 1.2: Create `src/components/mod.rs`

**Action**: Create the components directory structure.

**Create files**:
1. `src/components/mod.rs`
2. `src/components/common/mod.rs`
3. `src/components/assets/mod.rs`
4. `src/components/attributes/mod.rs`

**`src/components/mod.rs` content**:
```rust
//! UI Components
//!
//! This module contains reusable UI components organized by domain.

pub mod common;
pub mod assets;
pub mod attributes;

mod startup_modal;
mod title_bar;
mod side_panel;
mod status_bar;
mod preview_panel;

pub use startup_modal::StartupModal;
pub use title_bar::TitleBar;
pub use side_panel::SidePanel;
pub use status_bar::StatusBar;
pub use preview_panel::PreviewPanel;
```

---

### Step 1.3: Extract [StartupModal](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#1864-2486)

**Action**: Move [StartupModal](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#1864-2486) component to its own file.

**Source**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 1864-2485 (function [StartupModal](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#1864-2486))  
**Target**: `src/components/startup_modal.rs` (NEW)

**Contains**:
- [StartupModal](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#1864-2486) component (~620 lines)
- Nested helper functions [parse_u32](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#1892-1900), [parse_f64](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#1901-1909)
- All associated styling and logic

**Dependencies to import**:
```rust
use dioxus::prelude::*;
use std::path::PathBuf;
use crate::state::ProjectSettings;
use crate::constants::*;
```

---

### Step 1.4: Extract [TitleBar](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#2487-2589)

**Action**: Move [TitleBar](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#2487-2589) component to its own file.

**Source**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 2487-2588  
**Target**: `src/components/title_bar.rs` (NEW)

**Contains**:
- [TitleBar](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#2487-2589) component (~100 lines)

**Dependencies**:
```rust
use dioxus::prelude::*;
use crate::constants::*;
```

---

### Step 1.5: Extract [SidePanel](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#2590-2750)

**Action**: Move [SidePanel](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#2590-2750) component to its own file.

**Source**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 2590-2749  
**Target**: `src/components/side_panel.rs` (NEW)

**Contains**:
- [SidePanel](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#2590-2750) component (~160 lines)

---

### Step 1.6: Extract [StatusBar](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4077-4091)

**Action**: Move [StatusBar](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4077-4091) component to its own file.

**Source**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 4077-4090  
**Target**: `src/components/status_bar.rs` (NEW)

**Contains**:
- [StatusBar](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4077-4091) component (~15 lines)

---

### Step 1.7: Extract [PreviewPanel](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#3941-4076)

**Action**: Move [PreviewPanel](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#3941-4076) component to its own file.

**Source**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 3941-4075  
**Target**: `src/components/preview_panel.rs` (NEW)

**Contains**:
- [PreviewPanel](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#3941-4076) component (~135 lines)

---

### Step 1.8: Extract Numeric/Provider Field Components

**Action**: Move form field components to `src/components/common/fields.rs`.

**Source**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 2751-3006  
**Target**: `src/components/common/fields.rs` (NEW)

**Contains**:
- [NumericField](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#2751-2828) component (lines 2751-2827, ~70 lines)
- [ProviderTextField](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#2829-2886) component (lines 2829-2885, ~55 lines)
- [ProviderFloatField](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#2887-2947) component (lines 2887-2946, ~60 lines)
- [ProviderIntegerField](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#2948-3007) component (lines 2948-3006, ~60 lines)

**`src/components/common/mod.rs` content**:
```rust
mod fields;
pub use fields::*;
```

---

### Step 1.9: Extract [AttributesPanelContent](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#3009-3941)

**Action**: Move [AttributesPanelContent](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#3009-3941) to attributes module.

**Source**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 3009-3940 (~930 lines!)  
**Target**: `src/components/attributes/attributes_panel.rs` (NEW)

**Contains**:
- [AttributesPanelContent](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#3009-3941) component

**This is a LARGE component** — after extraction, consider further splitting:
- `src/components/attributes/clip_attributes.rs` — clip transform editing
- `src/components/attributes/generative_controls.rs` — version selector, generate button
- `src/components/attributes/provider_inputs.rs` — dynamic provider field rendering

---

### Step 1.10: Extract Assets Panel Components

**Action**: Move asset-related components to assets module.

**Source 1**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 4424-4603 ([AssetsPanelContent](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4424-4604))  
**Target 1**: `src/components/assets/assets_panel.rs` (NEW)

**Source 2**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 4605-4865 ([AssetItem](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4605-4866))  
**Target 2**: `src/components/assets/asset_item.rs` (NEW)

**`src/components/assets/mod.rs` content**:
```rust
mod assets_panel;
mod asset_item;

pub use assets_panel::AssetsPanelContent;
pub use asset_item::AssetItem;
```

---

### Step 1.11: Extract Utility Functions from [app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs)

**Action**: Move standalone utility functions to appropriate locations.

**Functions to extract** (from [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 4092-4422):

| Function | Lines | Target File |
|----------|-------|-------------|
| [spawn_asset_duration_probe](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4092-4132) | 4092-4131 | [src/core/media.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/media.rs) |
| [spawn_missing_duration_probes](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4133-4146) | 4133-4145 | [src/core/media.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/media.rs) |
| [resolve_asset_duration_seconds](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4147-4187) | 4147-4186 | [src/core/media.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/media.rs) |
| [ensure_generative_config](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4188-4210) | 4188-4209 | [src/state/generative.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/generative.rs) |
| [load_global_provider_entries](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4211-4220) | 4211-4219 | [src/core/provider_store.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/provider_store.rs) |
| [generative_info_for_clip](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4221-4241) | 4221-4240 | [src/state/generative.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/generative.rs) |
| [input_value_as_string](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4242-4250) | 4242-4249 | [src/state/providers.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/providers.rs) |
| [input_value_as_i64](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4251-4257) | 4251-4256 | [src/state/providers.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/providers.rs) |
| [input_value_as_f64](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4258-4264) | 4258-4263 | [src/state/providers.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/providers.rs) |
| [input_value_as_bool](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4265-4272) | 4265-4271 | [src/state/providers.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/providers.rs) |
| [list_global_provider_files](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4273-4294) | 4273-4293 | [src/core/provider_store.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/provider_store.rs) |
| [read_provider_file](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4295-4298) | 4295-4297 | [src/core/provider_store.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/provider_store.rs) |
| [write_provider_file](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4299-4306) | 4299-4305 | [src/core/provider_store.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/provider_store.rs) |
| [provider_path_for_entry](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4307-4310) | 4307-4309 | [src/core/provider_store.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/provider_store.rs) |
| [default_provider_entry](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4311-4323) | 4311-4322 | [src/core/provider_store.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/provider_store.rs) |
| [timeline_zoom_bounds](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4324-4331) | 4324-4330 | `src/timeline/mod.rs` |
| [parse_f32_input](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4332-4339) | 4332-4338 | [src/utils.rs](file:///c:/repos/nla-ai-videocreator-rust/src/utils.rs) |
| [parse_f64_input](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4340-4347) | 4340-4346 | [src/utils.rs](file:///c:/repos/nla-ai-videocreator-rust/src/utils.rs) |
| [parse_i64_input](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4348-4355) | 4348-4354 | [src/utils.rs](file:///c:/repos/nla-ai-videocreator-rust/src/utils.rs) |
| [parse_version_index](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4356-4361) | 4356-4360 | [src/state/generative.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/generative.rs) |
| [delete_generative_version_files](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4362-4384) | 4362-4383 | [src/state/generative.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/generative.rs) |
| [asset_display_name](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4385-4393) | 4385-4392 | [src/state/asset.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/asset.rs) |
| [next_generative_index](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4394-4413) | 4394-4412 | [src/state/generative.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/generative.rs) |
| [update_clip_transform](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs#4414-4423) | 4414-4422 | [src/state/project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs) |

---

### Step 1.12: Extract JavaScript Snippets

**Action**: Move embedded JS to separate files or a dedicated module.

**Source**: [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) lines 70-212  
- `PREVIEW_CANVAS_SCRIPT` (~55 lines)
- `PREVIEW_NATIVE_HOST_SCRIPT` (~50 lines)
- `TIMELINE_VIEWPORT_SCRIPT` (~35 lines)

**Target**: `src/scripts.rs` (NEW) or keep inline but document clearly

**Rationale**: These are tightly coupled to the preview/timeline implementation. Keeping them as module-level constants is acceptable, but they should be near their consumers or in a dedicated scripts module.

---

## Phase 2: Split [timeline.rs](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs)

> **Priority: HIGH**  
> **Current size**: 1,218 lines  
> **Target**: 5-7 files, each < 250 lines

### Step 2.1: Create Timeline Module Directory

**Action**: Convert [src/timeline.rs](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs) to `src/timeline/` directory.

**Create structure**:
```
src/timeline/
├── mod.rs
├── panel.rs
├── ruler.rs
├── track_label.rs
├── track_row.rs
├── clip_element.rs
└── playback_controls.rs
```

---

### Step 2.2: Create `src/timeline/mod.rs`

**Content**:
```rust
//! Timeline components
//!
//! This module contains the timeline panel and related components:
//! - TimelinePanel: Main timeline container with header and tracks
//! - TimeRuler: Time ruler with tick marks
//! - TrackLabel: Track label in the sidebar
//! - TrackRow: Track row content area
//! - ClipElement: Interactive clip element

mod panel;
mod ruler;
mod track_label;
mod track_row;
mod clip_element;
mod playback_controls;

pub use panel::TimelinePanel;
pub use ruler::TimeRuler;
pub use track_label::TrackLabel;
pub use track_row::TrackRow;
pub use clip_element::ClipElement;
pub use playback_controls::PlaybackBtn;

// Timeline constants
pub const THUMB_TILE_WIDTH_PX: f64 = 60.0;
pub const MAX_THUMB_TILES: usize = 120;
pub const MIN_CLIP_WIDTH_PX: f64 = 20.0;
pub const MIN_CLIP_WIDTH_FLOOR_PX: f64 = 2.0;
pub const MIN_CLIP_WIDTH_SCALE: f64 = 0.2;
```

---

### Step 2.3: Extract [TimelinePanel](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#27-483)

**Source**: [src/timeline.rs](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs) lines 27-482  
**Target**: `src/timeline/panel.rs` (NEW)

**Contains**:
- [TimelinePanel](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#27-483) component (~455 lines)

---

### Step 2.4: Extract [TimeRuler](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#484-622)

**Source**: [src/timeline.rs](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs) lines 484-621  
**Target**: `src/timeline/ruler.rs` (NEW)

**Contains**:
- [TimeRuler](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#484-622) component (~137 lines)

---

### Step 2.5: Extract [PlaybackBtn](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#623-640)

**Source**: [src/timeline.rs](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs) lines 623-639  
**Target**: `src/timeline/playback_controls.rs` (NEW)

**Contains**:
- [PlaybackBtn](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#623-640) component (~16 lines)

---

### Step 2.6: Extract [TrackLabel](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#641-666)

**Source**: [src/timeline.rs](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs) lines 641-665  
**Target**: `src/timeline/track_label.rs` (NEW)

**Contains**:
- [TrackLabel](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#641-666) component (~24 lines)

---

### Step 2.7: Extract [TrackRow](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#666-766)

**Source**: [src/timeline.rs](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs) lines 666-765  
**Target**: `src/timeline/track_row.rs` (NEW)

**Contains**:
- [TrackRow](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#666-766) component (~99 lines)

---

### Step 2.8: Extract [ClipElement](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#767-1218)

**Source**: [src/timeline.rs](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs) lines 767-1217  
**Target**: `src/timeline/clip_element.rs` (NEW)

**Contains**:
- [ClipElement](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#767-1218) component (~450 lines)

**Note**: This is the largest timeline component. Consider further splitting if needed:
- Drag/resize logic
- Context menu logic
- Thumbnail rendering logic

---

## Phase 3: Split [state/project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs)

> **Priority: MEDIUM**  
> **Current size**: 832 lines  
> **Target**: 5-6 files, each focused on one concern

### Step 3.1: Create Project State Directory

**Action**: Convert [src/state/project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs) to `src/state/project/` directory.

**Create structure**:
```
src/state/project/
├── mod.rs
├── project.rs
├── track.rs
├── clip.rs
├── marker.rs
├── settings.rs
└── persistence.rs
```

---

### Step 3.2: Create `src/state/project/mod.rs`

**Content**:
```rust
//! Project data model
//!
//! This module contains the core data structures for a video project:
//! - Project: The top-level container
//! - Track: Timeline tracks
//! - Clip: Media clips on tracks
//! - Marker: Point-in-time annotations
//! - ProjectSettings: Project-level settings

mod project;
mod track;
mod clip;
mod marker;
mod settings;
mod persistence;

pub use project::Project;
pub use track::{Track, TrackType};
pub use clip::{Clip, ClipTransform};
pub use marker::Marker;
pub use settings::ProjectSettings;
```

---

### Step 3.3: Extract [ProjectSettings](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#23-34)

**Source**: [src/state/project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs) lines 23-47  
**Target**: `src/state/project/settings.rs` (NEW)

**Contains**:
- [ProjectSettings](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#23-34) struct
- [default_project_duration_seconds](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#35-38) function
- `Default` impl

---

### Step 3.4: Extract [Track](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#67-75) and `TrackType`

**Source**: [src/state/project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs) lines 48-99  
**Target**: `src/state/project/track.rs` (NEW)

**Contains**:
- `TrackType` enum
- [Track](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#67-75) struct
- [Track](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#67-75) impl methods ([new](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#103-117), [default_video](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#86-90), [default_audio](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#91-95), [markers](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#96-100))

---

### Step 3.5: Extract [Clip](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#138-159) and [ClipTransform](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#108-122)

**Source**: [src/state/project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs) lines 100-185  
**Target**: `src/state/project/clip.rs` (NEW)

**Contains**:
- [ClipTransform](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#108-122) struct
- [Clip](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#138-159) struct
- Impl methods ([new](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#103-117), [end_time](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#176-180), [overlaps](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#181-186))

---

### Step 3.6: Extract [Marker](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#194-204)

**Source**: [src/state/project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs) lines 186-226  
**Target**: `src/state/project/marker.rs` (NEW)

**Contains**:
- [Marker](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#194-204) struct
- Impl methods ([new](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#103-117), [with_label](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#217-227))

---

### Step 3.7: Extract Persistence Methods

**Source**: [src/state/project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs) lines 650-783  
**Target**: `src/state/project/persistence.rs` (NEW)

**Contains**:
- `Project::save`
- `Project::save_to`
- `Project::load`
- `Project::create_in`
- `Project::create_in_with_settings`
- `Project::save_project_as`
- `Project::set_generative_active_version`
- `Project::sync_generative_configs`

**Approach**: Implement these as extension methods or keep in [project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs) with file I/O helpers extracted. Consider a `ProjectIO` struct or free functions that take `&Project`.

---

### Step 3.8: Refactor [Project](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#235-255) Core

**Source**: [src/state/project.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs) lines 227-649  
**Target**: `src/state/project/project.rs` (NEW)

**Contains**:
- [Project](file:///c:/repos/nla-ai-videocreator-rust/src/state/project.rs#235-255) struct definition
- Core methods (find, add, remove, etc.)
- Methods are organized by section comments (Track operations, Clip operations, etc.)

---

## Phase 4: Split [core/preview.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs)

> **Priority: MEDIUM**  
> **Current size**: 1,242 lines  
> **Target**: 5 files

### Step 4.1: Create Preview Module Directory

**Action**: Convert [src/core/preview.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs) to `src/core/preview/` directory.

**Create structure**:
```
src/core/preview/
├── mod.rs
├── renderer.rs
├── cache.rs
├── layers.rs
├── types.rs
└── utils.rs
```

---

### Step 4.2: Create `src/core/preview/mod.rs`

**Content**:
```rust
//! Preview rendering system
//!
//! Generates composited preview frames for the current timeline time.

mod renderer;
mod cache;
mod layers;
mod types;
mod utils;

pub use renderer::PreviewRenderer;
pub use cache::{FrameCache, CachedFrame};
pub use layers::{PreviewLayer, PreviewLayerStack, PreviewLayerGpu};
pub use types::*;
pub use utils::*;
```

---

### Step 4.3: Extract Types

**Source**: [src/core/preview.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs) lines 23-111  
**Target**: `src/core/preview/types.rs` (NEW)

**Contains**:
- [PreviewStats](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#23-41)
- `PreviewDecodeMode`
- [PreviewFrameInfo](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#49-54)
- [PreviewLayerPlacement](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#56-64)
- [PreviewLayerGpu](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#66-70)
- [PreviewLayerStack](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#72-77)
- [RenderOutput](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#79-84)
- [FrameKey](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#86-90)
- [CachedFrame](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#92-97)
- [CacheEntry](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#98-105)
- [PlateCache](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#106-112)

---

### Step 4.4: Extract [FrameCache](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#113-121)

**Source**: [src/core/preview.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs) lines 113-230  
**Target**: `src/core/preview/cache.rs` (NEW)

**Contains**:
- [FrameCache](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#113-121) struct
- All [FrameCache](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#113-121) impl methods

---

### Step 4.5: Extract [PreviewRenderer](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#234-242)

**Source**: [src/core/preview.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs) lines 232-914  
**Target**: `src/core/preview/renderer.rs` (NEW)

**Contains**:
- [PreviewRenderer](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#234-242) struct (~680 lines)
- All render methods

**Note**: This is still large. Consider further splitting render methods into:
- `renderer.rs` — struct + primary entry points
- `render_frame.rs` — [render_frame](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#265-350), [render_layers](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#856-991)
- `render_layer.rs` — [collect_layers](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#452-631), [load_clip_frame](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#763-866)

---

### Step 4.6: Extract Layer/Compositing Helpers

**Source**: [src/core/preview.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs) lines 915-1172  
**Target**: `src/core/preview/layers.rs` (NEW)

**Contains**:
- [PendingDecode](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#917-926)
- [DecodedFrame](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#927-932)
- [PreviewLayer](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#933-941)
- [preview_canvas_size](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#942-961)
- [composite_layer](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#962-1014)
- [rotate_rgba](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1015-1037)
- [compute_layer_placement](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1038-1084)
- [apply_opacity](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1085-1091)
- [draw_border](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1147-1173)

---

### Step 4.7: Extract Utility Functions

**Source**: [src/core/preview.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs) lines 1092-1241  
**Target**: `src/core/preview/utils.rs` (NEW)

**Contains**:
- [clamp_time](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1092-1100)
- [elapsed_ms](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1101-1104)
- [time_to_frame_index](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1105-1109)
- [frame_index_to_time](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1110-1115)
- [track_lane_id](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1116-1120)
- [image_size_bytes](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1121-1128)
- [scale_image_to_fit](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1129-1146)
- [resolve_generative_path](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1174-1205)
- [resolve_asset_source](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview.rs#1206-1242)

---

## Phase 5: Split [core/preview_gpu.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs)

> **Priority: MEDIUM**  
> **Current size**: 1,039 lines  
> **Target**: 4 files

### Step 5.1: Create Preview GPU Module Directory

**Action**: Convert [src/core/preview_gpu.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs) to `src/core/preview_gpu/` directory.

**Create structure**:
```
src/core/preview_gpu/
├── mod.rs
├── surface.rs
├── shaders.rs
├── types.rs
└── layers.rs
```

---

### Step 5.2: Create `src/core/preview_gpu/mod.rs`

**Content**:
```rust
//! GPU-accelerated preview rendering
//!
//! Uses wgpu for hardware-accelerated compositing.

mod surface;
mod shaders;
mod types;
mod layers;

pub use surface::PreviewGpuSurface;
pub use types::PreviewBounds;
```

---

### Step 5.3: Extract Shaders

**Source**: [src/core/preview_gpu.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs) lines 118-210  
**Target**: `src/core/preview_gpu/shaders.rs` (NEW)

**Contains**:
- `PREVIEW_SHADER` const
- `BORDER_SHADER` const
- `PREVIEW_CLEAR_COLOR` const
- `BORDER_COLOR_LINEAR` const

---

### Step 5.4: Extract Types

**Source**: [src/core/preview_gpu.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs) lines 38-116, 212-257  
**Target**: `src/core/preview_gpu/types.rs` (NEW)

**Contains**:
- [Vertex](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#38-42) struct + impl
- `QUAD_VERTICES` const
- [LayerUniform](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#96-100) struct + impl
- [BorderUniform](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#212-216) struct
- [PreviewBounds](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#224-231) struct + impl
- [GpuLayer](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#250-258) struct

---

### Step 5.5: Extract [PreviewGpuSurface](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#260-284)

**Source**: [src/core/preview_gpu.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs) lines 260-1004  
**Target**: `src/core/preview_gpu/surface.rs` (NEW)

**Contains**:
- [PreviewGpuSurface](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#260-284) struct
- All impl methods

**Note**: This is a large struct. Consider splitting:
- `surface.rs` — struct definition + initialization
- `render.rs` — [render_layers](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#856-991) method
- `upload.rs` — [upload_layers](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#1031-1034) method

---

### Step 5.6: Extract Layer Management

**Source**: Parts of [PreviewGpuSurface](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#260-284) impl  
**Target**: `src/core/preview_gpu/layers.rs` (NEW)

**Contains**:
- [create_layer_texture](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#567-590) (extracted as free function)
- [create_layer](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#591-647) (extracted as free function)
- [compute_layer_uniform](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#648-697) (extracted as free function)
- [align_to](file:///c:/repos/nla-ai-videocreator-rust/src/core/preview_gpu.rs#1007-1014) utility function

---

## Phase 6: Update Module Exports

> **Priority: HIGH** (Do this as you complete each phase)

### Step 6.1: Update [src/main.rs](file:///c:/repos/nla-ai-videocreator-rust/src/main.rs)

**Add module declarations**:
```rust
mod constants;
mod components;
// timeline is already declared, just update path
```

---

### Step 6.2: Update [src/state/mod.rs](file:///c:/repos/nla-ai-videocreator-rust/src/state/mod.rs)

**If using project subdirectory**:
```rust
mod project;
mod asset;
mod selection;
mod providers;
mod generative;

pub use project::*;
pub use asset::*;
pub use selection::*;
pub use providers::*;
pub use generative::*;
```

---

### Step 6.3: Update [src/core/mod.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/mod.rs)

**If using preview subdirectories**:
```rust
pub mod thumbnailer;
pub mod media;
pub mod preview;
pub mod preview_store;
pub mod preview_gpu;
pub mod provider_store;
pub mod generation;
mod video_decode;
```

---

## Execution Order (Recommended)

Execute phases in this order to minimize breakage and enable incremental testing:

### Week 1: Foundation
1. **Step 1.1** — Extract constants (low risk, immediate benefit)
2. **Step 1.2** — Create component directory structure
3. **Step 1.3-1.7** — Extract small, independent components (StatusBar, TitleBar, SidePanel, PreviewPanel)
4. **Step 1.12** — Document JS snippets

### Week 2: Heavy Lifting — app.rs
5. **Step 1.8** — Extract field components
6. **Step 1.9** — Extract AttributesPanelContent
7. **Step 1.10** — Extract Assets panel components
8. **Step 1.11** — Extract utility functions
9. **Run `cargo check`** — fix any import issues

### Week 3: Timeline
10. **Steps 2.1-2.8** — Split timeline.rs into module

### Week 4: State & Core
11. **Steps 3.1-3.8** — Split state/project.rs
12. **Steps 4.1-4.7** — Split core/preview.rs
13. **Steps 5.1-5.6** — Split core/preview_gpu.rs
14. **Step 6** — Final export updates

### Ongoing
- After each major phase, run `cargo check` and `cargo test`
- Update any relative imports in [app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) as components are extracted

---

## AI Agent Guidelines

### When Working on Refactored Codebase

1. **Start with [mod.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/mod.rs) files** — These are your navigation maps
2. **One file = one concern** — If you need multiple unrelated things, you're in the wrong file
3. **Check re-exports** — If a type isn't where you expect, check the parent [mod.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/mod.rs)
4. **Constants live in `src/constants.rs`** — Don't hunt for color values

### File Loading Strategy

For typical tasks:
- **UI changes**: Load only the specific component file + `constants.rs`
- **State changes**: Load `state/{feature}/mod.rs` + specific file
- **Preview changes**: Load `core/preview/mod.rs` + specific file

### Import Patterns

After refactoring, prefer:
```rust
// Good — clear where things come from
use crate::components::assets::AssetsPanelContent;
use crate::timeline::TimelinePanel;
use crate::state::project::{Project, Track, Clip};
use crate::constants::*;

// Avoid — loses context
use crate::*;
```

---

## Verification Checklist

After completing all phases:

- [ ] `cargo check` passes with no errors
- [ ] `cargo test` passes all existing tests
- [ ] Each [mod.rs](file:///c:/repos/nla-ai-videocreator-rust/src/core/mod.rs) has a module-level doc comment
- [ ] No file exceeds 500 lines (except complex components like [ClipElement](file:///c:/repos/nla-ai-videocreator-rust/src/timeline.rs#767-1218))
- [ ] No circular dependencies between modules
- [ ] All color/UI constants are in `constants.rs`
- [ ] [app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) is under 400 lines

---

## Appendix: File Size Targets

| File/Module | Current Lines | Target Lines |
|-------------|---------------|--------------|
| [src/app.rs](file:///c:/repos/nla-ai-videocreator-rust/src/app.rs) | 4,870 | < 400 |
| `src/timeline/panel.rs` | — | < 500 |
| `src/timeline/clip_element.rs` | — | < 500 |
| `src/components/*/` | — | < 300 each |
| `src/state/project/*.rs` | — | < 200 each |
| `src/core/preview/*.rs` | — | < 300 each |
| `src/core/preview_gpu/*.rs` | — | < 250 each |

---

*Document generated: 2026-01-08*

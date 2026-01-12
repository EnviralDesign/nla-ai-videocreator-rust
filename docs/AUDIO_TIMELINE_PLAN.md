# Audio Timeline Implementation Plan (Rust)

## Goals (MVP)
- Import audio assets and place them on audio tracks (already in data model).
- Waveform visualization on audio clips.
- Low-latency playback and scrubbing tied to the timeline playhead.
- Audio duration probes, caching, and reuse across sessions.

## Current Codebase Surface Area
- Audio assets/tracks/clips already exist in state (`src/state/asset.rs`, `src/state/project/*`).
- Duration probing uses `ffprobe` for audio/video (`src/core/media.rs`).
- Timeline clip rendering is visual-only; no waveform layer (`src/timeline/clip_element.rs`).
- Thumbnail cache is image/video only (`src/core/thumbnailer.rs`).
- Preview renderer ignores audio (`src/core/preview/renderer.rs`).

## Proposed Architecture (Codebase-Aware)

### Core Modules
- Add `src/core/audio/` with submodules:
  - `decode.rs`: decode audio to PCM (f32) from file.
  - `resample.rs`: resample to engine output format.
  - `playback.rs`: audio output stream + mixer.
  - `waveform.rs`: peak extraction + cache IO.
  - `cache.rs`: cache metadata and invalidation.
- Keep UI-specific logic in `src/components/` and `src/timeline/`.

### Audio Engine Output Format
- Standardize on `48kHz`, stereo, interleaved `f32`.
- Rationale: video-centric default and stable device compatibility.

### Decode + Resample
- Use `ffmpeg-next` for decode and resample (already in repo).
- Seek by timestamp; flush decoder; decode and drop samples until exact position.
- For non-FFmpeg fallback, consider `symphonia` later (not MVP).

### Playback + Sync
- Use `cpal` for low-latency output and full control.
- Audio callback mixes active clips based on a shared atomic sample counter.
- UI playhead reads the audio clock (sample counter / sample rate).
- Scrubbing:
  - On drag, reset the engine to the new timeline time.
  - Decode a short buffer (100-250ms) for immediate feedback.

### Waveform Generation + Cache
- Compute peak envelope (min/max) per block.
- Store multi-resolution "mip" levels for zooming.
- Cache format (project-local): `.cache/audio/peaks/<asset_id>.peaks`
  - Header includes: version, file size, mtime, sample rate, channels, block sizes.
  - Invalidate on size/mtime/version mismatch.
- Build peaks on import and on-demand (lazy + background).

### Timeline Rendering
- For audio clips, render a waveform overlay in `ClipElement`.
- Use a per-clip canvas layer and draw from peak data for the visible range.
- On zoom change, redraw from the appropriate mip level (no re-decode).

### Export (Future)
- Offline mix using the same engine (no real-time constraints).
- Optionally use ffmpeg to encode/mux after mixdown.

## Phased Delivery
1. Audio engine v1: decode + playback + audio clock.
2. Waveform v1: peak cache + canvas drawing.
3. Scrubbing polish: short-buffer preview while dragging.
4. Export mixdown path (offline render).
5. Optional: beat detection markers.

## Decisions (Proposed)
- Decode/resample: `ffmpeg-next` (extend existing dependency).
- Playback: `cpal` (low-latency callback control).
- Waveform cache: project-local peak files with multi-res blocks.

## Open Questions / Risks
- Cache location consistency: project-local vs app cache (docs say project-local).
- AAC licensing risk if we decode via FFmpeg (same as current video decode).
- Performance on very long audio files: peak build should be backgrounded.

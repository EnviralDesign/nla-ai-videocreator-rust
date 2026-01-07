//! Root application component
//! 
//! This defines the main App component and the overall layout structure.

use dioxus::desktop::{use_window, use_wry_event_handler};
use dioxus::desktop::tao::event::{Event as TaoEvent, WindowEvent as TaoWindowEvent};
use dioxus::prelude::*;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};
use crate::core::preview_gpu::{PreviewBounds, PreviewGpuSurface};
use crate::timeline::TimelinePanel;

// =============================================================================
// COLOR SCHEME - Charcoal Monochrome with Functional Accents
// =============================================================================
pub const BG_DEEPEST: &str = "#09090b";
pub const BG_BASE: &str = "#0a0a0b";
pub const BG_ELEVATED: &str = "#141414";
pub const BG_SURFACE: &str = "#1a1a1a";
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

// Panel dimensions
const PANEL_MIN_WIDTH: f64 = 180.0;
const PANEL_MAX_WIDTH: f64 = 400.0;
const PANEL_DEFAULT_WIDTH: f64 = 250.0;
const PANEL_COLLAPSED_WIDTH: f64 = 40.0;
const TIMELINE_MIN_HEIGHT: f64 = 100.0;
const TIMELINE_MAX_HEIGHT: f64 = 500.0;
const TIMELINE_DEFAULT_HEIGHT: f64 = 220.0;
const TIMELINE_COLLAPSED_HEIGHT: f64 = 32.0;  // Must match header height exactly
const DEFAULT_CLIP_DURATION_SECONDS: f64 = 2.0;
const PREVIEW_FPS: u64 = 24;
const PREVIEW_FRAME_INTERVAL_MS: u64 = 1000 / PREVIEW_FPS;
const PREVIEW_CACHE_BUDGET_BYTES: usize = 8usize * 1024 * 1024 * 1024;
const PREVIEW_PREFETCH_SCRUB_SECONDS: f64 = 0.5;
const PREVIEW_PREFETCH_PLAYBACK_SECONDS: f64 = 3.0;
const SHOW_CACHE_TICKS: bool = false;
const PREVIEW_CANVAS_SCRIPT: &str = r#"
let canvas = null;
let ctx = null;

function getCanvas() {
    if (!canvas || !document.body.contains(canvas)) {
        canvas = document.getElementById("preview-canvas");
        ctx = canvas ? canvas.getContext("2d") : null;
    }
    return { canvas, ctx };
}

while (true) {
    const msg = await dioxus.recv();
    if (!msg) {
        continue;
    }
    if (msg.kind === "clear") {
        const state = getCanvas();
        if (state.ctx && state.canvas) {
            state.ctx.clearRect(0, 0, state.canvas.width, state.canvas.height);
        }
        continue;
    }
    if (msg.kind !== "frame") {
        continue;
    }

    const version = msg.version;
    const width = msg.width;
    const height = msg.height;

    const state = getCanvas();
    if (!state.ctx || !state.canvas) {
        continue;
    }

    if (state.canvas.width !== width || state.canvas.height !== height) {
        state.canvas.width = width;
        state.canvas.height = height;
    }

    try {
        const response = await fetch("http://nla.localhost/preview/raw/" + version);
        if (!response.ok) {
            continue;
        }
        const buffer = await response.arrayBuffer();
        if (buffer.byteLength !== width * height * 4) {
            continue;
        }
        const imageData = new ImageData(new Uint8ClampedArray(buffer), width, height);
        state.ctx.putImageData(imageData, 0, 0);
    } catch (_) {
        // Ignore transient decode or fetch errors.
    }
}
"#;

const PREVIEW_NATIVE_HOST_SCRIPT: &str = r#"
const hostId = "preview-native-host";
let last = null;

function sendBounds() {
    const host = document.getElementById(hostId);
    if (!host) {
        return;
    }
    const rect = host.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    const next = {
        x: rect.left,
        y: rect.top,
        width: rect.width,
        height: rect.height,
        dpr: dpr
    };
    if (last &&
        Math.abs(last.x - next.x) < 0.5 &&
        Math.abs(last.y - next.y) < 0.5 &&
        Math.abs(last.width - next.width) < 0.5 &&
        Math.abs(last.height - next.height) < 0.5 &&
        Math.abs(last.dpr - next.dpr) < 0.01) {
        return;
    }
    last = next;
    dioxus.send(next);
}

function attach() {
    const host = document.getElementById(hostId);
    if (!host) {
        setTimeout(attach, 100);
        return;
    }
    const observer = new ResizeObserver(() => sendBounds());
    observer.observe(host);
    window.addEventListener("resize", sendBounds, { passive: true });
    window.addEventListener("scroll", sendBounds, { passive: true });
    sendBounds();
}

attach();
await new Promise(() => {});
"#;

#[derive(Clone, Copy, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PreviewCanvasMessage {
    Frame { version: u64, width: u32, height: u32 },
    Clear,
}

/// Main application component
#[component]
pub fn App() -> Element {
    // Project state - the core data model
    let mut project = use_signal(|| crate::state::Project::default());
    
    // Core services
    let mut thumbnailer = use_signal(|| std::sync::Arc::new(crate::core::thumbnailer::Thumbnailer::new(std::path::PathBuf::from("projects/default")))); // Temporary default path, updated on load
    let thumbnail_refresh_tick = use_signal(|| 0_u64);
    let thumbnail_cache_buster = use_signal(|| 0_u64);
    let mut previewer = use_signal(|| std::sync::Arc::new(
        crate::core::preview::PreviewRenderer::new(
            std::path::PathBuf::from("projects/default"),
            PREVIEW_CACHE_BUDGET_BYTES,
        ),
    ));
    let preview_frame = use_signal(|| None::<crate::core::preview::PreviewFrameInfo>);
    let preview_stats = use_signal(|| None::<crate::core::preview::PreviewStats>);
    let mut preview_eval = use_signal(|| None::<document::Eval>);
    let mut preview_host_eval = use_signal(|| None::<document::Eval>);
    let preview_native_bounds = use_signal(|| None::<PreviewBounds>);
    let mut preview_native_active = use_signal(|| false);
    let mut preview_native_enabled = use_signal(|| false);
    let preview_native_attempted = use_signal(|| false);
    let mut preview_native_uploaded = use_signal(|| None::<u64>);
    let mut preview_gpu_upload_ms = use_signal(|| None::<f64>);
    let mut preview_layers =
        use_signal(|| None::<(u64, crate::core::preview::PreviewLayerStack)>);
    let mut preview_native_ready = use_signal(|| false);
    let preview_gpu = use_hook(|| Rc::new(RefCell::new(None::<PreviewGpuSurface>)));
    let mut show_preview_stats = use_signal(|| false);
    let mut use_hw_decode = use_signal(|| true);
    let mut clip_cache_buckets =
        use_signal(|| std::sync::Arc::new(HashMap::<uuid::Uuid, Vec<bool>>::new()));
    let preview_cache_tick = use_signal(|| 0_u64);
    let desktop = use_window();
    let desktop_for_bounds = desktop.clone();
    let desktop_for_events = desktop.clone();
    let desktop_for_redraw = desktop.clone();
    let mut preview_dirty = use_signal(|| true);

    // Startup Modal state - check if we have a valid project path on load
    // For MVP, we start with a dummy project, so we check if project_path is None
    let mut startup_done = use_signal(|| false);
    
    // Panel state
    let mut left_width = use_signal(|| PANEL_DEFAULT_WIDTH);
    let mut left_collapsed = use_signal(|| false);
    let mut right_width = use_signal(|| PANEL_DEFAULT_WIDTH);
    let mut right_collapsed = use_signal(|| false);
    let mut timeline_height = use_signal(|| TIMELINE_DEFAULT_HEIGHT);
    let mut timeline_collapsed = use_signal(|| false);
    
    // Timeline playback state
    let mut current_time = use_signal(|| 0.0_f64);        // Current time in seconds
    let mut zoom = use_signal(|| 100.0_f64);              // Pixels per second
    let mut is_playing = use_signal(|| false);            // Playback state
    let mut scroll_offset = use_signal(|| 0.0_f64);       // Horizontal scroll position
    
    // Derive duration from project
    let duration = project.read().duration();
    
    // Drag state
    let mut dragging = use_signal(|| None::<&'static str>);
    let mut drag_start_pos = use_signal(|| 0.0);
    let mut drag_start_size = use_signal(|| 0.0);
    
    // Asset Drag & Drop state
    let mut dragged_asset = use_signal(|| None::<uuid::Uuid>);
    let mut mouse_pos = use_signal(|| (0.0, 0.0));
    let mut selection = use_signal(|| crate::state::SelectionState::default());
    
    // Context menu state: (x, y, track_id) - None means no menu shown
    let mut context_menu = use_signal(|| None::<(f64, f64, uuid::Uuid)>);

    use_future(move || {
        let mut thumbnail_refresh_tick = thumbnail_refresh_tick.clone();
        async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
            loop {
                interval.tick().await;
                thumbnail_refresh_tick.set(thumbnail_refresh_tick() + 1);
            }
        }
    });

    use_future(move || {
        let mut current_time = current_time.clone();
        let mut is_playing = is_playing.clone();
        let project = project.clone();
        async move {
            let mut last_tick = Instant::now();
            loop {
                tokio::time::sleep(Duration::from_millis(16)).await;
                if !is_playing() {
                    last_tick = Instant::now();
                    continue;
                }

                let now = Instant::now();
                let delta = now.saturating_duration_since(last_tick);
                last_tick = now;

                let duration = project.read().duration();
                let next_time = (current_time() + delta.as_secs_f64()).min(duration);
                let snapped = (next_time * 60.0).round() / 60.0;
                current_time.set(snapped);

                if next_time >= duration {
                    is_playing.set(false);
                }
            }
        }
    });

    use_future(move || {
        let project = project.clone();
        let current_time = current_time.clone();
        let is_playing = is_playing.clone();
        let previewer = previewer.clone();
        let mut preview_frame = preview_frame.clone();
        let mut preview_layers = preview_layers.clone();
        let mut preview_stats = preview_stats.clone();
        let mut preview_dirty = preview_dirty.clone();
        let mut preview_cache_tick = preview_cache_tick.clone();
        let preview_native_ready = preview_native_ready.clone();
        let use_hw_decode = use_hw_decode.clone();
        async move {
            let render_request_id = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
            let render_gate = std::sync::Arc::new(tokio::sync::Semaphore::new(1));
            let prefetch_gate = std::sync::Arc::new(tokio::sync::Semaphore::new(1));
            let mut last_time = -1.0_f64;
            loop {
                tokio::time::sleep(Duration::from_millis(PREVIEW_FRAME_INTERVAL_MS)).await;

                let time = current_time();
                let dirty = preview_dirty();
                if !dirty && (time - last_time).abs() < 0.0001 {
                    continue;
                }

                let permit = match render_gate.clone().try_acquire_owned() {
                    Ok(permit) => permit,
                    Err(_) => continue,
                };
                let request_id = render_request_id
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    + 1;

                let project_snapshot = project.read().clone();
                let renderer = previewer.read().clone();
                let use_gpu = preview_native_ready();
                let decode_mode = if is_playing() {
                    crate::core::preview::PreviewDecodeMode::Sequential
                } else {
                    crate::core::preview::PreviewDecodeMode::Seek
                };
                let allow_hw_decode = use_hw_decode();
                let render_task = tokio::task::spawn_blocking(move || {
                    let result = if use_gpu {
                        renderer.render_layers(&project_snapshot, time, decode_mode, allow_hw_decode)
                    } else {
                        renderer.render_frame(&project_snapshot, time, decode_mode, allow_hw_decode)
                    };
                    drop(permit);
                    (result, project_snapshot, use_gpu, decode_mode, allow_hw_decode)
                })
                .await
                .ok();

                let Some((render_output, project_snapshot, use_gpu, decode_mode, allow_hw_decode)) = render_task else {
                    continue;
                };

                if render_request_id.load(std::sync::atomic::Ordering::Relaxed) != request_id {
                    continue;
                }

                let crate::core::preview::RenderOutput { frame, layers, stats } = render_output;
                preview_stats.set(Some(stats));
                if SHOW_CACHE_TICKS {
                    preview_cache_tick.set(preview_cache_tick() + 1);
                }

                let rendered = if use_gpu {
                    if let Some(layers) = layers {
                        preview_layers.set(Some((request_id, layers)));
                        preview_frame.set(None);
                        true
                    } else {
                        preview_layers.set(None);
                        preview_frame.set(None);
                        false
                    }
                } else {
                    preview_layers.set(None);
                    preview_frame.set(frame);
                    frame.is_some()
                };

                preview_dirty.set(false);
                let direction = if last_time < 0.0 {
                    0
                } else if time > last_time {
                    1
                } else if time < last_time {
                    -1
                } else {
                    0
                };
                last_time = time;

                if rendered && direction != 0 {
                    let fps = project_snapshot.settings.fps.max(1.0);
                    let prefetch_seconds = if is_playing() {
                        PREVIEW_PREFETCH_PLAYBACK_SECONDS
                    } else {
                        PREVIEW_PREFETCH_SCRUB_SECONDS
                    };
                    let prefetch_frames = (fps * prefetch_seconds).round() as u32;
                    if prefetch_frames > 0 {
                        if let Ok(prefetch_permit) = prefetch_gate.clone().try_acquire_owned() {
                            let renderer = previewer.read().clone();
                            tokio::task::spawn_blocking(move || {
                                renderer.prefetch_frames(
                                    &project_snapshot,
                                    time,
                                    direction,
                                    prefetch_frames,
                                    decode_mode,
                                    allow_hw_decode,
                                );
                                drop(prefetch_permit);
                            });
                        }
                    }
                }
            }
        }
    });

    use_effect(move || {
        let _tick = preview_cache_tick();
        if !SHOW_CACHE_TICKS {
            if !clip_cache_buckets().is_empty() {
                clip_cache_buckets.set(std::sync::Arc::new(HashMap::new()));
            }
            return;
        }
        let zoom_value = zoom().max(1.0);
        let project_snapshot = project.read().clone();
        let renderer = previewer.read().clone();
        let fps = project_snapshot.settings.fps.max(1.0);
        let bucket_hint_seconds = (6.0 / zoom_value).max(1.0 / fps);
        let cache_map = renderer.cached_buckets_for_project(&project_snapshot, bucket_hint_seconds);
        clip_cache_buckets.set(std::sync::Arc::new(cache_map));
    });

    use_effect(move || {
        if preview_eval().is_some() {
            return;
        }
        let eval = document::eval(PREVIEW_CANVAS_SCRIPT);
        preview_eval.set(Some(eval));
    });

    use_effect(move || {
        let frame = preview_frame();
        let Some(eval) = preview_eval() else {
            return;
        };
        if preview_native_active() {
            return;
        }
        let _ = match frame {
            Some(frame) => eval.send(PreviewCanvasMessage::Frame {
                version: frame.version,
                width: frame.width,
                height: frame.height,
            }),
            None => eval.send(PreviewCanvasMessage::Clear),
        };
    });

    use_effect(move || {
        if preview_host_eval().is_some() {
            return;
        }
        let eval = document::eval(PREVIEW_NATIVE_HOST_SCRIPT);
        preview_host_eval.set(Some(eval));
    });

    use_future(move || {
        let mut preview_native_bounds = preview_native_bounds.clone();
        let preview_host_eval = preview_host_eval.clone();
        let desktop = desktop_for_bounds.clone();
        async move {
            loop {
                let Some(eval) = preview_host_eval() else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                };
                let mut eval = eval;
                loop {
                    match eval.recv::<PreviewBounds>().await {
                        Ok(bounds) => {
                            if preview_native_bounds() != Some(bounds) {
                                preview_native_bounds.set(Some(bounds));
                                desktop.window.request_redraw();
                            }
                        }
                        Err(_) => break,
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    });

    use_effect(move || {
        let enabled = project.read().project_path.is_some() && startup_done();
        preview_native_enabled.set(enabled);
        if !enabled {
            preview_native_active.set(false);
            preview_native_uploaded.set(None);
            preview_gpu_upload_ms.set(None);
            preview_layers.set(None);
            preview_native_ready.set(false);
        }
    });

    let preview_layers_for_redraw = preview_layers.clone();
    use_effect(move || {
        if !preview_native_enabled() {
            return;
        }
        if preview_frame().is_some() || preview_layers_for_redraw().is_some() {
            desktop_for_redraw.window.request_redraw();
        }
    });

    use_wry_event_handler({
        let preview_gpu = preview_gpu.clone();
        let preview_native_bounds = preview_native_bounds.clone();
        let mut preview_native_active = preview_native_active.clone();
        let preview_native_enabled = preview_native_enabled.clone();
        let mut preview_native_attempted = preview_native_attempted.clone();
        let mut preview_native_uploaded = preview_native_uploaded.clone();
        let mut preview_gpu_upload_ms = preview_gpu_upload_ms.clone();
        let preview_layers = preview_layers.clone();
        let mut preview_native_ready = preview_native_ready.clone();
        let mut preview_dirty = preview_dirty.clone();
        let desktop = desktop_for_events.clone();
        move |event, target| {
            if !preview_native_enabled() {
                return;
            }
            let Some(bounds) = preview_native_bounds() else {
                return;
            };

            let is_main_window = match event {
                TaoEvent::RedrawRequested(window_id) => *window_id == desktop.window.id(),
                TaoEvent::WindowEvent { window_id, .. } => *window_id == desktop.window.id(),
                _ => false,
            };
            if !is_main_window {
                return;
            }

            let should_render = matches!(
                event,
                TaoEvent::RedrawRequested(window_id) if *window_id == desktop.window.id()
            );
            let should_update = should_render
                || matches!(
                    event,
                    TaoEvent::WindowEvent {
                        event: TaoWindowEvent::Resized(_) | TaoWindowEvent::Moved(_),
                        ..
                    }
                );
            if !should_update {
                return;
            }

            let mut gpu_state = preview_gpu.borrow_mut();
            if gpu_state.is_none() {
                if preview_native_attempted() {
                    return;
                }
                preview_native_attempted.set(true);
                if let Some(gpu) = PreviewGpuSurface::new(&desktop.window, target) {
                    *gpu_state = Some(gpu);
                    preview_native_ready.set(true);
                    preview_dirty.set(true);
                } else {
                    return;
                }
            }

            if let Some(gpu) = gpu_state.as_mut() {
                let mut uploaded = false;
                let upload_ms = if let Some((stack_id, stack)) = preview_layers() {
                    if stack.layers.is_empty() {
                        if preview_native_active() {
                            preview_native_active.set(false);
                        }
                        preview_native_uploaded.set(None);
                        gpu.clear_layers();
                        uploaded = true;
                        Some(0.0)
                    } else {
                        if !preview_native_active() {
                            preview_native_active.set(true);
                        }
                        if preview_native_uploaded() != Some(stack_id) {
                            let upload_start = Instant::now();
                            uploaded = gpu.upload_layers(&stack);
                            let elapsed_ms = upload_start.elapsed().as_secs_f64() * 1000.0;
                            if uploaded {
                                preview_native_uploaded.set(Some(stack_id));
                            }
                            Some(elapsed_ms)
                        } else {
                            Some(0.0)
                        }
                    }
                } else {
                    if preview_native_active() {
                        preview_native_active.set(false);
                    }
                    preview_native_uploaded.set(None);
                    gpu.clear_layers();
                    uploaded = true;
                    Some(0.0)
                };
                if let Some(ms) = upload_ms {
                    preview_gpu_upload_ms.set(Some(ms));
                }

                let changed = gpu.apply_bounds(bounds);
                if should_render || changed || uploaded {
                    gpu.render_layers();
                }
            }
        }
    });

    // Dialog state
    let mut show_new_project_dialog = use_signal(|| false); // Kept for "File > New" inside app
    
    // On first load, if project has no path effectively, treat as "No Project Loaded"
    // But since we initialize with default(), we need a flag to block interaction until New/Open
    // We'll use specific "show_startup_modal" derived state
    
    let show_startup = project.read().project_path.is_none() && !startup_done();

    // Read current values
    let left_w = if left_collapsed() { PANEL_COLLAPSED_WIDTH } else { left_width() };
    let right_w = if right_collapsed() { PANEL_COLLAPSED_WIDTH } else { right_width() };
    let timeline_h = if timeline_collapsed() { TIMELINE_COLLAPSED_HEIGHT } else { timeline_height() };
    
    // Is currently dragging? (for cursor and user-select styling)
    let is_dragging = dragging().is_some();
    let user_select_style = if is_dragging { "none" } else { "auto" };
    let drag_cursor = match dragging() {
        Some("left") | Some("right") => "ew-resize",
        Some("timeline") => "ns-resize",
        Some("playhead") => "ew-resize",
        _ => "default",
    };
    
    // Ghost asset for drag and drop
    let drag_ghost_asset = if let Some(id) = dragged_asset() {
        project.read().assets.iter().find(|a| a.id == id).cloned()
    } else {
        None
    };
    
    // Which panel is currently being resized? (to disable transitions during drag)
    let left_resizing = dragging() == Some("left");
    let right_resizing = dragging() == Some("right");
    let timeline_resizing = dragging() == Some("timeline");

    rsx! {
        // Global CSS with drag state handling
        style {
            r#"
            *, *::before, *::after {{ box-sizing: border-box; }}
            html, body {{ margin: 0; padding: 0; overflow: hidden; background-color: {BG_BASE}; }}
            body {{ -webkit-font-smoothing: antialiased; }}
            ::-webkit-scrollbar {{ width: 6px; height: 6px; }}
            ::-webkit-scrollbar-track {{ background: transparent; }}
            ::-webkit-scrollbar-thumb {{ background: {BORDER_DEFAULT}; border-radius: 3px; }}
            ::-webkit-scrollbar-thumb:hover {{ background: {BORDER_STRONG}; }}
            .collapse-btn {{ opacity: 0.6; transition: opacity 0.15s ease, background-color 0.15s ease; }}
            .collapse-btn:hover {{ opacity: 1; background-color: {BG_HOVER} !important; }}
            .resize-handle {{ transition: background-color 0.15s ease; }}
            .resize-handle:hover {{ background-color: {BORDER_ACCENT} !important; }}
            .resize-handle:active {{ background-color: {BORDER_ACCENT} !important; }}
            .collapsed-rail {{ transition: background-color 0.15s ease; }}
            .collapsed-rail:hover {{ background-color: {BG_HOVER} !important; }}
            .resize-handle-left:hover > div, .resize-handle-right:hover > div {{ opacity: 1 !important; }}
            "#
        }

        // Main app container
        div {
            class: "app-container",
            style: "
                display: flex; flex-direction: column;
                width: 100vw; height: 100vh;
                background-color: {BG_BASE}; color: {TEXT_PRIMARY};
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
                overflow: hidden; position: fixed; top: 0; left: 0;
                user-select: {user_select_style};
                cursor: {drag_cursor};
            ",
            
            onmousemove: move |e| {
                if dragged_asset().is_some() {
                    mouse_pos.set((e.client_coordinates().x, e.client_coordinates().y));
                }
                
                if let Some(target) = dragging() {
                    e.prevent_default();
                    match target {
                        "left" => {
                            let delta = e.client_coordinates().x - drag_start_pos();
                            let new_w = (drag_start_size() + delta).clamp(PANEL_MIN_WIDTH, PANEL_MAX_WIDTH);
                            left_width.set(new_w);
                        }
                        "right" => {
                            let delta = drag_start_pos() - e.client_coordinates().x;
                            let new_w = (drag_start_size() + delta).clamp(PANEL_MIN_WIDTH, PANEL_MAX_WIDTH);
                            right_width.set(new_w);
                        }
                        "timeline" => {
                            let delta = drag_start_pos() - e.client_coordinates().y;
                            let new_h = (drag_start_size() + delta).clamp(TIMELINE_MIN_HEIGHT, TIMELINE_MAX_HEIGHT);
                            timeline_height.set(new_h);
                        }
                        "playhead" => {
                            // Convert mouse x delta to time delta using zoom factor
                            let delta_px = e.client_coordinates().x - drag_start_pos();
                            let delta_time = delta_px / zoom();
                            let raw_time = (drag_start_size() + delta_time).clamp(0.0, duration);
                            // Snap to frame boundary (60fps)
                            let snapped_time = (raw_time * 60.0).round() / 60.0;
                            current_time.set(snapped_time);
                        }
                        _ => {}
                    }
                }
            },
            onmouseup: move |_| {
                dragging.set(None);
                dragged_asset.set(None);
            },
            // Suppress the browser's default context menu - we'll use custom menus
            oncontextmenu: move |e| e.prevent_default(),
            // Note: We intentionally don't clear drag on mouseleave so drag continues
            // if the user moves outside the window and back in while still holding mouse button

            // Drag Ghost
            if let Some(asset) = drag_ghost_asset {
                div {
                    style: "
                        position: fixed; left: {mouse_pos().0 + 15.0}px; top: {mouse_pos().1 + 15.0}px;
                        background-color: {BG_ELEVATED}; border: 1px solid {ACCENT_VIDEO};
                        border-radius: 4px; padding: 6px 10px; font-size: 12px; pointer-events: none;
                        z-index: 10000; box-shadow: 0 4px 12px rgba(0,0,0,0.3); opacity: 0.9;
                        color: {TEXT_PRIMARY}; display: flex; align-items: center; gap: 6px;
                    ",
                    span { "📄" } // Generic icon for now
                    "{asset.name}"
                }
            }

            TitleBar { 
                project_name: project.read().name.clone(),
                on_new_project: move |_| {
                    show_new_project_dialog.set(true);
                },
                on_save: move |_| {
                     // Since project knows its own path (if loaded/saved once), we can just save
                     // If it's effectively unsaved (default path), we might want a "Save As" flow eventually
                     // For now, MVP assumes we have a path from startup or just saves to current effective path
                     let _ = project.read().save(); 
                },
                show_preview_stats: show_preview_stats(),
                on_toggle_preview_stats: move |_| {
                    show_preview_stats.set(!show_preview_stats());
                },
                use_hw_decode: use_hw_decode(),
                on_toggle_hw_decode: move |_| {
                    use_hw_decode.set(!use_hw_decode());
                    preview_dirty.set(true);
                },
            }

            // Main content
            div {
                class: "main-content",
                style: "display: flex; flex: 1; overflow: hidden;",

                // Left panel - Assets
                SidePanel {
                    title: "Assets",
                    width: left_w,
                    collapsed: left_collapsed(),
                    side: "left",
                    is_resizing: left_resizing,
                    on_toggle: move |_| left_collapsed.set(!left_collapsed()),
                    on_resize_start: move |e: MouseEvent| {
                        e.prevent_default();
                        dragging.set(Some("left"));
                        drag_start_pos.set(e.client_coordinates().x);
                        drag_start_size.set(left_width());
                    },
                    
                    // Assets panel content
                    AssetsPanelContent {
                        assets: project.read().assets.clone(),
                        thumbnailer: thumbnailer.read().clone(),
                        thumbnail_cache_buster: thumbnail_cache_buster(),
                        thumbnail_refresh_tick: thumbnail_refresh_tick(),
                        on_import: move |asset: crate::state::Asset| {
                            project.write().add_asset(asset.clone());
                            preview_dirty.set(true);
                            let thumbs = thumbnailer.read().clone();
                            let mut thumbnail_cache_buster = thumbnail_cache_buster.clone();
                            // Spawn background task for thumbnail generation
                            spawn(async move {
                                thumbs.generate(&asset, false).await;
                                thumbnail_cache_buster.set(thumbnail_cache_buster() + 1);
                            });
                        },
                        on_import_file: move |path: std::path::PathBuf| {
                            // Implicit Copy: Always import directly designated by the strict folder policy
                            // We assume a project exists because the startup modal blocks everything else
                            let import_result = project.write().import_file(&path);
                            match import_result {
                                Ok(asset_id) => {
                                    preview_dirty.set(true);
                                    // Trigger thumbnail generation for the new asset
                                    if let Some(asset) = project.read().find_asset(asset_id).cloned() {
                                        let thumbs = thumbnailer.read().clone();
                                        let mut thumbnail_cache_buster = thumbnail_cache_buster.clone();
                                        spawn(async move {
                                            thumbs.generate(&asset, false).await;
                                            thumbnail_cache_buster.set(thumbnail_cache_buster() + 1);
                                        });
                                    }
                                    spawn_asset_duration_probe(project, asset_id);
                                },
                                Err(e) => println!("Failed to import file {:?}: {}", path, e),
                            }
                        },
                        on_regenerate_thumbnails: move |asset_id: uuid::Uuid| {
                            if let Some(asset) = project.read().find_asset(asset_id).cloned() {
                                let thumbs = thumbnailer.read().clone();
                                let mut thumbnail_cache_buster = thumbnail_cache_buster.clone();
                                spawn(async move {
                                     // Force regeneration logic could require a flag in future,
                                     // but our `generate` function currently checks existence.
                                     // To force it, we should probably delete the cache dir first or add a force flag.
                                     // For now, let's just assume the user deleted the cache or we want to try again if it failed.
                                     // To firmly force it, we'll manually nuke the cache dir for this asset here before calling generate.
                                    
                                     // We need to resolve the cache path to delete it.
                                     // Thumbnailer encapsulates the path logic.
                                     // Let's just trust `generate` or update `Thumbnailer` later to support `force`.
                                     // Actually, user asked for manual trigger. Let's make it robust by deleting first.
                                     // But `Thumbnailer` struct has `cache_root` private.
                                     // Let's just call `generate` for now. If it already exists it returns silently.
                                     // If we really want "Re-generate", we need to clear it.
                                     // Let's updated `Thumbnailer` in a separate step or just rely on 'generate' doing a check.
                                     // User said "what triggers re-generation... force manually".
                                     // If `generate` returns early on existing folder, this won't do anything if folder exists.
                                     // We should probably modify `Thumbnailer::generate` to accept a `force` boolean.
                                     // I will do that in the next step. For now, let's wire up the UI.
                                     thumbs.generate(&asset, true).await;
                                     thumbnail_cache_buster.set(thumbnail_cache_buster() + 1);
                                });
                            }
                        },
                        on_delete: move |id| {
                            project.write().remove_asset(id);
                            preview_dirty.set(true);
                        },
                        on_add_to_timeline: move |asset_id| {
                            // Add clip at current playhead position using asset duration when available
                            let time = current_time();
                            let duration = resolve_asset_duration_seconds(project, asset_id)
                                .unwrap_or(DEFAULT_CLIP_DURATION_SECONDS);
                            project.write().add_clip_from_asset(asset_id, time, duration);
                            preview_dirty.set(true);
                        },
                        on_drag_start: move |id| dragged_asset.set(Some(id)),
                    }
                }

                // Center
                div {
                    class: "center-area",
                    style: "display: flex; flex-direction: column; flex: 1; overflow: hidden;",

                    PreviewPanel {
                        width: project.read().settings.width,
                        height: project.read().settings.height,
                        fps: project.read().settings.fps,
                        preview_frame: preview_frame(),
                        preview_stats: preview_stats(),
                        preview_gpu_upload_ms: preview_gpu_upload_ms(),
                        show_preview_stats: show_preview_stats(),
                        preview_native_active: preview_native_active(),
                    }

                    // Timeline resize handle
                    div {
                        class: "resize-handle",
                        style: "height: 4px; background-color: {BORDER_DEFAULT}; cursor: ns-resize; flex-shrink: 0;",
                        onmousedown: move |e| {
                            if !timeline_collapsed() {
                                e.prevent_default();
                                dragging.set(Some("timeline"));
                                drag_start_pos.set(e.client_coordinates().y);
                                drag_start_size.set(timeline_height());
                            }
                        },
                    }

                        TimelinePanel {
                            height: timeline_h,
                            collapsed: timeline_collapsed(),
                            is_resizing: timeline_resizing,
                            on_toggle: move |_| timeline_collapsed.set(!timeline_collapsed()),
                            // Project data
                            tracks: project.read().tracks.clone(),

                            clips: project.read().clips.clone(),
                            assets: project.read().assets.clone(),
                            thumbnailer: thumbnailer.read().clone(),
                            thumbnail_cache_buster: thumbnail_cache_buster(),
                            thumbnail_refresh_tick: thumbnail_refresh_tick(),
                            clip_cache_buckets: clip_cache_buckets(),
                            // Timeline state
                            current_time: current_time(),
                        duration: duration,
                        zoom: zoom(),
                        is_playing: is_playing(),
                        scroll_offset: scroll_offset(),
                        // Callbacks
                        on_seek: move |t: f64| {
                            // Snap to frame boundary (60fps) and clamp to duration
                            let snapped = ((t * 60.0).round() / 60.0).clamp(0.0, duration);
                            current_time.set(snapped);
                        },
                        on_zoom_change: move |z: f64| zoom.set(z.clamp(20.0, 500.0)),
                        on_play_pause: move |_| is_playing.set(!is_playing()),
                        on_scroll: move |offset: f64| scroll_offset.set(offset),
                        on_seek_start: move |e: MouseEvent| {
                            dragging.set(Some("playhead"));
                            drag_start_pos.set(e.client_coordinates().x);
                            drag_start_size.set(current_time());
                        },
                        on_seek_end: move |_| dragging.set(None),
                        is_seeking: dragging() == Some("playhead"),
                        // Track management
                        on_add_video_track: move |_| {
                            project.write().add_video_track();
                            preview_dirty.set(true);
                        },
                        on_add_audio_track: move |_| {
                            project.write().add_audio_track();
                            preview_dirty.set(true);
                        },
                        on_track_context_menu: move |(x, y, track_id)| {
                            context_menu.set(Some((x, y, track_id)));
                        },
                        // Clip operations
                        on_clip_delete: move |clip_id| {
                            project.write().remove_clip(clip_id);
                            selection.write().remove_clip(clip_id);
                            preview_dirty.set(true);
                        },
                        on_clip_move: move |(clip_id, new_start)| {
                            project.write().move_clip(clip_id, new_start);
                            preview_dirty.set(true);
                        },
                        on_clip_resize: move |(clip_id, new_start, new_duration)| {
                            project.write().resize_clip(clip_id, new_start, new_duration);
                            preview_dirty.set(true);
                        },
                        on_clip_move_track: move |(clip_id, direction)| {
                            if project.write().move_clip_to_adjacent_track(clip_id, direction) {
                                preview_dirty.set(true);
                            }
                        },
                        selected_clips: selection.read().clip_ids.clone(),
                        on_clip_select: move |clip_id| {
                            selection.write().select_clip(clip_id);
                        },
                        // Asset Drag & Drop
                        dragged_asset: dragged_asset(),
                        on_asset_drop: move |(track_id, time, asset_id)| {
                            let duration = resolve_asset_duration_seconds(project, asset_id)
                                .unwrap_or(DEFAULT_CLIP_DURATION_SECONDS);
                            let clip = crate::state::Clip::new(asset_id, track_id, time, duration);
                            project.write().add_clip(clip);
                            preview_dirty.set(true);
                        },
                    }
                }

                // Right panel
                SidePanel {
                    title: "Attributes",  
                    width: right_w,
                    collapsed: right_collapsed(),
                    side: "right",
                    is_resizing: right_resizing,
                    on_toggle: move |_| right_collapsed.set(!right_collapsed()),
                    on_resize_start: move |e: MouseEvent| {
                        e.prevent_default();
                        dragging.set(Some("right"));
                        drag_start_pos.set(e.client_coordinates().x);
                        drag_start_size.set(right_width());
                    },
                    
                    // Attributes panel placeholder content
                    AttributesPanelContent {
                        project: project,
                        selection: selection,
                        preview_dirty: preview_dirty,
                    }
                }
            }

            StatusBar {}
            
            // Context menu overlay
            if let Some((x, y, track_id)) = context_menu() {
                // Backdrop to catch clicks outside menu
                div {
                    style: "
                        position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                        z-index: 999;
                    ",
                    onclick: move |_| context_menu.set(None),
                }
                // The actual menu
                div {
                    style: "
                        position: fixed; 
                        left: min({x}px, calc(100vw - 150px)); 
                        top: min({y}px, calc(100vh - 120px));
                        background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                        border-radius: 6px; padding: 4px 0; min-width: 140px;
                        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                        z-index: 1000; font-size: 12px;
                    ",
                    // Check if this is the Markers track (can't delete)
                    {
                        let is_markers = project.read().find_track(track_id)
                            .map(|t| t.track_type == crate::state::TrackType::Marker)
                            .unwrap_or(false);
                        let track_name = project.read().find_track(track_id)
                            .map(|t| t.name.clone())
                            .unwrap_or_default();
                        
                        if is_markers {
                            rsx! {
                                div {
                                    style: "
                                        padding: 6px 12px; color: {TEXT_DIM};
                                        cursor: not-allowed;
                                    ",
                                    "Cannot delete Markers track"
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    style: "
                                        padding: 6px 12px; color: #ef4444; cursor: pointer;
                                        transition: background-color 0.1s ease;
                                    ",
                                    onmouseenter: move |_| {
                                        // Would need state for hover, skipping for now
                                    },
                                    onclick: move |_| {
                                        project.write().remove_track(track_id);
                                        selection.write().clear();
                                        preview_dirty.set(true);
                                        context_menu.set(None);
                                    },
                                    "🗑 Delete \"{track_name}\""
                                }

                                div {
                                    style: "height: 1px; background-color: {BORDER_SUBTLE}; margin: 2px 0;",
                                }

                                div {
                                    style: "
                                        padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                                        transition: background-color 0.1s ease;
                                    ",
                                    onmouseenter: move |_| {},
                                    onclick: move |_| {
                                        project.write().move_track_up(track_id);
                                        preview_dirty.set(true);
                                        context_menu.set(None);
                                    },
                                    "↑ Move Up"
                                }

                                div {
                                    style: "
                                        padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                                        transition: background-color 0.1s ease;
                                    ",
                                    onmouseenter: move |_| {},
                                    onclick: move |_| {
                                        project.write().move_track_down(track_id);
                                        preview_dirty.set(true);
                                        context_menu.set(None);
                                    },
                                    "↓ Move Down"
                                }
                            }
                        }
                    }
                }
            }

            // Startup Modal (Blocks everything until Project is created/loaded)
            if show_startup {
                StartupModal {
                    on_create: move |(parent_dir, name, settings): (std::path::PathBuf, String, crate::state::ProjectSettings)| {
                        // Create full path: parent_dir/name
                        let project_dir = parent_dir.join(&name);
                        match crate::state::Project::create_in_with_settings(&project_dir, &name, settings) {
                            Ok(new_proj) => {
                                // Initialize thumbnailer with new project path
                                thumbnailer.set(std::sync::Arc::new(crate::core::thumbnailer::Thumbnailer::new(new_proj.project_path.clone().unwrap())));
                                previewer.set(std::sync::Arc::new(
                                    crate::core::preview::PreviewRenderer::new(
                                        new_proj.project_path.clone().unwrap(),
                                        PREVIEW_CACHE_BUDGET_BYTES,
                                    ),
                                ));
                                project.set(new_proj);
                                preview_dirty.set(true);
                                spawn_missing_duration_probes(project);
                                startup_done.set(true);
                            },
                            Err(e) => println!("Error creating project: {}", e),
                        }
                    },
                    on_open: move |path: std::path::PathBuf| {
                         match crate::state::Project::load(&path) { // path is the project folder
                            Ok(loaded_proj) => {
                                // Initialize thumbnailer with loaded project path
                                thumbnailer.set(std::sync::Arc::new(crate::core::thumbnailer::Thumbnailer::new(loaded_proj.project_path.clone().unwrap())));
                                previewer.set(std::sync::Arc::new(
                                    crate::core::preview::PreviewRenderer::new(
                                        loaded_proj.project_path.clone().unwrap(),
                                        PREVIEW_CACHE_BUDGET_BYTES,
                                    ),
                                ));
                                project.set(loaded_proj);
                                preview_dirty.set(true);
                                spawn_missing_duration_probes(project);
                                startup_done.set(true);
                            },
                            Err(e) => println!("Error loading project: {}", e),
                        }
                    }
                }
            }

            // New Project Modal (File > New Project)
            if show_new_project_dialog() {
                // ... reusing the existing modal structure but purely for NEW projects
                // We reuse the startup modal logic essentially, but let's keep the existing simple one for now
                // just adapted to immediate creation
                div {
                    style: "
                        position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                        background-color: rgba(0, 0, 0, 0.5);
                        display: flex; align-items: center; justify-content: center;
                        z-index: 2000;
                    ",
                    onclick: move |_| show_new_project_dialog.set(false),
                    
                    div {
                         style: "
                            width: 400px; background-color: {BG_ELEVATED};
                            border: 1px solid {BORDER_DEFAULT}; border-radius: 8px;
                            padding: 24px; box-shadow: 0 10px 25px rgba(0,0,0,0.5);
                        ",
                         onclick: move |e| e.stop_propagation(),
                         
                         h3 { style: "margin: 0 0 16px 0; font-size: 16px; color: {TEXT_PRIMARY};", "New Project" }
                         // ... (keep existing simple input for now) ...
                         // NOTE: We should probably unify this with StartupModal eventually
                         div {
                            style: "margin-bottom: 20px;",
                             button {
                                style: "width: 100%; padding: 10px; background: {ACCENT_VIDEO}; border: none; border-radius: 4px; color: white; cursor: pointer;",
                                onclick: move |_| {
                                     // Quick hack: just reset to startup modal for now to force the flow
                                    project.set(crate::state::Project::default()); // Reset to untitled
                                    startup_done.set(false); // Trigger startup modal
                                    show_new_project_dialog.set(false);
                                },
                                "Go to Project Wizard"
                             }
                         }
                    }
                }
            }
        }
    }
}

/// Modal shown on app launch
#[component]
fn StartupModal(
    on_create: EventHandler<(std::path::PathBuf, String, crate::state::ProjectSettings)>,
    on_open: EventHandler<std::path::PathBuf>
) -> Element {
    let mut name = use_signal(|| "My New Project".to_string());
    let mut width = use_signal(|| "1920".to_string());
    let mut height = use_signal(|| "1080".to_string());
    let mut fps = use_signal(|| "60".to_string());
    let mut duration = use_signal(|| "60".to_string());
    
    // Default projects folder
    let projects_folder = std::env::current_dir().unwrap_or_default().join("projects");
    let projects_folder_clone = projects_folder.clone();
    let projects_folder_for_browse = projects_folder.clone();
    let projects_folder_for_open = projects_folder.clone();
    let projects_folder_for_scan = projects_folder.clone();
    
    // Use `Option<PathBuf>` to store the selected parent directory
    let mut parent_dir = use_signal(move || projects_folder_clone.clone());
    
    // Refresh counter - increment to force re-scan of projects
    let mut refresh_counter = use_signal(|| 0u32);
    
    // Context menu state: Option<(x, y, project_path, project_name)>
    let mut context_menu: Signal<Option<(f64, f64, std::path::PathBuf, String)>> = use_signal(|| None);

    fn parse_u32(value: &str, default: u32, min: u32) -> u32 {
        value
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|v| *v >= min)
            .unwrap_or(default)
    }

    fn parse_f64(value: &str, default: f64, min: f64) -> f64 {
        value
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| *v >= min)
            .unwrap_or(default)
    }
    
    // Scan for existing projects (folders containing project.json)
    // Re-runs when refresh_counter changes
    let _ = refresh_counter(); // Subscribe to changes
    let existing_projects: Vec<(String, std::path::PathBuf)> = if projects_folder_for_scan.exists() {
        std::fs::read_dir(&projects_folder_for_scan)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_dir())
                    .filter(|entry| entry.path().join("project.json").exists())
                    .map(|entry| {
                        let path = entry.path();
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        (name, path)
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    rsx! {
        div {
            style: "
                position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                background-color: {BG_BASE}; z-index: 9999;
                display: flex; align-items: center; justify-content: center;
            ",
            
            div {
                style: "
                    width: 720px; display: flex; flex-direction: column; 
                    background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                    border-radius: 12px; overflow: hidden; 
                    box-shadow: 0 25px 60px rgba(0,0,0,0.6), 0 0 0 1px rgba(255,255,255,0.03);
                ",
                
                // Header
                div {
                    style: "
                        padding: 28px 32px 24px; 
                        border-bottom: 1px solid {BORDER_DEFAULT}; 
                        background: linear-gradient(180deg, {BG_SURFACE} 0%, {BG_ELEVATED} 100%);
                    ",
                    h1 { 
                        style: "
                            margin: 0; font-size: 22px; font-weight: 600; 
                            color: {TEXT_PRIMARY}; letter-spacing: -0.3px;
                        ", 
                        "NLA AI Video Creator" 
                    }
                    p { 
                        style: "margin: 6px 0 0; font-size: 13px; color: {TEXT_MUTED};", 
                        "Create a new project or open an existing one" 
                    }
                }
                
                // Main content area
                div {
                    style: "display: flex; min-height: 400px;",
                    
                    // ═══════════════════════════════════════════════════════════
                    // LEFT: Create New Project
                    // ═══════════════════════════════════════════════════════════
                    div {
                        style: "
                            flex: 1.2; padding: 24px 28px; 
                            border-right: 1px solid {BORDER_DEFAULT}; 
                            display: flex; flex-direction: column;
                        ",
                        
                        // Section header
                        div {
                            style: "display: flex; align-items: center; gap: 10px; margin-bottom: 20px;",
                            div {
                                style: "
                                    width: 32px; height: 32px; border-radius: 8px;
                                    background: linear-gradient(135deg, {ACCENT_VIDEO}22 0%, {ACCENT_VIDEO}11 100%);
                                    border: 1px solid {ACCENT_VIDEO}33;
                                    display: flex; align-items: center; justify-content: center;
                                    font-size: 14px;
                                ",
                                "✨"
                            }
                            h2 { 
                                style: "margin: 0; font-size: 15px; font-weight: 600; color: {TEXT_PRIMARY};", 
                                "Create New Project" 
                            }
                        }
                        
                        // Form content
                        div {
                            style: "flex: 1; display: flex; flex-direction: column; gap: 16px;",
                            
                            // Project Name
                            div {
                                label { 
                                    style: "
                                        display: block; font-size: 11px; font-weight: 500;
                                        color: {TEXT_MUTED}; margin-bottom: 6px;
                                        text-transform: uppercase; letter-spacing: 0.5px;
                                    ", 
                                    "Project Name" 
                                }
                                input {
                                    style: "
                                        width: 100%; padding: 10px 14px; 
                                        background: {BG_BASE}; border: 1px solid {BORDER_DEFAULT}; 
                                        border-radius: 6px; color: {TEXT_PRIMARY}; 
                                        font-size: 13px; outline: none;
                                        transition: border-color 0.15s ease;
                                    ",
                                    value: "{name}",
                                    placeholder: "Enter project name...",
                                    oninput: move |e| name.set(e.value()),
                                }
                            }

                            // Resolution section
                            div {
                                label { 
                                    style: "
                                        display: block; font-size: 11px; font-weight: 500;
                                        color: {TEXT_MUTED}; margin-bottom: 8px;
                                        text-transform: uppercase; letter-spacing: 0.5px;
                                    ", 
                                    "Resolution" 
                                }
                                
                                // Preset buttons
                                div {
                                    style: "display: flex; gap: 6px; margin-bottom: 10px;",
                                    
                                    // 1080p preset
                                    button {
                                        style: "
                                            padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                            border: 1px solid {BORDER_DEFAULT}; cursor: pointer;
                                            background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                            transition: all 0.15s ease;
                                        ",
                                        onclick: move |_| {
                                            width.set("1920".to_string());
                                            height.set("1080".to_string());
                                        },
                                        "1080p"
                                    }
                                    
                                    // 4K preset
                                    button {
                                        style: "
                                            padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                            border: 1px solid {BORDER_DEFAULT}; cursor: pointer;
                                            background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                            transition: all 0.15s ease;
                                        ",
                                        onclick: move |_| {
                                            width.set("3840".to_string());
                                            height.set("2160".to_string());
                                        },
                                        "4K"
                                    }
                                    
                                    // Vertical (9:16) preset
                                    button {
                                        style: "
                                            padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                            border: 1px solid {BORDER_DEFAULT}; cursor: pointer;
                                            background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                            transition: all 0.15s ease;
                                        ",
                                        onclick: move |_| {
                                            width.set("1080".to_string());
                                            height.set("1920".to_string());
                                        },
                                        "9:16"
                                    }
                                    
                                    // Square preset
                                    button {
                                        style: "
                                            padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                            border: 1px solid {BORDER_DEFAULT}; cursor: pointer;
                                            background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                            transition: all 0.15s ease;
                                        ",
                                        onclick: move |_| {
                                            width.set("1080".to_string());
                                            height.set("1080".to_string());
                                        },
                                        "1:1"
                                    }
                                }
                                
                                // Custom resolution inputs
                                div {
                                    style: "display: flex; gap: 8px; align-items: center;",
                                    input {
                                        style: "
                                            flex: 1; padding: 8px 12px; background: {BG_BASE};
                                            border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                            color: {TEXT_PRIMARY}; font-size: 13px; outline: none;
                                            text-align: center;
                                        ",
                                        r#type: "number",
                                        min: "1",
                                        step: "1",
                                        value: "{width}",
                                        oninput: move |e| width.set(e.value()),
                                    }
                                    span { 
                                        style: "color: {TEXT_DIM}; font-size: 12px; font-weight: 500;", 
                                        "×" 
                                    }
                                    input {
                                        style: "
                                            flex: 1; padding: 8px 12px; background: {BG_BASE};
                                            border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                            color: {TEXT_PRIMARY}; font-size: 13px; outline: none;
                                            text-align: center;
                                        ",
                                        r#type: "number",
                                        min: "1",
                                        step: "1",
                                        value: "{height}",
                                        oninput: move |e| height.set(e.value()),
                                    }
                                }
                            }

                            // FPS & Duration row
                            div {
                                style: "display: flex; gap: 16px;",
                                div {
                                    style: "flex: 1;",
                                    label { 
                                        style: "
                                            display: block; font-size: 11px; font-weight: 500;
                                            color: {TEXT_MUTED}; margin-bottom: 6px;
                                            text-transform: uppercase; letter-spacing: 0.5px;
                                        ", 
                                        "Frame Rate" 
                                    }
                                    div {
                                        style: "display: flex; align-items: center; gap: 6px;",
                                        input {
                                            style: "
                                                flex: 1; padding: 8px 12px; background: {BG_BASE};
                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                                color: {TEXT_PRIMARY}; font-size: 13px; outline: none;
                                            ",
                                            r#type: "number",
                                            min: "1",
                                            step: "1",
                                            value: "{fps}",
                                            oninput: move |e| fps.set(e.value()),
                                        }
                                        span { 
                                            style: "color: {TEXT_DIM}; font-size: 11px;", 
                                            "fps" 
                                        }
                                    }
                                }
                                div {
                                    style: "flex: 1;",
                                    label { 
                                        style: "
                                            display: block; font-size: 11px; font-weight: 500;
                                            color: {TEXT_MUTED}; margin-bottom: 6px;
                                            text-transform: uppercase; letter-spacing: 0.5px;
                                        ", 
                                        "Duration" 
                                    }
                                    div {
                                        style: "display: flex; align-items: center; gap: 6px;",
                                        input {
                                            style: "
                                                flex: 1; padding: 8px 12px; background: {BG_BASE};
                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                                color: {TEXT_PRIMARY}; font-size: 13px; outline: none;
                                            ",
                                            r#type: "number",
                                            min: "1",
                                            step: "1",
                                            value: "{duration}",
                                            oninput: move |e| duration.set(e.value()),
                                        }
                                        span { 
                                            style: "color: {TEXT_DIM}; font-size: 11px;", 
                                            "sec" 
                                        }
                                    }
                                }
                            }

                            // Divider
                            div { 
                                style: "height: 1px; background: linear-gradient(90deg, {BORDER_SUBTLE} 0%, transparent 100%); margin: 4px 0;" 
                            }
                            
                            // Location
                            div {
                                label { 
                                    style: "
                                        display: block; font-size: 11px; font-weight: 500;
                                        color: {TEXT_MUTED}; margin-bottom: 6px;
                                        text-transform: uppercase; letter-spacing: 0.5px;
                                    ", 
                                    "Save Location" 
                                }
                                div {
                                    style: "display: flex; gap: 8px;",
                                    div {
                                        style: "
                                            flex: 1; padding: 8px 12px; background: {BG_BASE};
                                            border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                            color: {TEXT_DIM}; font-size: 12px;
                                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
                                        ",
                                        "{parent_dir().to_string_lossy()}"
                                    }
                                    button {
                                        class: "collapse-btn",
                                        style: "
                                            padding: 8px 14px; background: {BG_SURFACE}; 
                                            border: 1px solid {BORDER_DEFAULT}; border-radius: 6px; 
                                            color: {TEXT_SECONDARY}; font-size: 12px; cursor: pointer;
                                            transition: all 0.15s ease;
                                        ",
                                        onclick: move |_| {
                                            let start_dir = projects_folder_for_browse.clone();
                                            if let Some(path) = rfd::FileDialog::new()
                                                .set_directory(&start_dir)
                                                .pick_folder() 
                                            {
                                                parent_dir.set(path);
                                            }
                                        },
                                        "Browse"
                                    }
                                }
                            }
                        }
                        
                        // Create button
                        button {
                            class: "collapse-btn",
                            style: "
                                width: 100%; padding: 12px; margin-top: 20px;
                                background: linear-gradient(180deg, {ACCENT_VIDEO} 0%, #1ea34b 100%);
                                border: none; border-radius: 8px;
                                color: white; font-size: 13px; font-weight: 600; 
                                cursor: pointer; transition: all 0.2s ease;
                                box-shadow: 0 2px 8px rgba(34, 197, 94, 0.3);
                            ",
                            onclick: move |_| {
                                let n = name();
                                if !n.trim().is_empty() {
                                    let settings = crate::state::ProjectSettings {
                                        width: parse_u32(&width(), 1920, 1),
                                        height: parse_u32(&height(), 1080, 1),
                                        fps: parse_f64(&fps(), 60.0, 1.0),
                                        duration_seconds: parse_f64(&duration(), 60.0, 1.0),
                                    };
                                    on_create.call((parent_dir(), n, settings));
                                }
                            },
                            "Create Project"
                        }
                    }
                    
                    // ═══════════════════════════════════════════════════════════
                    // RIGHT: Open Existing Project
                    // ═══════════════════════════════════════════════════════════
                    div {
                        style: "
                            flex: 0.8; padding: 24px 28px; 
                            display: flex; flex-direction: column; 
                            background-color: {BG_BASE}; 
                            min-width: 0; overflow: hidden;
                        ",
                        
                        // Section header
                        div {
                            style: "display: flex; align-items: center; gap: 10px; margin-bottom: 16px;",
                            div {
                                style: "
                                    width: 32px; height: 32px; border-radius: 8px;
                                    background: linear-gradient(135deg, {ACCENT_AUDIO}22 0%, {ACCENT_AUDIO}11 100%);
                                    border: 1px solid {ACCENT_AUDIO}33;
                                    display: flex; align-items: center; justify-content: center;
                                    font-size: 14px;
                                ",
                                "📂"
                            }
                            h2 { 
                                style: "margin: 0; font-size: 15px; font-weight: 600; color: {TEXT_PRIMARY};", 
                                "Recent Projects" 
                            }
                        }
                        
                        // Project list or empty state
                        if existing_projects.is_empty() {
                            div {
                                style: "
                                    flex: 1; display: flex; flex-direction: column; 
                                    align-items: center; justify-content: center; 
                                    border: 1px dashed {BORDER_DEFAULT}; border-radius: 8px;
                                    padding: 32px;
                                ",
                                div { 
                                    style: "font-size: 40px; opacity: 0.3; margin-bottom: 12px;", 
                                    "📁" 
                                }
                                p { 
                                    style: "margin: 0; font-size: 13px; color: {TEXT_DIM}; text-align: center;", 
                                    "No projects yet" 
                                }
                                p { 
                                    style: "margin: 6px 0 0; font-size: 11px; color: {TEXT_DIM}; text-align: center;", 
                                    "Create one to get started" 
                                }
                            }
                        } else {
                            div {
                                style: "
                                    flex: 1; overflow-y: auto; overflow-x: hidden;
                                    border: 1px solid {BORDER_SUBTLE}; border-radius: 8px;
                                    background-color: {BG_ELEVATED};
                                    min-height: 0;
                                ",
                                for (proj_name, proj_path) in existing_projects.iter() {
                                    {
                                        let path_clone = proj_path.clone();
                                        let path_for_menu = proj_path.clone();
                                        let name_for_menu = proj_name.clone();
                                        let on_open_clone = on_open.clone();
                                        rsx! {
                                            div {
                                                class: "collapse-btn",
                                                key: "{proj_path.display()}",
                                                style: "
                                                    padding: 12px 14px; cursor: pointer;
                                                    border-bottom: 1px solid {BORDER_SUBTLE};
                                                    transition: background-color 0.15s ease;
                                                ",
                                                onclick: move |_| {
                                                    on_open_clone.call(path_clone.clone());
                                                },
                                                oncontextmenu: move |e| {
                                                    e.prevent_default();
                                                    context_menu.set(Some((
                                                        e.client_coordinates().x,
                                                        e.client_coordinates().y,
                                                        path_for_menu.clone(),
                                                        name_for_menu.clone()
                                                    )));
                                                },
                                                div {
                                                    style: "display: flex; align-items: center; gap: 10px; min-width: 0;",
                                                    div {
                                                        style: "
                                                            width: 28px; height: 28px; border-radius: 6px;
                                                            background: {BG_SURFACE}; border: 1px solid {BORDER_SUBTLE};
                                                            display: flex; align-items: center; justify-content: center;
                                                            font-size: 12px; flex-shrink: 0;
                                                        ",
                                                        "🎬"
                                                    }
                                                    div {
                                                        style: "flex: 1; min-width: 0; overflow: hidden;",
                                                        div { 
                                                            style: "
                                                                font-size: 13px; font-weight: 500; color: {TEXT_PRIMARY}; 
                                                                white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
                                                            ", 
                                                            "{proj_name}" 
                                                        }
                                                    }
                                                    // Arrow indicator
                                                    span {
                                                        style: "color: {TEXT_DIM}; font-size: 10px;",
                                                        "→"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Browse button
                        button {
                            class: "collapse-btn",
                            style: "
                                width: 100%; padding: 10px; margin-top: 16px; flex-shrink: 0;
                                background-color: {BG_SURFACE}; border: 1px solid {BORDER_DEFAULT};
                                border-radius: 8px; color: {TEXT_SECONDARY}; 
                                font-size: 12px; font-weight: 500; cursor: pointer;
                                transition: all 0.15s ease;
                                display: flex; align-items: center; justify-content: center; gap: 6px;
                            ",
                            onclick: move |_| {
                                let start_dir = projects_folder_for_open.clone();
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_directory(&start_dir)
                                    .set_title("Open Project")
                                    .pick_folder()
                                {
                                    on_open.call(path);
                                }
                            },
                            span { style: "font-size: 11px;", "📁" }
                            "Browse for Project..."
                        }
                    }
                }
            }
            
            // Context menu overlay for project deletion
            if let Some((x, y, proj_path, proj_name)) = context_menu() {
                // Backdrop to catch clicks outside menu
                div {
                    style: "
                        position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                        z-index: 10000;
                    ",
                    onclick: move |_| context_menu.set(None),
                }
                // The actual menu
                div {
                    style: "
                        position: fixed; 
                        left: min({x}px, calc(100vw - 160px)); 
                        top: min({y}px, calc(100vh - 60px));
                        background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                        border-radius: 8px; padding: 4px 0; min-width: 160px;
                        box-shadow: 0 8px 24px rgba(0,0,0,0.4);
                        z-index: 10001; font-size: 12px;
                    ",
                    div {
                        class: "collapse-btn",
                        style: "
                            padding: 8px 14px; color: #ef4444; cursor: pointer;
                            display: flex; align-items: center; gap: 8px;
                            transition: background-color 0.1s ease;
                        ",
                        onclick: move |_| {
                            // Delete the project folder
                            if let Err(e) = std::fs::remove_dir_all(&proj_path) {
                                println!("Failed to delete project {:?}: {}", proj_path, e);
                            } else {
                                println!("Deleted project: {:?}", proj_path);
                            }
                            // Close menu and refresh list
                            context_menu.set(None);
                            refresh_counter.set(refresh_counter() + 1);
                        },
                        span { "🗑" }
                        "Delete \"{proj_name}\""
                    }
                }
            }
        }
    }
}

#[component]
fn TitleBar(
    project_name: String, 
    on_new_project: EventHandler<MouseEvent>,
    on_save: EventHandler<MouseEvent>,
    show_preview_stats: bool,
    on_toggle_preview_stats: EventHandler<MouseEvent>,
    use_hw_decode: bool,
    on_toggle_hw_decode: EventHandler<MouseEvent>,
) -> Element {
    let stats_toggle_bg = if show_preview_stats { BG_HOVER } else { BG_BASE };
    let hw_toggle_bg = if use_hw_decode { BG_HOVER } else { BG_BASE };
    rsx! {
        div {
            style: "
                display: flex; align-items: center; justify-content: space-between;
                height: 40px; padding: 0 16px;
                background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                user-select: none;
            ",
            div {
                style: "display: flex; align-items: center; gap: 20px;",
                span { style: "font-size: 13px; font-weight: 600; color: {TEXT_SECONDARY};", "NLA AI Video Creator" }
                
                // Simple File > New Project button for now (can expand to full menu later)
                button {
                    class: "collapse-btn", // Reusing hover style
                    style: "
                        background: transparent; border: none; color: {TEXT_PRIMARY}; 
                        font-size: 12px; cursor: pointer; padding: 4px 8px; border-radius: 4px;
                    ",
                    onclick: move |e| on_new_project.call(e),
                    "New Project"
                }

                // Save button
                button {
                    class: "collapse-btn",
                    style: "
                        background: transparent; border: none; color: {TEXT_PRIMARY}; 
                        font-size: 12px; cursor: pointer; padding: 4px 8px; border-radius: 4px;
                    ",
                    onclick: move |e| on_save.call(e),
                    "Save"
                }
            }
            
            span { style: "font-size: 13px; color: {TEXT_MUTED};", "{project_name}" }
            
            div {
                style: "display: flex; align-items: center; justify-content: flex-end; gap: 12px; min-width: 220px;",
                div {
                    style: "display: flex; align-items: center; gap: 6px;",
                    span {
                        style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.6px;",
                        "Stats"
                    }
                    button {
                        class: "collapse-btn",
                        style: "
                            background: {stats_toggle_bg};
                            border: 1px solid {BORDER_DEFAULT};
                            color: {TEXT_PRIMARY}; font-size: 11px; cursor: pointer;
                            padding: 4px 10px; border-radius: 999px;
                        ",
                        onclick: move |e| on_toggle_preview_stats.call(e),
                        if show_preview_stats { "On" } else { "Off" }
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 6px;",
                    span {
                        style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.6px;",
                        "HW Dec"
                    }
                    button {
                        class: "collapse-btn",
                        style: "
                            background: {hw_toggle_bg};
                            border: 1px solid {BORDER_DEFAULT};
                            color: {TEXT_PRIMARY}; font-size: 11px; cursor: pointer;
                            padding: 4px 10px; border-radius: 999px;
                        ",
                        onclick: move |e| on_toggle_hw_decode.call(e),
                        if use_hw_decode { "On" } else { "Off" }
                    }
                }
            }
        }
    }
}

#[component]
fn SidePanel(
    title: &'static str,
    width: f64,
    collapsed: bool,
    side: &'static str,
    is_resizing: bool,
    on_toggle: EventHandler<MouseEvent>,
    on_resize_start: EventHandler<MouseEvent>,
    children: Element,  // Custom content for this panel
) -> Element {
    let is_left = side == "left";
    // Arrow points toward the resize edge when expanded, away when collapsed
    let icon = if collapsed { 
        if is_left { "▶" } else { "◀" } 
    } else { 
        if is_left { "◀" } else { "▶" } 
    };
    let border = if is_left { 
        format!("border-right: 1px solid {BORDER_DEFAULT};") 
    } else { 
        format!("border-left: 1px solid {BORDER_DEFAULT};") 
    };
    
    // Only apply transition when NOT resizing
    let transition = if is_resizing { "none" } else { "width 0.2s ease, min-width 0.2s ease" };
    
    // Cursor for collapsed state (entire rail is clickable)
    let rail_cursor = if collapsed { "pointer" } else { "default" };

    // Class for hover effect when collapsed
    let panel_class = if collapsed { "collapsed-rail" } else { "" };

    rsx! {
        div {
            class: "{panel_class}",
            style: "
                display: flex; flex-direction: row;
                width: {width}px; min-width: {width}px;
                background-color: {BG_ELEVATED}; {border}
                transition: {transition};
                overflow: hidden;
                cursor: {rail_cursor};
            ",
            // Make entire collapsed panel clickable
            onclick: move |e| {
                if collapsed {
                    on_toggle.call(e);
                }
            },

            // Resize handle (left edge for right panel)
            if !collapsed && !is_left {
                div {
                    class: "resize-handle",
                    style: "width: 4px; height: 100%; background-color: {BORDER_DEFAULT}; cursor: ew-resize; flex-shrink: 0;",
                    onmousedown: move |e| {
                        e.prevent_default();
                        e.stop_propagation();
                        on_resize_start.call(e);
                    },
                }
            }

            div {
                style: "display: flex; flex-direction: column; flex: 1; overflow: hidden;",

                // Header - different layout when collapsed vs expanded
                if collapsed {
                    // Collapsed: just the arrow button centered
                    div {
                        style: "
                            display: flex; align-items: center; justify-content: center;
                            height: 32px;
                            background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                            flex-shrink: 0;
                        ",
                        button {
                            class: "collapse-btn",
                            style: "
                                width: 24px; height: 24px; border: none; border-radius: 4px;
                                background: transparent; color: {TEXT_MUTED}; font-size: 10px;
                                cursor: pointer; display: flex; align-items: center; justify-content: center;
                            ",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_toggle.call(e);
                            },
                            "{icon}"
                        }
                    }
                } else {
                    // Expanded: title on left, arrow on the side nearest resize bar
                    div {
                        style: "
                            display: flex; align-items: center;
                            height: 32px; padding: 0 8px;
                            background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                            flex-shrink: 0;
                        ",
                        // For left panel: [title ... arrow] (arrow near right edge/resize bar)
                        // For right panel: [arrow ... title] (arrow near left edge/resize bar)
                        if !is_left {
                            button {
                                class: "collapse-btn",
                                style: "
                                    width: 24px; height: 24px; border: none; border-radius: 4px;
                                    background: transparent; color: {TEXT_MUTED}; font-size: 10px;
                                    cursor: pointer; display: flex; align-items: center; justify-content: center;
                                    margin-right: 8px;
                                ",
                                onclick: move |e| on_toggle.call(e),
                                "{icon}"
                            }
                        }
                        
                        span { 
                            style: "font-size: 11px; font-weight: 500; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px; flex: 1;", 
                            "{title}" 
                        }
                        
                        if is_left {
                            button {
                                class: "collapse-btn",
                                style: "
                                    width: 24px; height: 24px; border: none; border-radius: 4px;
                                    background: transparent; color: {TEXT_MUTED}; font-size: 10px;
                                    cursor: pointer; display: flex; align-items: center; justify-content: center;
                                ",
                                onclick: move |e| on_toggle.call(e),
                                "{icon}"
                            }
                        }
                    }
                }

                // Content (only when expanded)
                if !collapsed {
                    div {
                        style: "flex: 1; overflow-y: auto;",
                        {children}
                    }
                }
            }

            // Resize handle (right edge for left panel)
            if !collapsed && is_left {
                div {
                    class: "resize-handle",
                    style: "width: 4px; height: 100%; background-color: {BORDER_DEFAULT}; cursor: ew-resize; flex-shrink: 0;",
                    onmousedown: move |e| {
                        e.prevent_default();
                        e.stop_propagation();
                        on_resize_start.call(e);
                    },
                }
            }
        }
    }
}

#[component]
fn NumericField(
    label: &'static str,
    value: f32,
    step: &'static str,
    clamp_min: Option<f32>,
    clamp_max: Option<f32>,
    on_commit: EventHandler<f32>,
) -> Element {
    let mut text = use_signal(|| format!("{:.2}", value));

    let make_commit = || {
        let mut text = text.clone();
        let on_commit = on_commit.clone();
        move || {
            let mut parsed = parse_f32_input(&text(), value);
            if let Some(min) = clamp_min {
                parsed = parsed.max(min);
            }
            if let Some(max) = clamp_max {
                parsed = parsed.min(max);
            }
            on_commit.call(parsed);
            text.set(format!("{:.2}", parsed));
        }
    };

    let mut commit_on_blur = make_commit();
    let mut commit_on_key = make_commit();

    let on_blur = move |_| {
        commit_on_blur();
    };

    let on_keydown = move |e: KeyboardEvent| {
        if e.key() == Key::Enter {
            commit_on_key();
        }
    };

    let text_value = text();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px;",
            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
            input {
                r#type: "number",
                step: "{step}",
                value: "{text_value}",
                oninput: move |e| text.set(e.value()),
                onblur: on_blur,
                onkeydown: on_keydown,
            }
        }
    }
}

#[component]
fn AttributesPanelContent(
    project: Signal<crate::state::Project>,
    selection: Signal<crate::state::SelectionState>,
    preview_dirty: Signal<bool>,
) -> Element {
    let selection_state = selection.read();
    let selected_count = selection_state.clip_ids.len();
    let selected_clip_id = selection_state.primary_clip();
    drop(selection_state);

    if selected_count == 0 {
        return rsx! {
            div {
                style: "padding: 12px;",
                div {
                    style: "
                        display: flex; align-items: center; justify-content: center;
                        height: 80px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                        color: {TEXT_DIM}; font-size: 12px;
                    ",
                    "No selection"
                }
            }
        };
    }

    if selected_count > 1 {
        return rsx! {
            div {
                style: "padding: 12px;",
                div {
                    style: "
                        display: flex; align-items: center; justify-content: center;
                        height: 80px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                        color: {TEXT_DIM}; font-size: 12px;
                    ",
                    "{selected_count} items selected"
                }
            }
        };
    }

    let Some(clip_id) = selected_clip_id else {
        return rsx! {
            div {
                style: "padding: 12px;",
                div {
                    style: "
                        display: flex; align-items: center; justify-content: center;
                        height: 80px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                        color: {TEXT_DIM}; font-size: 12px;
                    ",
                    "Selection missing"
                }
            }
        };
    };

    let project_read = project.read();
    let clip = match project_read.clips.iter().find(|c| c.id == clip_id) {
        Some(clip) => clip.clone(),
        None => {
            drop(project_read);
            return rsx! {
                div {
                    style: "padding: 12px;",
                    div {
                        style: "
                            display: flex; align-items: center; justify-content: center;
                            height: 80px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                            color: {TEXT_DIM}; font-size: 12px;
                        ",
                        "Selection missing"
                    }
                }
            };
        }
    };

    let asset_name = project_read
        .find_asset(clip.asset_id)
        .map(|asset| asset.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    drop(project_read);

    let transform = clip.transform;
    let clip_id = clip.id;

    rsx! {
        div {
            style: "padding: 12px; display: flex; flex-direction: column; gap: 12px;",
            div {
                style: "display: flex; flex-direction: column; gap: 6px;",
                span { style: "font-size: 11px; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px;", "Clip" }
                span { style: "font-size: 12px; color: {TEXT_PRIMARY};", "{asset_name}" }
            }

            div {
                style: "
                    display: flex; flex-direction: column; gap: 10px;
                    padding: 10px; background-color: {BG_SURFACE};
                    border: 1px solid {BORDER_SUBTLE}; border-radius: 6px;
                ",
                div {
                    style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;",
                    "Transform"
                }
                div {
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px;",
                    NumericField {
                        key: "{clip_id}-position-x",
                        label: "Position X",
                        value: transform.position_x,
                        step: "1",
                        clamp_min: None,
                        clamp_max: None,
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.position_x = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                    NumericField {
                        key: "{clip_id}-position-y",
                        label: "Position Y",
                        value: transform.position_y,
                        step: "1",
                        clamp_min: None,
                        clamp_max: None,
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.position_y = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                    NumericField {
                        key: "{clip_id}-scale-x",
                        label: "Scale X",
                        value: transform.scale_x,
                        step: "0.01",
                        clamp_min: Some(0.01),
                        clamp_max: None,
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.scale_x = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                    NumericField {
                        key: "{clip_id}-scale-y",
                        label: "Scale Y",
                        value: transform.scale_y,
                        step: "0.01",
                        clamp_min: Some(0.01),
                        clamp_max: None,
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.scale_y = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                    NumericField {
                        key: "{clip_id}-rotation",
                        label: "Rotation",
                        value: transform.rotation_deg,
                        step: "1",
                        clamp_min: None,
                        clamp_max: None,
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.rotation_deg = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                    NumericField {
                        key: "{clip_id}-opacity",
                        label: "Opacity",
                        value: transform.opacity,
                        step: "0.05",
                        clamp_min: Some(0.0),
                        clamp_max: Some(1.0),
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.opacity = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                }
            }
        }
    }
}
#[component]
fn PreviewPanel(
    width: u32,
    height: u32,
    fps: f64,
    preview_frame: Option<crate::core::preview::PreviewFrameInfo>,
    preview_stats: Option<crate::core::preview::PreviewStats>,
    preview_gpu_upload_ms: Option<f64>,
    show_preview_stats: bool,
    preview_native_active: bool,
) -> Element {
    let fps_label = format!("{:.0}", fps);
    let has_frame = preview_frame.is_some();
    let canvas_visibility = if preview_native_active {
        "hidden"
    } else if has_frame {
        "visible"
    } else {
        "hidden"
    };
    let show_placeholder = !preview_native_active && !has_frame;
    let stats_text = if show_preview_stats {
        preview_stats.map(|stats| {
        let total_queries = stats.cache_hits + stats.cache_misses;
        let hit_ratio = if total_queries > 0 {
            (stats.cache_hits as f64 / total_queries as f64) * 100.0
        } else {
            0.0
        };
        let hw_total = stats.hw_decode_frames + stats.sw_decode_frames;
        let hw_label = if hw_total > 0 {
            let pct = (stats.hw_decode_frames as f64 / hw_total as f64) * 100.0;
            format!("hwdec {:.0}%", pct)
        } else {
            "hwdec --".to_string()
        };
        let scan_ms = (stats.collect_ms - stats.video_decode_ms - stats.still_load_ms).max(0.0);
        let mut lines = Vec::new();
        lines.push(format!("total {:.1}ms", stats.total_ms));
        lines.push(format!("scan {:.1}ms", scan_ms));
        lines.push(format!("vdec {:.1}ms", stats.video_decode_ms));
        lines.push(format!("  seek {:.1}ms", stats.video_decode_seek_ms));
        lines.push(format!("  pkt {:.1}ms", stats.video_decode_packet_ms));
        lines.push(format!("  xfer {:.1}ms", stats.video_decode_transfer_ms));
        lines.push(format!("  scale {:.1}ms", stats.video_decode_scale_ms));
        lines.push(format!("  copy {:.1}ms", stats.video_decode_copy_ms));
        lines.push(hw_label);
        lines.push(format!("still {:.1}ms", stats.still_load_ms));
        lines.push(format!("comp {:.1}ms", stats.composite_ms));
        lines.push(format!("upload {:.1}ms", stats.encode_ms));
        if let Some(gpu_ms) = preview_gpu_upload_ms {
            lines.push(format!("gpu {:.1}ms", gpu_ms));
        }
        lines.push(format!("hit {:.0}%", hit_ratio));
        lines.push(format!("layers {}", stats.layers));
        lines.join("\n")
        })
    } else {
        None
    };
    let stats_text = stats_text.unwrap_or_default();
    let show_stats_overlay = show_preview_stats && !stats_text.is_empty();
    rsx! {
        div {
            style: "display: flex; flex-direction: column; flex: 1; min-height: 0; background-color: {BG_DEEPEST};",

            div {
                style: "
                    display: grid; grid-template-columns: auto 1fr auto; align-items: center;
                    height: 32px; padding: 0 14px;
                    background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                ",
                span {
                    style: "grid-column: 1; font-size: 11px; font-weight: 500; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px;",
                    "Preview"
                }
                span {
                    style: "
                        grid-column: 2; justify-self: center; min-width: 0;
                        font-family: 'SF Mono', Consolas, monospace;
                        font-size: 10px; color: {TEXT_DIM};
                        white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
                    ",
                    ""
                }
                div {
                    style: "grid-column: 3; justify-self: end; display: flex; align-items: center; gap: 6px; font-family: 'SF Mono', Consolas, monospace; font-size: 11px; color: {TEXT_DIM};",
                    span { "{width} x {height}" }
                    span { style: "color: {TEXT_MUTED};", "@" }
                    span { "{fps_label}" }
                }
            }

            div {
                style: "flex: 1; display: flex; background-color: {BG_DEEPEST}; padding: 0; position: relative; min-height: 0; overflow: hidden;",
                div {
                    style: "position: relative; flex: 1; display: flex; align-items: center; justify-content: center; min-height: 0;",
                    div {
                        id: "preview-native-host",
                        style: "position: absolute; inset: 0; background-color: transparent; pointer-events: none; z-index: 0;",
                    }
                    canvas {
                        id: "preview-canvas",
                        width: "1",
                        height: "1",
                        style: "position: relative; z-index: 1; max-width: 100%; max-height: 100%; width: auto; height: auto; border: none; border-radius: 0; background-color: #000; visibility: {canvas_visibility};",
                    }
                    if show_placeholder {
                        div {
                            style: "display: flex; flex-direction: column; align-items: center; gap: 12px; color: {TEXT_DIM};",
                            div {
                                style: "width: 48px; height: 48px; border: 1px solid {BORDER_DEFAULT}; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 14px;",
                                "?"
                            }
                            span { style: "font-size: 12px;", "No preview" }
                        }
                    }
                }
                if show_stats_overlay {
                    div {
                        style: "
                            width: 200px; padding: 10px 12px; border-left: 1px solid {BORDER_SUBTLE};
                            background-color: {BG_SURFACE};
                            font-family: 'SF Mono', Consolas, monospace;
                            font-size: 10px; color: {TEXT_DIM};
                            white-space: pre; user-select: text; cursor: text;
                            overflow: auto;
                        ",
                        "{stats_text}"
                    }
                }
            }
        }
    }
}

#[component]
fn StatusBar() -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; height: 22px; padding: 0 14px; background-color: {BG_SURFACE}; border-top: 1px solid {BORDER_DEFAULT}; font-size: 11px; color: {TEXT_DIM};",
            span { "Ready" }
            div {
                style: "display: flex; gap: 16px; font-family: 'SF Mono', Consolas, monospace;",
                span { "60 fps" }
                span { "00:00 / 00:00" }
            }
        }
    }
}

fn spawn_asset_duration_probe(
    mut project: Signal<crate::state::Project>,
    asset_id: uuid::Uuid,
) {
    let (project_root, asset_path, needs_probe) = {
        let project_read = project.read();
        let project_root = project_read.project_path.clone();
        let asset = project_read.find_asset(asset_id);
        let needs_probe = asset
            .map(|asset| asset.duration_seconds.is_none() && (asset.is_video() || asset.is_audio()))
            .unwrap_or(false);
        let asset_path = asset.and_then(|asset| match &asset.kind {
            crate::state::AssetKind::Video { path } => Some(path.clone()),
            crate::state::AssetKind::Audio { path } => Some(path.clone()),
            _ => None,
        });
        (project_root, asset_path, needs_probe)
    };

    let Some(project_root) = project_root else { return; };
    let Some(asset_path) = asset_path else { return; };
    if !needs_probe {
        return;
    }

    let absolute_path = project_root.join(asset_path);

    spawn(async move {
        let duration = tokio::task::spawn_blocking(move || {
            crate::core::media::probe_duration_seconds(&absolute_path)
        })
        .await
        .ok()
        .flatten();

        if let Some(duration) = duration {
            project.write().set_asset_duration(asset_id, Some(duration));
        }
    });
}

fn spawn_missing_duration_probes(project: Signal<crate::state::Project>) {
    let asset_ids: Vec<uuid::Uuid> = project
        .read()
        .assets
        .iter()
        .filter(|asset| asset.duration_seconds.is_none() && (asset.is_video() || asset.is_audio()))
        .map(|asset| asset.id)
        .collect();

    for asset_id in asset_ids {
        spawn_asset_duration_probe(project, asset_id);
    }
}

fn resolve_asset_duration_seconds(
    mut project: Signal<crate::state::Project>,
    asset_id: uuid::Uuid,
) -> Option<f64> {
    let (project_root, asset_path, cached_duration, should_probe) = {
        let project_read = project.read();
        let project_root = project_read.project_path.clone();
        let asset = project_read.find_asset(asset_id);
        let cached_duration = asset.and_then(|asset| asset.duration_seconds);
        let should_probe = asset
            .map(|asset| asset.is_video() || asset.is_audio())
            .unwrap_or(false);
        let asset_path = asset.and_then(|asset| match &asset.kind {
            crate::state::AssetKind::Video { path } => Some(path.clone()),
            crate::state::AssetKind::Audio { path } => Some(path.clone()),
            _ => None,
        });
        (project_root, asset_path, cached_duration, should_probe)
    };

    if let Some(duration) = cached_duration {
        return Some(duration);
    }

    if !should_probe {
        return None;
    }

    let Some(project_root) = project_root else { return None; };
    let Some(asset_path) = asset_path else { return None; };

    let absolute_path = project_root.join(asset_path);
    let duration = crate::core::media::probe_duration_seconds(&absolute_path);
    if let Some(duration) = duration {
        project.write().set_asset_duration(asset_id, Some(duration));
        return Some(duration);
    }

    None
}

fn parse_f32_input(value: &str, fallback: f32) -> f32 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return fallback;
    }
    trimmed.parse::<f32>().unwrap_or(fallback)
}

fn update_clip_transform(
    mut project: Signal<crate::state::Project>,
    clip_id: uuid::Uuid,
    update: impl FnOnce(&mut crate::state::ClipTransform),
) {
    if let Some(clip) = project.write().clips.iter_mut().find(|clip| clip.id == clip_id) {
        update(&mut clip.transform);
    }
}

/// Assets panel content - displays project assets and import functionality
#[component]
fn AssetsPanelContent(
    assets: Vec<crate::state::Asset>,
    thumbnailer: std::sync::Arc<crate::core::thumbnailer::Thumbnailer>,
    thumbnail_cache_buster: u64,
    thumbnail_refresh_tick: u64,
    on_import: EventHandler<crate::state::Asset>,
    on_import_file: EventHandler<std::path::PathBuf>,
    on_delete: EventHandler<uuid::Uuid>,
    on_regenerate_thumbnails: EventHandler<uuid::Uuid>,
    on_add_to_timeline: EventHandler<uuid::Uuid>,
    on_drag_start: EventHandler<uuid::Uuid>,
) -> Element {
    let _ = thumbnail_refresh_tick;
    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; padding: 8px;",
            
            // Import button
            button {
                style: "
                    width: 100%; padding: 8px 12px; margin-bottom: 8px;
                    background-color: {BG_SURFACE}; border: 1px dashed {BORDER_DEFAULT};
                    border-radius: 6px; color: {TEXT_SECONDARY}; font-size: 12px;
                    cursor: pointer; transition: all 0.15s ease;
                ",
                onclick: move |_| {
                    // Use rfd for native file dialog
                    if let Some(paths) = rfd::FileDialog::new()
                        .add_filter("Media Files", &["mp4", "mov", "avi", "mp3", "wav", "png", "jpg", "jpeg", "gif", "webp"])
                        .add_filter("Video", &["mp4", "mov", "avi", "mkv", "webm"])
                        .add_filter("Audio", &["mp3", "wav", "ogg", "flac"])
                        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                        .set_title("Import Assets")
                        .pick_files()
                    {
                        for path in paths {
                            on_import_file.call(path);
                        }
                    }
                },
                "📁 Import Files..."
            }
            
            // Generative asset buttons
            div {
                style: "
                    display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px;
                    padding: 8px; background-color: {BG_SURFACE}; border-radius: 6px;
                    border: 1px solid {BORDER_SUBTLE};
                ",
                div { 
                    style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px;",
                    "✨ New Generative"
                }
                div {
                    style: "display: flex; gap: 4px;",
                    
                    // Generative Video button
                    button {
                        style: "
                            flex: 1; padding: 6px 8px;
                            background: transparent; border: 1px dashed {ACCENT_VIDEO};
                            border-radius: 4px; color: {ACCENT_VIDEO}; font-size: 11px;
                            cursor: pointer; transition: all 0.15s ease;
                        ",
                        onclick: {
                            let on_import = on_import.clone();
                            move |_| {
                                // Generate unique ID for this generative asset
                                let id = uuid::Uuid::new_v4();
                                let folder = std::path::PathBuf::from(format!("generated/video/{}", id));
                                let asset = crate::state::Asset::new_generative_video(
                                    format!("Gen Video {}", &id.to_string()[..8]),
                                    folder
                                );
                                on_import.call(asset);
                            }
                        },
                        "🎬 Video"
                    }
                    
                    // Generative Image button
                    button {
                        style: "
                            flex: 1; padding: 6px 8px;
                            background: transparent; border: 1px dashed {ACCENT_VIDEO};
                            border-radius: 4px; color: {ACCENT_VIDEO}; font-size: 11px;
                            cursor: pointer; transition: all 0.15s ease;
                        ",
                        onclick: {
                            let on_import = on_import.clone();
                            move |_| {
                                let id = uuid::Uuid::new_v4();
                                let folder = std::path::PathBuf::from(format!("generated/image/{}", id));
                                let asset = crate::state::Asset::new_generative_image(
                                    format!("Gen Image {}", &id.to_string()[..8]),
                                    folder
                                );
                                on_import.call(asset);
                            }
                        },
                        "🖼️ Image"
                    }
                    
                    // Generative Audio button
                    button {
                        style: "
                            flex: 1; padding: 6px 8px;
                            background: transparent; border: 1px dashed {ACCENT_AUDIO};
                            border-radius: 4px; color: {ACCENT_AUDIO}; font-size: 11px;
                            cursor: pointer; transition: all 0.15s ease;
                        ",
                        onclick: {
                            let on_import = on_import.clone();
                            move |_| {
                                let id = uuid::Uuid::new_v4();
                                let folder = std::path::PathBuf::from(format!("generated/audio/{}", id));
                                let asset = crate::state::Asset::new_generative_audio(
                                    format!("Gen Audio {}", &id.to_string()[..8]),
                                    folder
                                );
                                on_import.call(asset);
                            }
                        },
                        "🔊 Audio"
                    }
                }
            }
            // Asset list
            div {
                style: "flex: 1; overflow-y: auto;",
                
                if assets.is_empty() {
                    div {
                        style: "
                            display: flex; flex-direction: column; align-items: center; justify-content: center;
                            height: 120px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                            color: {TEXT_DIM}; font-size: 12px; text-align: center; padding: 12px;
                        ",
                        div { style: "font-size: 24px; margin-bottom: 8px;", "📂" }
                        "No assets yet"
                        div { style: "font-size: 10px; color: {TEXT_DIM}; margin-top: 4px;", "Import files or create generative assets" }
                    }
                } else {
                    for asset in assets.iter() {
                        AssetItem { 
                            asset: asset.clone(),
                            thumbnailer: thumbnailer.clone(),
                            thumbnail_cache_buster: thumbnail_cache_buster,
                            on_delete: move |id| on_delete.call(id),
                            on_regenerate_thumbnails: move |id| on_regenerate_thumbnails.call(id),
                            on_add_to_timeline: move |id| on_add_to_timeline.call(id),
                            on_drag_start: move |id| on_drag_start.call(id),
                        }
                    }
                }
            }
        }
    }
}

/// Individual asset item in the list
#[component]
fn AssetItem(
    asset: crate::state::Asset,
    thumbnailer: std::sync::Arc<crate::core::thumbnailer::Thumbnailer>,
    thumbnail_cache_buster: u64,
    on_delete: EventHandler<uuid::Uuid>,
    on_regenerate_thumbnails: EventHandler<uuid::Uuid>,
    on_add_to_timeline: EventHandler<uuid::Uuid>,
    on_drag_start: EventHandler<uuid::Uuid>,
) -> Element {
    let mut show_menu = use_signal(|| false);
    let mut menu_pos = use_signal(|| (0.0, 0.0));

    // Icon based on asset type
    let icon = match &asset.kind {
        crate::state::AssetKind::Video { .. } => "🎬",
        crate::state::AssetKind::Image { .. } => "🖼️",
        crate::state::AssetKind::Audio { .. } => "🔊",
        crate::state::AssetKind::GenerativeVideo { .. } => "✨🎬",
        crate::state::AssetKind::GenerativeImage { .. } => "✨🖼️",
        crate::state::AssetKind::GenerativeAudio { .. } => "✨🔊",
    };
    
    // Color accent based on type
    let accent = match &asset.kind {
        crate::state::AssetKind::Video { .. } | crate::state::AssetKind::GenerativeVideo { .. } => ACCENT_VIDEO,
        crate::state::AssetKind::Audio { .. } | crate::state::AssetKind::GenerativeAudio { .. } => ACCENT_AUDIO,
        crate::state::AssetKind::Image { .. } | crate::state::AssetKind::GenerativeImage { .. } => ACCENT_VIDEO,
    };
    
    let thumb_url = if asset.is_visual() {
        thumbnailer.get_thumbnail_path(asset.id, 0.0).map(|p| {
            let url = crate::utils::get_local_file_url(&p);
            format!("{}?v={}", url, thumbnail_cache_buster)
        })
    } else {
        None
    };
    
    // Generative assets have a subtle dashed border
    let border_style = if asset.is_generative() {
        format!("1px dashed {}", BORDER_DEFAULT)  // Subtle dashed, not accent-colored
    } else {
        format!("1px solid {}", BORDER_SUBTLE)
    };

    let asset_id = asset.id;
    let asset_name = asset.name.clone();
    
    rsx! {
        div {
            style: "position: relative;",
            
            div {
                style: "
                    display: flex; align-items: center; gap: 8px;
                    padding: 8px; margin-bottom: 4px;
                    background-color: {BG_SURFACE}; border: {border_style}; border-radius: 4px;
                    cursor: grab; transition: background-color 0.1s ease;
                    user-select: none;
                ",
                oncontextmenu: move |e| {
                    e.prevent_default();
                    let coords = e.client_coordinates();
                    menu_pos.set((coords.x, coords.y));
                    show_menu.set(true);
                },
                onmousedown: move |e| {
                    // Left click starts drag
                    e.prevent_default(); // prevent browser default drag (we use our own)
                    on_drag_start.call(asset_id);
                },
                // Type indicator
                div {
                    style: "width: 3px; height: 24px; border-radius: 2px; background-color: {accent};",
                }
                // Thumbnail + icon
                div {
                    style: "
                        width: 36px; height: 24px; border-radius: 3px; overflow: hidden;
                        background-color: {BG_BASE}; border: 1px solid {BORDER_SUBTLE};
                        display: flex; align-items: center; justify-content: center;
                        position: relative; flex-shrink: 0;
                    ",
                    if let Some(src_url) = thumb_url.clone() {
                        img {
                            src: "{src_url}",
                            style: "width: 100%; height: 100%; object-fit: cover; pointer-events: none;",
                            draggable: "false",
                        }
                        span {
                            style: "
                                position: absolute; right: 2px; bottom: 2px;
                                font-size: 9px; color: {TEXT_PRIMARY};
                                background-color: rgba(0,0,0,0.6); padding: 1px 3px;
                                border-radius: 3px; pointer-events: none;
                            ",
                            "{icon}"
                        }
                    } else {
                        span { style: "font-size: 12px; color: {TEXT_MUTED}; pointer-events: none;", "{icon}" }
                    }
                }
                // Name
                span { 
                    style: "flex: 1; font-size: 12px; color: {TEXT_PRIMARY}; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{asset_name}" 
                }
            }
            
            // Context menu for this asset
            if show_menu() {
                // Backdrop
                div {
                    style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 999;",
                    onclick: move |_| show_menu.set(false),
                }
                // Menu
                {
                    let (x, y) = menu_pos();
                    rsx! {
                        div {
                            style: "
                                position: fixed; 
                                left: min({x}px, calc(100vw - 140px)); 
                                top: min({y}px, calc(100vh - 50px));
                                background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                                border-radius: 6px; padding: 4px 0; min-width: 120px;
                                box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                                z-index: 1000; font-size: 12px;
                            ",
                            // Add to timeline option
                            div {
                                style: "
                                    padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                                    transition: background-color 0.1s ease;
                                ",
                                onclick: move |_| {
                                    on_add_to_timeline.call(asset_id);
                                    show_menu.set(false);
                                },
                                "➕ Add to Timeline"
                            }
                             // Regenerate Thumbnails
                            div {
                                style: "
                                    padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                                    transition: background-color 0.1s ease;
                                ",
                                onclick: move |_| {
                                    on_regenerate_thumbnails.call(asset_id);
                                    show_menu.set(false);
                                },
                                "🔄 Refresh Thumbnails"
                            }
                            // Divider
                            div {
                                style: "height: 1px; background-color: {BORDER_SUBTLE}; margin: 4px 0;",
                            }
                            // Delete option
                            div {
                                style: "
                                    padding: 6px 12px; color: #ef4444; cursor: pointer;
                                    transition: background-color 0.1s ease;
                                ",
                                onclick: move |_| {
                                    on_delete.call(asset_id);
                                    show_menu.set(false);
                                },
                                "🗑 Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}





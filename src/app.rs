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
use crate::core::media::{resolve_asset_duration_seconds, spawn_asset_duration_probe, spawn_missing_duration_probes};
use crate::core::preview_gpu::{PreviewBounds, PreviewGpuSurface};
use crate::core::provider_store::{
    default_provider_entry,
    list_global_provider_files,
    load_global_provider_entries_or_empty,
    provider_path_for_entry,
    read_provider_file,
    write_provider_file,
};
use crate::state::{ensure_generative_config, ProviderEntry};
use crate::timeline::{timeline_zoom_bounds, TimelinePanel};
use crate::hotkeys::{handle_hotkey, HotkeyAction, HotkeyContext, HotkeyResult};
use crate::constants::*;
use crate::components::{
    NewProjectModal, PreviewPanel, ProvidersModal, SidePanel, StartupModal, StatusBar,
    TitleBar, TrackContextMenu,
};
use crate::components::assets::AssetsPanelContent;
use crate::components::attributes::AttributesPanelContent;


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
    let mut provider_entries = use_signal(|| Vec::<ProviderEntry>::new());
    
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
    let mut preview_native_suspended = use_signal(|| false);
    let preview_gpu = use_hook(|| Rc::new(RefCell::new(None::<PreviewGpuSurface>)));
    let mut show_preview_stats = use_signal(|| false);
    let mut use_hw_decode = use_signal(|| true);
    let timeline_viewport_width = use_signal(|| None::<f64>);
    let mut timeline_viewport_eval = use_signal(|| None::<document::Eval>);
    let mut timeline_zoom_initialized = use_signal(|| false);
    let mut last_project_path = use_signal(|| None::<std::path::PathBuf>);
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

    use_effect(move || {
        let (min_zoom, max_zoom) = timeline_zoom_bounds(
            project.read().duration(),
            timeline_viewport_width(),
            project.read().settings.fps,
        );
        let current_zoom = zoom();
        let clamped = current_zoom.clamp(min_zoom, max_zoom);
        if (clamped - current_zoom).abs() > 0.01 {
            zoom.set(clamped);
        }
    });

    use_effect(move || {
        let current_path = project.read().project_path.clone();
        if current_path != last_project_path() {
            last_project_path.set(current_path);
            timeline_zoom_initialized.set(false);
        }
    });

    use_effect(move || {
        if timeline_zoom_initialized() {
            return;
        }
        let Some(_width) = timeline_viewport_width() else {
            return;
        };
        if project.read().project_path.is_none() {
            return;
        }
        let (min_zoom, _max_zoom) = timeline_zoom_bounds(
            project.read().duration(),
            timeline_viewport_width(),
            project.read().settings.fps,
        );
        zoom.set(min_zoom);
        timeline_zoom_initialized.set(true);
    });
    
    // Drag state
    let mut dragging = use_signal(|| None::<&'static str>);
    let mut drag_start_pos = use_signal(|| 0.0);
    let mut drag_start_size = use_signal(|| 0.0);
    
    // Asset Drag & Drop state
    let mut dragged_asset = use_signal(|| None::<uuid::Uuid>);
    let mut mouse_pos = use_signal(|| (0.0, 0.0));
    let mut selection = use_signal(|| crate::state::SelectionState::default());
    let attributes_key = selection
        .read()
        .primary_clip()
        .map(|id| id.to_string())
        .unwrap_or_else(|| "none".to_string());
    
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
            let mut last_interaction = Instant::now();
            loop {
                tokio::time::sleep(Duration::from_millis(PREVIEW_FRAME_INTERVAL_MS)).await;

                let time = current_time();
                let dirty = preview_dirty();
                let time_changed = (time - last_time).abs() >= 0.0001;

                if !is_playing() && (time_changed || dirty) {
                    last_interaction = Instant::now();
                }

                if !is_playing()
                    && !dirty
                    && last_interaction.elapsed()
                        >= Duration::from_millis(PREVIEW_IDLE_PREFETCH_DELAY_MS)
                {
                    if let Ok(prefetch_permit) = prefetch_gate.clone().try_acquire_owned() {
                        let project_snapshot = project.read().clone();
                        let renderer = previewer.read().clone();
                        let allow_hw_decode = use_hw_decode();
                        let fps = project_snapshot.settings.fps.max(1.0);
                        let ahead_frames =
                            (fps * PREVIEW_IDLE_PREFETCH_AHEAD_SECONDS).round() as u32;
                        let behind_frames =
                            (fps * PREVIEW_IDLE_PREFETCH_BEHIND_SECONDS).round() as u32;
                        tokio::task::spawn_blocking(move || {
                            if ahead_frames > 0 {
                                renderer.prefetch_frames(
                                    &project_snapshot,
                                    time,
                                    1,
                                    ahead_frames,
                                    crate::core::preview::PreviewDecodeMode::Sequential,
                                    allow_hw_decode,
                                );
                            }
                            if behind_frames > 0 {
                                renderer.prefetch_frames(
                                    &project_snapshot,
                                    time,
                                    -1,
                                    behind_frames,
                                    crate::core::preview::PreviewDecodeMode::Sequential,
                                    allow_hw_decode,
                                );
                            }
                            drop(prefetch_permit);
                        });
                    }
                }

                if !dirty && !time_changed {
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

    use_effect(move || {
        if timeline_viewport_eval().is_some() {
            return;
        }
        let eval = document::eval(TIMELINE_VIEWPORT_SCRIPT);
        timeline_viewport_eval.set(Some(eval));
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

    use_future(move || {
        let mut timeline_viewport_width = timeline_viewport_width.clone();
        let timeline_viewport_eval = timeline_viewport_eval.clone();
        async move {
            loop {
                let Some(eval) = timeline_viewport_eval() else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                };
                let mut eval = eval;
                loop {
                    match eval.recv::<f64>().await {
                        Ok(width) => {
                            let width = width.max(0.0);
                            if timeline_viewport_width() != Some(width) {
                                timeline_viewport_width.set(Some(width));
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
        let preview_native_suspended = preview_native_suspended.clone();
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

            if preview_native_suspended() {
                if let Some(gpu) = preview_gpu.borrow_mut().as_mut() {
                    gpu.clear_layers();
                }
                if preview_native_active() {
                    preview_native_active.set(false);
                }
                if preview_native_uploaded().is_some() {
                    preview_native_uploaded.set(None);
                }
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
    let show_providers_dialog = use_signal(|| false);
    let provider_editor_path = use_signal(|| None::<std::path::PathBuf>);
    let provider_editor_text = use_signal(String::new);
    let provider_editor_error = use_signal(|| None::<String>);
    let provider_editor_dirty = use_signal(|| false);
    let provider_files = use_signal(|| Vec::<std::path::PathBuf>::new());

    let mut open_providers_dialog = {
        let mut show_providers_dialog = show_providers_dialog.clone();
        let mut provider_entries = provider_entries.clone();
        let mut provider_files = provider_files.clone();
        let mut provider_editor_path = provider_editor_path.clone();
        let mut provider_editor_text = provider_editor_text.clone();
        let mut provider_editor_error = provider_editor_error.clone();
        let mut provider_editor_dirty = provider_editor_dirty.clone();
        move || {
            provider_entries.set(load_global_provider_entries_or_empty());
            let files = list_global_provider_files();
            provider_files.set(files.clone());
            provider_editor_error.set(None);
            provider_editor_dirty.set(false);
            if let Some(first) = files.first() {
                provider_editor_path.set(Some(first.clone()));
                provider_editor_text.set(read_provider_file(first).unwrap_or_default());
            } else {
                provider_editor_path.set(None);
                provider_editor_text.set(String::new());
            }
            show_providers_dialog.set(true);
        }
    };

    let on_provider_reload = {
        let mut provider_entries = provider_entries.clone();
        let mut provider_files = provider_files.clone();
        let mut provider_editor_path = provider_editor_path.clone();
        let mut provider_editor_text = provider_editor_text.clone();
        let mut provider_editor_error = provider_editor_error.clone();
        let mut provider_editor_dirty = provider_editor_dirty.clone();
        move |_: MouseEvent| {
            let files = list_global_provider_files();
            provider_files.set(files.clone());
            provider_entries.set(load_global_provider_entries_or_empty());
            provider_editor_error.set(None);
            provider_editor_dirty.set(false);
            let selected = provider_editor_path().filter(|path| path.exists());
            let next = selected.or_else(|| files.first().cloned());
            if let Some(path) = next {
                provider_editor_path.set(Some(path.clone()));
                provider_editor_text.set(read_provider_file(&path).unwrap_or_default());
            } else {
                provider_editor_path.set(None);
                provider_editor_text.set(String::new());
            }
        }
    };

    let on_provider_new = {
        let mut provider_entries = provider_entries.clone();
        let mut provider_files = provider_files.clone();
        let mut provider_editor_path = provider_editor_path.clone();
        let mut provider_editor_text = provider_editor_text.clone();
        let mut provider_editor_error = provider_editor_error.clone();
        let mut provider_editor_dirty = provider_editor_dirty.clone();
        move |_: MouseEvent| {
            let entry = default_provider_entry();
            let json = serde_json::to_string_pretty(&entry).unwrap_or_else(|_| "{}".to_string());
            let path = provider_path_for_entry(&entry);
            if let Err(err) = write_provider_file(&path, &json) {
                provider_editor_error.set(Some(format!("Failed to create provider: {}", err)));
                return;
            }
            provider_editor_path.set(Some(path));
            provider_editor_text.set(json);
            provider_editor_error.set(None);
            provider_editor_dirty.set(false);
            provider_entries.set(load_global_provider_entries_or_empty());
            provider_files.set(list_global_provider_files());
        }
    };

    let on_provider_save = {
        let mut provider_entries = provider_entries.clone();
        let mut provider_files = provider_files.clone();
        let mut provider_editor_path = provider_editor_path.clone();
        let mut provider_editor_error = provider_editor_error.clone();
        let mut provider_editor_dirty = provider_editor_dirty.clone();
        let provider_editor_text = provider_editor_text.clone();
        move |_: MouseEvent| {
            let text = provider_editor_text();
            let entry: ProviderEntry = match serde_json::from_str(&text) {
                Ok(entry) => entry,
                Err(err) => {
                    provider_editor_error.set(Some(format!("Invalid JSON: {}", err)));
                    return;
                }
            };
            let path = provider_editor_path().unwrap_or_else(|| provider_path_for_entry(&entry));
            if let Err(err) = write_provider_file(&path, &text) {
                provider_editor_error.set(Some(format!("Failed to save provider: {}", err)));
                return;
            }
            provider_editor_path.set(Some(path));
            provider_editor_error.set(None);
            provider_editor_dirty.set(false);
            provider_entries.set(load_global_provider_entries_or_empty());
            provider_files.set(list_global_provider_files());
        }
    };

    let on_provider_delete = {
        let mut provider_entries = provider_entries.clone();
        let mut provider_files = provider_files.clone();
        let mut provider_editor_path = provider_editor_path.clone();
        let mut provider_editor_text = provider_editor_text.clone();
        let mut provider_editor_error = provider_editor_error.clone();
        let mut provider_editor_dirty = provider_editor_dirty.clone();
        move |_: MouseEvent| {
            let Some(path) = provider_editor_path() else {
                provider_editor_error.set(Some("No provider selected.".to_string()));
                return;
            };
            if let Err(err) = std::fs::remove_file(&path) {
                provider_editor_error.set(Some(format!("Failed to delete provider: {}", err)));
                return;
            }
            provider_entries.set(load_global_provider_entries_or_empty());
            let files = list_global_provider_files();
            provider_files.set(files.clone());
            provider_editor_error.set(None);
            provider_editor_dirty.set(false);
            if let Some(first) = files.first() {
                provider_editor_path.set(Some(first.clone()));
                provider_editor_text.set(read_provider_file(first).unwrap_or_default());
            } else {
                provider_editor_path.set(None);
                provider_editor_text.set(String::new());
            }
        }
    };

    let desktop_for_modal_redraw = desktop.clone();
    let preview_gpu_for_modal = preview_gpu.clone();
    use_effect(move || {
        let suspended = show_providers_dialog() || show_new_project_dialog();
        if preview_native_suspended() == suspended {
            return;
        }
        preview_native_suspended.set(suspended);
        if suspended {
            if let Some(gpu) = preview_gpu_for_modal.borrow_mut().as_mut() {
                gpu.clear_layers();
            }
            if preview_native_active() {
                preview_native_active.set(false);
            }
            if preview_native_uploaded().is_some() {
                preview_native_uploaded.set(None);
            }
        } else {
            desktop_for_modal_redraw.window.request_redraw();
        }
    });
    
    // On first load, if project has no path effectively, treat as "No Project Loaded"
    // But since we initialize with default(), we need a flag to block interaction until New/Open
    // We'll use specific "show_startup_modal" derived state
    
    let show_startup = project.read().project_path.is_none() && !startup_done();
    let providers_root_label = crate::core::provider_store::global_providers_root()
        .display()
        .to_string();
    let provider_save_label = if provider_editor_dirty() { "Save *" } else { "Save" };
    let provider_selected_label = provider_editor_path()
        .as_ref()
        .and_then(|path| path.file_name().and_then(|name| name.to_str()))
        .unwrap_or("No provider")
        .to_string();

    // Read current values
    let left_w = if left_collapsed() { PANEL_COLLAPSED_WIDTH } else { left_width() };
    let right_w = if right_collapsed() { PANEL_COLLAPSED_WIDTH } else { right_width() };
    let timeline_h = if timeline_collapsed() { TIMELINE_COLLAPSED_HEIGHT } else { timeline_height() };
    
    // Is currently dragging? (for cursor and user-select styling)

    // Always disable text selection for UI elements to feel like a native app
    // Specific text areas that need selection (e.g. inputs, logs) must override this
    let user_select_style = "none";
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
            // Enable keyboard focus on this container for hotkeys
            tabindex: "0",
            // Hotkey handler
            onkeydown: move |e: KeyboardEvent| {
                // Build context for hotkey dispatch
                let hotkey_context = HotkeyContext {
                    timeline_visible: !timeline_collapsed(),
                    has_selection: !selection.read().clip_ids.is_empty(),
                    input_focused: false, // TODO: track when input fields have focus
                };

                // Get modifier states
                let modifiers = e.modifiers();
                let shift = modifiers.shift();
                let ctrl = modifiers.ctrl();
                let alt = modifiers.alt();
                let meta = modifiers.meta();

                // Dispatch the hotkey
                match handle_hotkey(&e.key(), shift, ctrl, alt, meta, &hotkey_context) {
                    HotkeyResult::Action(action) => {
                        e.prevent_default();
                        match action {
                            HotkeyAction::TimelineZoomIn => {
                                let (min_zoom, max_zoom) = timeline_zoom_bounds(
                                    duration,
                                    timeline_viewport_width(),
                                    project.read().settings.fps,
                                );
                                let new_zoom = (zoom() * 1.25).clamp(min_zoom, max_zoom);
                                zoom.set(new_zoom);
                            }
                            HotkeyAction::TimelineZoomOut => {
                                let (min_zoom, max_zoom) = timeline_zoom_bounds(
                                    duration,
                                    timeline_viewport_width(),
                                    project.read().settings.fps,
                                );
                                let new_zoom = (zoom() * 0.8).clamp(min_zoom, max_zoom);
                                zoom.set(new_zoom);
                            }
                        }
                    }
                    HotkeyResult::NoMatch | HotkeyResult::Suppressed => {}
                }
            },
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
                on_open_providers: move |_| {
                    open_providers_dialog();
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
                        panel_width: left_w,
                        on_import: move |asset: crate::state::Asset| {
                            let project_root = project.read().project_path.clone();
                            project.write().add_asset(asset.clone());
                            if let Some(project_root) = project_root {
                                ensure_generative_config(&project_root, &asset);
                            }
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
                        on_rename: move |(asset_id, name): (uuid::Uuid, String)| {
                            let trimmed = name.trim();
                            if trimmed.is_empty() {
                                return;
                            }
                            project.write().rename_asset(asset_id, trimmed.to_string());
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
                            min_zoom: timeline_zoom_bounds(
                                duration,
                                timeline_viewport_width(),
                                project.read().settings.fps,
                            )
                            .0,
                            max_zoom: timeline_zoom_bounds(
                                duration,
                                timeline_viewport_width(),
                                project.read().settings.fps,
                            )
                            .1,
                            is_playing: is_playing(),
                            scroll_offset: scroll_offset(),
                            // Callbacks
                            on_seek: move |t: f64| {
                                // Snap to frame boundary (60fps) and clamp to duration
                                let snapped = ((t * 60.0).round() / 60.0).clamp(0.0, duration);
                                current_time.set(snapped);
                            },
                            on_zoom_change: move |z: f64| {
                                let (min_zoom, max_zoom) = timeline_zoom_bounds(
                                    duration,
                                    timeline_viewport_width(),
                                    project.read().settings.fps,
                                );
                                zoom.set(z.clamp(min_zoom, max_zoom));
                            },
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
                            // Selection
                            on_deselect_all: move |_| {
                                selection.write().clear();
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
                        key: "{attributes_key}",
                        project: project,
                        selection: selection,
                        preview_dirty: preview_dirty,
                        providers: provider_entries,
                        previewer: previewer,
                        thumbnailer: thumbnailer.read().clone(),
                        thumbnail_cache_buster: thumbnail_cache_buster,
                    }
                }
            }

            StatusBar {}
            
            TrackContextMenu {
                context_menu: context_menu,
                project: project,
                selection: selection,
                preview_dirty: preview_dirty,
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
                                provider_entries.set(load_global_provider_entries_or_empty());
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
                                provider_entries.set(load_global_provider_entries_or_empty());
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

            NewProjectModal {
                show: show_new_project_dialog,
                on_go_to_wizard: move |_| {
                    project.set(crate::state::Project::default());
                    startup_done.set(false);
                    show_new_project_dialog.set(false);
                }
            }

            ProvidersModal {
                show: show_providers_dialog,
                provider_files: provider_files,
                provider_editor_path: provider_editor_path,
                provider_editor_text: provider_editor_text,
                provider_editor_error: provider_editor_error,
                provider_editor_dirty: provider_editor_dirty,
                providers_root_label: providers_root_label.clone(),
                provider_save_label: provider_save_label.to_string(),
                provider_selected_label: provider_selected_label.to_string(),
                on_provider_new: on_provider_new,
                on_provider_reload: on_provider_reload,
                on_provider_save: on_provider_save,
                on_provider_delete: on_provider_delete,
            }
        }
    }
}


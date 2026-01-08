use dioxus::prelude::*;
use crate::constants::*;

#[component]
pub fn PreviewPanel(
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
                            style: "position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: {TEXT_DIM}; z-index: 2;",
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

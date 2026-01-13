use dioxus::prelude::*;
use crate::constants::{BORDER_STRONG, BORDER_SUBTLE, TEXT_DIM};

/// Time ruler with tick marks and labels
/// All elements here use pointer-events: none so clicks pass through to parent
#[component]
pub(crate) fn TimeRuler(duration: f64, zoom: f64, scroll_offset: f64, fps: f64) -> Element {
    let _ = scroll_offset;
    let fps = fps.max(1.0);
    let fps_i = fps.round().max(1.0) as i32;
    // Calculate tick spacing based on zoom level.
    let target_px_per_tick = 90.0;
    let target_seconds = (target_px_per_tick / zoom.max(0.1)).max(0.5);
    let nice_ticks = [
        0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0,
    ];
    let mut seconds_per_major_tick = *nice_ticks.last().unwrap_or(&10.0);
    for tick in nice_ticks {
        if tick >= target_seconds {
            seconds_per_major_tick = tick;
            break;
        }
    }
    
    // Show frame ticks only at high zoom levels (when there's enough space)
    // At 100px/s zoom, each frame is ~1.67px apart - too dense
    // At 300px/s zoom, each frame is 5px apart - usable
    // At 500px/s zoom, each frame is ~8.3px apart - comfortable
    let show_frame_ticks = zoom >= 240.0;
    
    // Generate tick positions
    let num_ticks = (duration / seconds_per_major_tick).ceil() as i32 + 1;
    
    let content_width = duration * zoom;
    let visible_start_time = 0.0;
    let visible_end_time = duration;
    
    rsx! {
        // Entire ruler container ignores pointer events - clicks pass through
        div {
            style: "position: absolute; left: 0; top: 0; width: 100%; height: 100%; pointer-events: none;",
            
            // Frame ticks (subtle, only at high zoom)
            if show_frame_ticks {
                {
                    let start_frame = (visible_start_time * fps).floor() as i32;
                    let end_frame = (visible_end_time * fps).ceil() as i32;
                    
                    rsx! {
                        for frame in start_frame..=end_frame {
                            {
                                let frame_time = frame as f64 / fps;
                                let x = frame_time * zoom;
                                // Skip frame ticks that land on second boundaries
                                let is_on_second = frame % fps_i == 0;
                                
                                if !is_on_second && x <= content_width + 10.0 {
                                    rsx! {
                                        div {
                                            key: "frame-{frame}",
                                            style: "
                                                position: absolute;
                                                left: {x}px;
                                                bottom: 0;
                                                width: 1px;
                                                height: 4px;
                                                background-color: {BORDER_SUBTLE};
                                                pointer-events: none;
                                            ",
                                        }
                                    }
                                } else {
                                    rsx! {}
                                }
                            }
                        }
                    }
                }
            }
            
            // Second/major ticks and labels
            for i in 0..num_ticks {
                {
                    let t = i as f64 * seconds_per_major_tick;
                    let x = t * zoom;
                    let minutes = t as i32 / 60;
                    let seconds = t as i32 % 60;
                    let label = format!("{}:{:02}", minutes, seconds);
                    
                    if x <= content_width + 50.0 {
                        rsx! {
                            // Container for tick + label (key must be on first node)
                            div {
                                key: "tick-group-{i}",
                                // Major tick (second boundary)
                                div {
                                    style: "
                                        position: absolute;
                                        left: {x}px;
                                        bottom: 0;
                                        width: 1px;
                                        height: 10px;
                                        background-color: {BORDER_STRONG};
                                        pointer-events: none;
                                    ",
                                }
                                // Label - right-align last tick to prevent overflow
                                {
                                    // Check if this is the last visible tick
                                    let is_last_tick = i == num_ticks - 1;
                                    let next_tick_x = (i as f64 + 1.0) * seconds_per_major_tick * zoom;
                                    let is_near_end = next_tick_x > content_width;
                                    let should_right_align = is_last_tick || is_near_end;
                                    
                                    // For last label, use transform to shift text left of anchor point
                                    let label_style = if should_right_align {
                                        format!(
                                            "position: absolute; left: {}px; top: 3px; font-size: 9px; color: {}; font-family: 'SF Mono', Consolas, monospace; user-select: none; pointer-events: none; transform: translateX(-100%);",
                                            x - 4.0, TEXT_DIM
                                        )
                                    } else {
                                        format!(
                                            "position: absolute; left: {}px; top: 3px; font-size: 9px; color: {}; font-family: 'SF Mono', Consolas, monospace; user-select: none; pointer-events: none;",
                                            x + 4.0, TEXT_DIM
                                        )
                                    };
                                    rsx! {
                                        div {
                                            style: "{label_style}",
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }
        }
    }
}


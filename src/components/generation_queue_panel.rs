use dioxus::prelude::*;

use crate::constants::*;
use crate::state::{GenerationJob, GenerationJobStatus, ProviderOutputType};

#[component]
pub fn GenerationQueuePanel(
    open: bool,
    jobs: Vec<GenerationJob>,
    on_close: EventHandler<MouseEvent>,
    on_delete_job: EventHandler<uuid::Uuid>,
) -> Element {
    if !open {
        return rsx! {};
    }

    let mut context_menu = use_signal(|| None::<(f64, f64, uuid::Uuid)>);
    let count_label = if jobs.is_empty() {
        "Empty".to_string()
    } else {
        format!("{}", jobs.len())
    };

    rsx! {
        div {
            style: "
                position: fixed; top: 40px; right: 12px;
                width: 320px; max-height: calc(100vh - 60px);
                display: flex; flex-direction: column; gap: 10px;
                padding: 12px; background-color: {BG_ELEVATED};
                border: 1px solid {BORDER_DEFAULT}; border-radius: 10px;
                box-shadow: 0 12px 28px rgba(0,0,0,0.45);
                z-index: 120;
            ",
            div {
                style: "display: flex; align-items: center; justify-content: space-between;",
                div {
                    style: "display: flex; flex-direction: column; gap: 2px;",
                    span { style: "font-size: 12px; color: {TEXT_PRIMARY};", "Generation Queue" }
                    span { style: "font-size: 10px; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.4px;", "{count_label}" }
                }
                button {
                    class: "collapse-btn",
                    style: "
                        padding: 4px 8px; border-radius: 6px;
                        border: 1px solid {BORDER_DEFAULT};
                        background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                        font-size: 11px; cursor: pointer;
                    ",
                    onclick: move |e| on_close.call(e),
                    "Close"
                }
            }

            div {
                style: "display: flex; flex-direction: column; gap: 8px; overflow-y: auto;",
                if jobs.is_empty() {
                    div {
                        style: "
                            padding: 12px; border: 1px dashed {BORDER_DEFAULT};
                            border-radius: 8px; font-size: 11px; color: {TEXT_DIM};
                        ",
                        "No generation jobs yet."
                    }
                } else {
                    for job in jobs.iter().rev() {
                        {
                            let (status_label, status_color) = match job.status {
                                GenerationJobStatus::Queued => ("Queued", TEXT_MUTED),
                                GenerationJobStatus::Running => ("Running", ACCENT_MARKER),
                                GenerationJobStatus::Succeeded => ("Done", ACCENT_VIDEO),
                                GenerationJobStatus::Failed => ("Failed", "#ef4444"),
                            };
                            let output_label = match job.output_type {
                                ProviderOutputType::Image => "Image",
                                ProviderOutputType::Video => "Video",
                                ProviderOutputType::Audio => "Audio",
                            };
                            let progress_percent = job
                                .progress
                                .map(|progress| (progress.clamp(0.0, 1.0) * 100.0).round() as u32)
                                .unwrap_or(0);
                            let job_id = job.id;
                            rsx! {
                                div {
                                    key: "{job.id}",
                                    style: "
                                        display: flex; flex-direction: column; gap: 6px;
                                        padding: 10px; background-color: {BG_SURFACE};
                                        border: 1px solid {BORDER_SUBTLE}; border-radius: 8px;
                                    ",
                                    oncontextmenu: move |e| {
                                        e.prevent_default();
                                        let coords = e.client_coordinates();
                                        context_menu.set(Some((coords.x, coords.y, job_id)));
                                    },
                                    div {
                                        style: "display: flex; align-items: center; justify-content: space-between; gap: 8px;",
                                        span { style: "font-size: 12px; color: {TEXT_PRIMARY};", "{job.asset_label}" }
                                        span {
                                            style: "
                                                padding: 2px 8px; font-size: 9px;
                                                color: {status_color}; border: 1px solid {status_color};
                                                border-radius: 999px; text-transform: uppercase;
                                                letter-spacing: 0.6px;
                                            ",
                                            "{status_label}"
                                        }
                                    }
                                    div {
                                        style: "display: flex; align-items: center; justify-content: space-between;",
                                        span { style: "font-size: 10px; color: {TEXT_MUTED};", "{job.provider.name}" }
                                        span { style: "font-size: 10px; color: {TEXT_DIM};", "{output_label}" }
                                    }
                                    if let Some(version) = job.version.as_ref() {
                                        span { style: "font-size: 10px; color: {TEXT_DIM};", "Output: {version}" }
                                    }
                                    if job.status == GenerationJobStatus::Running {
                                        div {
                                            style: "
                                                height: 6px; border-radius: 999px;
                                                background-color: {BG_BASE}; overflow: hidden;
                                            ",
                                            div {
                                                style: "height: 100%; width: {progress_percent}%; background-color: {ACCENT_MARKER};",
                                            }
                                        }
                                    }
                                    if let Some(error) = job.error.as_ref() {
                                        span { style: "font-size: 10px; color: #fca5a5;", "{error}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some((x, y, job_id)) = context_menu() {
            div {
                style: "
                    position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                    z-index: 130;
                ",
                onclick: move |_| context_menu.set(None),
            }
            div {
                style: "
                    position: fixed;
                    left: min({x}px, calc(100vw - 180px));
                    top: min({y}px, calc(100vh - 140px));
                    background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                    border-radius: 6px; padding: 4px 0; min-width: 160px;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                    z-index: 131; font-size: 12px;
                ",
                {
                    let job = jobs.iter().find(|job| job.id == job_id);
                    if let Some(job) = job {
                        if job.status == GenerationJobStatus::Running {
                            rsx! {
                                div {
                                    style: "
                                        padding: 6px 12px; color: {TEXT_DIM};
                                        cursor: not-allowed;
                                    ",
                                    "Running job (cannot remove)"
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    style: "
                                        padding: 6px 12px; color: #ef4444; cursor: pointer;
                                        transition: background-color 0.1s ease;
                                    ",
                                    onclick: move |_| {
                                        on_delete_job.call(job_id);
                                        context_menu.set(None);
                                    },
                                    "Remove from queue"
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div {
                                style: "
                                    padding: 6px 12px; color: {TEXT_DIM};
                                    cursor: not-allowed;
                                ",
                                "Job not found"
                            }
                        }
                    }
                }
            }
        }
    }
}

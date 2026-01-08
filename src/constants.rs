//! Shared UI constants such as colors, panel sizing, and scripts.
//! These values were previously defined in `app.rs` and now live in a dedicated module.

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

pub const PANEL_MIN_WIDTH: f64 = 180.0;
pub const PANEL_MAX_WIDTH: f64 = 400.0;
pub const PANEL_DEFAULT_WIDTH: f64 = 250.0;
pub const PANEL_COLLAPSED_WIDTH: f64 = 40.0;
pub const TIMELINE_MIN_HEIGHT: f64 = 100.0;
pub const TIMELINE_MAX_HEIGHT: f64 = 500.0;
pub const TIMELINE_DEFAULT_HEIGHT: f64 = 220.0;
pub const TIMELINE_COLLAPSED_HEIGHT: f64 = 32.0;
pub const DEFAULT_CLIP_DURATION_SECONDS: f64 = 2.0;
pub const PREVIEW_FPS: u64 = 24;
pub const PREVIEW_FRAME_INTERVAL_MS: u64 = 1000 / PREVIEW_FPS;
pub const PREVIEW_CACHE_BUDGET_BYTES: usize = 8usize * 1024 * 1024 * 1024;
pub const PREVIEW_PREFETCH_SCRUB_SECONDS: f64 = 0.5;
pub const PREVIEW_PREFETCH_PLAYBACK_SECONDS: f64 = 3.0;
pub const PREVIEW_IDLE_PREFETCH_DELAY_MS: u64 = 800;
pub const PREVIEW_IDLE_PREFETCH_AHEAD_SECONDS: f64 = 5.0;
pub const PREVIEW_IDLE_PREFETCH_BEHIND_SECONDS: f64 = 1.0;
pub const SHOW_CACHE_TICKS: bool = false;
pub const TIMELINE_MIN_ZOOM_FLOOR: f64 = 0.1;
pub const TIMELINE_MAX_PX_PER_FRAME: f64 = 8.0;

pub const PREVIEW_CANVAS_SCRIPT: &str = r#"
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

pub const PREVIEW_NATIVE_HOST_SCRIPT: &str = r#"
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

pub const TIMELINE_VIEWPORT_SCRIPT: &str = r#"
const hostId = "timeline-scroll-host";
let lastWidth = null;

function sendWidth() {
    const host = document.getElementById(hostId);
    if (!host) {
        return;
    }
    const width = host.clientWidth || 0;
    if (lastWidth !== null && Math.abs(lastWidth - width) < 0.5) {
        return;
    }
    lastWidth = width;
    dioxus.send(width);
}

function attach() {
    const host = document.getElementById(hostId);
    if (!host) {
        setTimeout(attach, 100);
        return;
    }
    const observer = new ResizeObserver(() => sendWidth());
    observer.observe(host);
    window.addEventListener("resize", sendWidth, { passive: true });
    sendWidth();
}

attach();
await new Promise(() => {});
"#;

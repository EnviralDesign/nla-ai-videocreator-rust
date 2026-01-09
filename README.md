<p align="center">
  <img src="media/image.png" alt="NLA AI Video Creator" width="800"/>
</p>

<h1 align="center">🎬 NLA AI Video Creator</h1>

<p align="center">
  <sub><em>(official name coming soon™ — we're open to suggestions)</em></sub>
</p>

<p align="center">
  <strong>A local-first, AI-native video editor for generative content creation.</strong><br/>
  <em>Bring Your Own Workflow. Keep your data. Own your creative pipeline.</em>
</p>

<p align="center">
  <a href="#-whats-this">What's This?</a> •
  <a href="#-current-status">Status</a> •
  <a href="#-comfyui-integration">ComfyUI</a> •
  <a href="#%EF%B8%8F-under-the-hood">Under the Hood</a> •
  <a href="#-get-involved">Get Involved</a>
</p>

---

## 🤔 What's This?

**NLA AI Video Creator** is an open-source desktop app that bridges the gap between AI generation tools and video editing. If you've ever found yourself:

- Juggling between ComfyUI, file explorers, and video editors
- Manually renaming and organizing generated assets
- Wishing you could see your AI-generated clips on a timeline *with* your audio
- Wanting to iterate on generations without losing your creative flow

...then this project is for you.

### The Vision

A purpose-built timeline editor where:

- 🎵 **Audio, images, and video live together** — See your soundtrack alongside AI-generated visuals
- 🔌 **ComfyUI is a first-class citizen** — Connect your local workflows directly to the editor
- 🧠 **Generation happens in-context** — Select a clip, tweak parameters, hit generate, see results
- 💾 **Everything stays local** — Your projects, your machine, your data

> **Philosophy:** This isn't trying to replace Premiere or DaVinci. It's the missing link between "I have cool AI workflows" and "I have a finished video."

---

## 🚧 Current Status

**⚠️ Active Development — Not Production Ready**

This is a passion project in early stages. Things work, things break, APIs change. If you're looking for a polished tool to use *today*, check back later!

**If you're here to:**
- ⭐ Watch the project evolve
- 🛠️ Contribute code or ideas
- 🧪 Experiment with early builds

...you're in the right place. Star the repo to follow along!

### What Works Today

| Feature | Status |
|---------|--------|
| Timeline with tracks (video/audio/markers) | ✅ |
| Drag, resize, and manage clips | ✅ |
| GPU-accelerated preview with transforms | ✅ |
| ComfyUI workflow integration (image gen) | ✅ |
| Generative assets with version history | ✅ |
| Provider Builder UI (no JSON editing required) | ✅ |
| Project save/load | ✅ |

### What's Coming

- [ ] Audio playback & waveform visualization
- [ ] Video generation workflow support (backend mostly complete)
- [ ] Smart input suggestions (timeline as implicit wiring)
- [ ] More provider adapters (fal.ai, Replicate, etc.)
- [ ] Export to video file
- [ ] macOS & Linux builds

See the full roadmap in [docs/PROJECT.md](./docs/PROJECT.md).

---

## 🔌 ComfyUI Integration

This is where things get interesting. **Bring Your Own Workflow™** — your ComfyUI setups become first-class providers in the editor.

### How It Works

1. **Point the app at your local ComfyUI** instance
2. **Use the Provider Builder** to select which workflow inputs to expose (prompts, seeds, steps, CFG, etc.)
3. **Bind parameters** via a visual node browser — no JSON editing required
4. **Generate directly from the timeline** — results land in your project with version history

No vendor lock-in. No cloud dependency. Your workflows, your way.

### Why This Matters

ComfyUI has become the power-user's playground for AI image and video generation. But it's a *workflow tool*, not an *editing tool*. This project aims to be the bridge — letting you orchestrate your ComfyUI outputs in a timeline-based environment without leaving your creative flow.

The provider system is designed to be extensible. ComfyUI is the first adapter, but the architecture supports:
- Custom HTTP endpoints
- Commercial APIs (fal.ai, Replicate, etc.) — planned
- Any backend that can accept parameters and return media

---

## ⚙️ Under the Hood

For the developers curious about what makes this tick — the preview and compositing pipeline is where we've invested significant effort. Here's the architecture:

### 🎞️ Preview Pipeline

The challenge: Dioxus runs in a WebView (WebView2 on Windows), but we need GPU-accelerated video compositing. Our solution bypasses WebView limitations entirely.

```
┌─────────────────────────────────────────────────────────────────┐
│                     Preview Pipeline                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐   │
│  │   FFmpeg     │    │    Frame     │    │   wgpu Native    │   │
│  │   Decode     │── ▶│    Cache     │──▶│   Compositor     │   │
│  │   Workers    │    │   (LRU)      │    │   Surface        │   │
│  └──────────────┘    └──────────────┘    └──────────────────┘   │
│         │                   │                     │             │
│         ▼                   ▼                     ▼             │
│   • In-process decode   • 8GB budget         • Layer stacking   │
│   • HW accel (D3D11VA)  • Prefetch window    • Per-clip xforms  │
│   • Parallel workers    • Latest-wins        • GPU compositing  │
│   • CPU fallback        • Per-asset keying   • Native surface  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Three key components:**

1. **FFmpeg Decode Workers** — In-process video decoding via `ffmpeg-next`. Supports hardware acceleration on Windows (D3D11VA/DXVA2) with automatic CPU fallback. Parallel decode workers keyed by track to avoid decoder contention.

2. **Frame Cache** — LRU cache with an 8GB budget for smooth scrubbing. Prefetch windows (5s ahead, 1s behind) warm the cache when idle. Latest-wins scheduling cancels stale decode jobs when you scrub quickly — only the frames you need get decoded.

3. **wgpu Native Compositor** — A native GPU surface that renders alongside the WebView. Each layer uploads as a texture; shaders handle transforms (position, scale, rotation, opacity) and compositing. This bypasses the DOM entirely for zero-copy GPU rendering.

### 📊 Diagnostic Tools

We built in visibility for optimization work:
- **Preview stats overlay** with per-stage timing (seek, decode, transfer, scale, upload)
- **Hardware decode percentage** — see how much is offloaded to the GPU
- **Cache hit rate** tracking — know when you're hitting vs. missing
- **HW Dec toggle** — force CPU decode for A/B comparisons

> **Work in Progress:** We're still optimizing. The GPU currently receives RGBA after CPU conversion — a future path keeps YUV/NV12 on the GPU to avoid the round-trip. There's headroom to improve.

---

## 🛠️ Tech Stack

| Component | Technology |
|-----------|------------|
| Language | **Rust** — Fast, safe, no runtime |
| UI Framework | **Dioxus 0.7** — Reactive, cross-platform, hot-patching |
| GPU Rendering | **wgpu** — WebGPU-based, cross-platform compositing |
| Video Decode | **FFmpeg** (ffmpeg-next) — In-process decode with HW accel |
| Async | **Tokio** — Background tasks, provider communication |

---

## 📚 Documentation

Detailed docs live in the `/docs` folder:

- **[PROJECT.md](./docs/PROJECT.md)** — Vision, architecture, roadmap, and session changelog
- **[CONTENT_ARCHITECTURE.md](./docs/CONTENT_ARCHITECTURE.md)** — How assets, generation, and the timeline work together
- **[PROVIDER_SETUP_GUIDE.md](./docs/PROVIDER_SETUP_GUIDE.md)** — Setting up ComfyUI and other providers
- **[DECODE-STRATEGIES.md](./docs/DECODE-STRATEGIES.md)** — Deep dive on NLE preview pipeline architecture

> 📝 **Full setup guides coming soon.** For now, adventurous developers can explore the docs and source code.

---

## 🤝 Get Involved

This is an open source project and contributions are welcome!

**Ways to help:**
- ⭐ **Star the repo** — Helps visibility
- 🐛 **Report issues** — Found a bug? Let us know
- 💡 **Suggest features** — Open a discussion
- 🔧 **Contribute code** — PRs welcome

### Areas We'd Love Help With

- Provider adapters for other services (fal.ai, Replicate, etc.)
- macOS and Linux testing/builds

---

## 📜 License

**MIT License** — See [LICENSE](./LICENSE) for details.

Use it, fork it, build on it. 🤖

---

<p align="center">
  <em>Built with 🦀 Rust and ☕ too much coffee</em>
</p>

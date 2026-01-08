//! GPU-accelerated preview rendering
//!
//! Uses wgpu for hardware-accelerated compositing.

mod surface;
mod shaders;
mod types;
mod layers;

pub use surface::PreviewGpuSurface;
pub use types::PreviewBounds;

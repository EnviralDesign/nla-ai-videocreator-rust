use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use crate::core::preview::PreviewLayerPlacement;

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

#[cfg(target_os = "windows")]
impl Vertex {
    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[cfg(target_os = "windows")]
pub(crate) const QUAD_VERTICES: [Vertex; 6] = [
    Vertex {
        position: [0.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0],
        uv: [0.0, 1.0],
    },
];

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LayerUniform {
    scale_center: [f32; 4],
    rotation_opacity: [f32; 4],
}

#[cfg(target_os = "windows")]
impl LayerUniform {
    pub(crate) fn new(
        scale: [f32; 2],
        center: [f32; 2],
        rotation_deg: f32,
        opacity: f32,
        aspect: f32,
    ) -> Self {
        let radians = -rotation_deg.to_radians();
        let (sin, cos) = radians.sin_cos();
        Self {
            scale_center: [scale[0], scale[1], center[0], center[1]],
            rotation_opacity: [cos, sin, opacity, aspect],
        }
    }
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct BorderUniform {
    pub(crate) rect: [f32; 4],  // x, y (NDC top-left corner), w, h (NDC size)
    pub(crate) color: [f32; 4], // rgba
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreviewBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub dpr: f64,
}

impl PreviewBounds {
    pub fn to_physical_position(self) -> dioxus::desktop::tao::dpi::PhysicalPosition<i32> {
        let scale = self.dpr.max(0.01);
        dioxus::desktop::tao::dpi::PhysicalPosition::new(
            (self.x * scale).round() as i32,
            (self.y * scale).round() as i32,
        )
    }

    pub fn to_physical_size(self) -> dioxus::desktop::tao::dpi::PhysicalSize<u32> {
        let scale = self.dpr.max(0.01);
        let width = (self.width * scale).round().max(1.0) as u32;
        let height = (self.height * scale).round().max(1.0) as u32;
        dioxus::desktop::tao::dpi::PhysicalSize::new(width, height)
    }
}

#[cfg(target_os = "windows")]
pub(crate) struct GpuLayer {
    pub(crate) texture: wgpu::Texture,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) uniform_buffer: wgpu::Buffer,
    pub(crate) uniform_bind_group: wgpu::BindGroup,
    pub(crate) size: (u32, u32),
    pub(crate) placement: PreviewLayerPlacement,
}


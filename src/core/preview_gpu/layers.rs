#[cfg(target_os = "windows")]
use dioxus::desktop::tao::dpi::PhysicalSize;

#[cfg(target_os = "windows")]
use crate::core::preview::PreviewLayerPlacement;

#[cfg(target_os = "windows")]
use super::types::{GpuLayer, LayerUniform};

#[cfg(target_os = "windows")]
use wgpu::util::DeviceExt;

#[cfg(target_os = "windows")]
pub(crate) fn create_layer_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("preview_gpu_layer_texture"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[cfg(target_os = "windows")]
pub(crate) fn create_layer(
    device: &wgpu::Device,
    sampler: &wgpu::Sampler,
    texture_layout: &wgpu::BindGroupLayout,
    uniform_layout: &wgpu::BindGroupLayout,
    width: u32,
    height: u32,
    placement: PreviewLayerPlacement,
) -> GpuLayer {
    let (texture, view) =
        create_layer_texture(device, width, height, wgpu::TextureFormat::Rgba8UnormSrgb);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("preview_gpu_layer_texture_bind_group"),
        layout: texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });

    let uniform = LayerUniform::new(
        [0.0, 0.0],
        [0.0, 0.0],
        placement.rotation_deg,
        placement.opacity,
        1.0,
    );
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("preview_gpu_layer_uniform"),
        contents: bytemuck::bytes_of(&uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("preview_gpu_layer_uniform_bind_group"),
        layout: uniform_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    GpuLayer {
        texture,
        bind_group,
        uniform_buffer,
        uniform_bind_group,
        size: (width, height),
        placement,
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn compute_layer_uniform(
    surface_size: PhysicalSize<u32>,
    placement: PreviewLayerPlacement,
    canvas_size: (u32, u32),
) -> Option<LayerUniform> {
    let surface_w = surface_size.width.max(1) as f32;
    let surface_h = surface_size.height.max(1) as f32;
    let canvas_w = canvas_size.0.max(1) as f32;
    let canvas_h = canvas_size.1.max(1) as f32;
    if surface_w <= 0.0 || surface_h <= 0.0 {
        return None;
    }

    let preview_scale = (surface_w / canvas_w).min(surface_h / canvas_h).max(0.0);
    if preview_scale <= 0.0 {
        return None;
    }

    let preview_w = canvas_w * preview_scale;
    let preview_h = canvas_h * preview_scale;
    let offset_x = (surface_w - preview_w) * 0.5;
    let offset_y = (surface_h - preview_h) * 0.5;

    let rect_w = placement.scaled_w * preview_scale;
    let rect_h = placement.scaled_h * preview_scale;
    if rect_w <= 0.0 || rect_h <= 0.0 {
        return None;
    }

    let rect_x = offset_x + placement.offset_x * preview_scale;
    let rect_y = offset_y + placement.offset_y * preview_scale;

    let center_x = rect_x + rect_w * 0.5;
    let center_y = rect_y + rect_h * 0.5;

    let scale_x = rect_w / surface_w * 2.0;
    let scale_y = rect_h / surface_h * 2.0;
    let center_x = center_x / surface_w * 2.0 - 1.0;
    let center_y = 1.0 - center_y / surface_h * 2.0;
    let aspect = surface_w / surface_h;

    Some(LayerUniform::new(
        [scale_x, scale_y],
        [center_x, center_y],
        placement.rotation_deg,
        placement.opacity,
        aspect,
    ))
}

#[cfg(target_os = "windows")]
pub(crate) fn align_to(value: u32, alignment: u32) -> u32 {
    if alignment == 0 {
        return value;
    }
    ((value + alignment - 1) / alignment) * alignment
}

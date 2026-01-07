use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use crate::core::preview::{PreviewLayerPlacement, PreviewLayerStack};
#[cfg(not(target_os = "windows"))]
use crate::core::preview::PreviewLayerStack;
#[cfg(target_os = "windows")]
use dioxus::desktop::tao::platform::windows::WindowExtWindows;
#[cfg(target_os = "windows")]
use std::num::NonZeroU64;
#[cfg(target_os = "windows")]
use wgpu::util::DeviceExt;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOP, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_LAYERED, WS_EX_TRANSPARENT,
};

#[cfg(target_os = "windows")]
const PREVIEW_NATIVE_OFFSET_X: i32 = -8;
#[cfg(target_os = "windows")]
const PREVIEW_NATIVE_OFFSET_Y: i32 = -1;

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

#[cfg(target_os = "windows")]
impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
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
const QUAD_VERTICES: [Vertex; 6] = [
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
struct LayerUniform {
    scale_translate: [f32; 4],
    opacity_pad: [f32; 4],
}

#[cfg(target_os = "windows")]
impl LayerUniform {
    fn new(scale: [f32; 2], translate: [f32; 2], opacity: f32) -> Self {
        Self {
            scale_translate: [scale[0], scale[1], translate[0], translate[1]],
            opacity_pad: [opacity, 0.0, 0.0, 0.0],
        }
    }
}

#[cfg(target_os = "windows")]
const PREVIEW_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct LayerUniform {
    scale_translate: vec4<f32>,
    opacity_pad: vec4<f32>,
};

@group(0) @binding(0)
var layer_tex: texture_2d<f32>;
@group(0) @binding(1)
var layer_sampler: sampler;
@group(1) @binding(0)
var<uniform> layer_uniform: LayerUniform;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let scale = layer_uniform.scale_translate.xy;
    let translate = layer_uniform.scale_translate.zw;
    let pos = input.position * scale + translate;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = input.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(layer_tex, layer_sampler, input.uv);
    return vec4<f32>(color.rgb, color.a * layer_uniform.opacity_pad.x);
}
"#;

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
struct GpuLayer {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    size: (u32, u32),
    placement: PreviewLayerPlacement,
}

#[cfg(target_os = "windows")]
pub struct PreviewGpuSurface {
    window: dioxus::desktop::tao::window::Window,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: dioxus::desktop::tao::dpi::PhysicalSize<u32>,
    position: dioxus::desktop::tao::dpi::PhysicalPosition<i32>,
    sampler: wgpu::Sampler,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    layers: Vec<GpuLayer>,
    canvas_size: (u32, u32),
    upload_scratch: Vec<u8>,
}

#[cfg(target_os = "windows")]
impl PreviewGpuSurface {
    pub fn new<T>(
        parent: &dioxus::desktop::tao::window::Window,
        target: &dioxus::desktop::tao::event_loop::EventLoopWindowTarget<T>,
    ) -> Option<Self> {
        use dioxus::desktop::tao::dpi::LogicalSize;
        use dioxus::desktop::tao::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};
        use dioxus::desktop::tao::window::WindowBuilder;

        let builder = WindowBuilder::new()
            .with_decorations(false)
            .with_resizable(false)
            .with_visible(false)
            .with_inner_size(LogicalSize::new(1.0, 1.0))
            .with_parent_window(parent.hwnd());

        let window = builder.build(target).ok()?;

        let instance = wgpu::Instance::default();
        let surface_target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&window) }.ok()?;
        let surface: wgpu::Surface<'static> =
            unsafe { instance.create_surface_unsafe(surface_target) }.ok()?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        ))
        .ok()?;

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|fmt| fmt.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("preview_gpu_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("preview_gpu_texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("preview_gpu_layer_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            std::mem::size_of::<LayerUniform>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("preview_gpu_shader"),
            source: wgpu::ShaderSource::Wgsl(PREVIEW_SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("preview_gpu_pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("preview_gpu_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("preview_gpu_vertex_buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        surface.configure(&device, &config);
        window.set_visible(true);

        // Make the window transparent to mouse events so clicks pass through to the webview
        // beneath. This prevents the overlay from blocking resize handles on adjacent panels.
        unsafe {
            let hwnd = window.hwnd();
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                ex_style | WS_EX_TRANSPARENT as isize | WS_EX_LAYERED as isize,
            );
        }

        Some(Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            position: dioxus::desktop::tao::dpi::PhysicalPosition::new(0, 0),
            sampler,
            texture_bind_group_layout,
            uniform_bind_group_layout,
            pipeline,
            vertex_buffer,
            layers: Vec::new(),
            canvas_size: (1, 1),
            upload_scratch: Vec::new(),
        })
    }

    fn create_layer_texture(
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

    fn create_layer(
        device: &wgpu::Device,
        sampler: &wgpu::Sampler,
        texture_layout: &wgpu::BindGroupLayout,
        uniform_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        placement: PreviewLayerPlacement,
    ) -> GpuLayer {
        let (texture, view) =
            Self::create_layer_texture(device, width, height, wgpu::TextureFormat::Rgba8UnormSrgb);
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

        let uniform = LayerUniform::new([0.0, 0.0], [0.0, 0.0], placement.opacity);
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

    fn compute_layer_uniform(
        &self,
        placement: PreviewLayerPlacement,
        canvas_size: (u32, u32),
    ) -> Option<LayerUniform> {
        let surface_w = self.size.width.max(1) as f32;
        let surface_h = self.size.height.max(1) as f32;
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

        let scale_x = rect_w / surface_w * 2.0;
        let scale_y = -(rect_h / surface_h) * 2.0;
        let translate_x = rect_x / surface_w * 2.0 - 1.0;
        let translate_y = 1.0 - rect_y / surface_h * 2.0;

        Some(LayerUniform::new(
            [scale_x, scale_y],
            [translate_x, translate_y],
            placement.opacity,
        ))
    }

    pub fn apply_bounds(&mut self, bounds: PreviewBounds) -> bool {
        // Inset the overlay bounds to prevent overlap with adjacent resize handles.
        // The resize handles are 4px wide, so we inset by that much plus a small margin.
        const INSET_LEFT: i32 = 8;
        const INSET_RIGHT: i32 = 8;
        const INSET_TOP: i32 = 4;
        const INSET_BOTTOM: i32 = 12; // Larger to clear the timeline resize handle

        let mut size = bounds.to_physical_size();
        let mut position = bounds.to_physical_position();

        // TODO: Remove once we can anchor to the WebView HWND directly.
        // This compensates for a small WebView2 client-area inset on Windows.
        position.x = position.x.saturating_add(PREVIEW_NATIVE_OFFSET_X);
        position.y = position.y.saturating_add(PREVIEW_NATIVE_OFFSET_Y);

        // Apply inset to prevent overlap with resize handles
        position.x = position.x.saturating_add(INSET_LEFT);
        position.y = position.y.saturating_add(INSET_TOP);
        let new_width = size.width.saturating_sub((INSET_LEFT + INSET_RIGHT) as u32);
        let new_height = size.height.saturating_sub((INSET_TOP + INSET_BOTTOM) as u32);
        size = dioxus::desktop::tao::dpi::PhysicalSize::new(new_width.max(1), new_height.max(1));

        if size.width == 0 || size.height == 0 {
            return false;
        }

        let mut changed = false;
        if self.position != position {
            self.window.set_outer_position(position);
            self.position = position;
            changed = true;
        }

        if self.size != size {
            self.window.set_inner_size(size);
            self.size = size;
            self.config.width = size.width.max(1);
            self.config.height = size.height.max(1);
            self.surface.configure(&self.device, &self.config);
            changed = true;
        }

        self.raise_window();
        changed
    }

    pub fn upload_layers(&mut self, stack: &PreviewLayerStack) -> bool {
        self.canvas_size = (
            stack.canvas_width.max(1),
            stack.canvas_height.max(1),
        );

        if stack.layers.is_empty() {
            self.layers.clear();
            return true;
        }

        if self.layers.len() > stack.layers.len() {
            self.layers.truncate(stack.layers.len());
        }

        let mut uploaded = false;
        for (index, layer) in stack.layers.iter().enumerate() {
            let width = layer.image.width().max(1);
            let height = layer.image.height().max(1);

            if index >= self.layers.len() {
                self.layers.push(Self::create_layer(
                    &self.device,
                    &self.sampler,
                    &self.texture_bind_group_layout,
                    &self.uniform_bind_group_layout,
                    width,
                    height,
                    layer.placement,
                ));
            } else if self.layers[index].size != (width, height) {
                self.layers[index] = Self::create_layer(
                    &self.device,
                    &self.sampler,
                    &self.texture_bind_group_layout,
                    &self.uniform_bind_group_layout,
                    width,
                    height,
                    layer.placement,
                );
            }

            if let Some(gpu_layer) = self.layers.get_mut(index) {
                gpu_layer.placement = layer.placement;
                let bytes = layer.image.as_raw();
                let expected = width as usize * height as usize * 4;
                if bytes.len() != expected {
                    continue;
                }

                let row_bytes = width * 4;
                let aligned_row_bytes =
                    align_to(row_bytes, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32);
                let data = if aligned_row_bytes == row_bytes {
                    bytes.as_slice()
                } else {
                    let rows = height as usize;
                    let padded_len = aligned_row_bytes as usize * rows;
                    self.upload_scratch.resize(padded_len, 0);
                    self.upload_scratch.fill(0);
                    let row_bytes_usize = row_bytes as usize;
                    let aligned_row_bytes_usize = aligned_row_bytes as usize;
                    for row in 0..rows {
                        let src_offset = row * row_bytes_usize;
                        let dst_offset = row * aligned_row_bytes_usize;
                        self.upload_scratch[dst_offset..dst_offset + row_bytes_usize]
                            .copy_from_slice(&bytes[src_offset..src_offset + row_bytes_usize]);
                    }
                    self.upload_scratch.as_slice()
                };

                self.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &gpu_layer.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    data,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(aligned_row_bytes),
                        rows_per_image: Some(height),
                    },
                    wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
                uploaded = true;
            }
        }

        uploaded
    }

    pub fn clear_layers(&mut self) {
        self.layers.clear();
        self.canvas_size = (1, 1);
    }

    pub fn render_layers(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(_) => return,
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("preview_gpu_clear"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("preview_gpu_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if !self.layers.is_empty() {
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

                let canvas_size = self.canvas_size;
                for layer in &self.layers {
                    let Some(uniform) =
                        self.compute_layer_uniform(layer.placement, canvas_size)
                    else {
                        continue;
                    };
                    self.queue.write_buffer(
                        &layer.uniform_buffer,
                        0,
                        bytemuck::bytes_of(&uniform),
                    );
                    pass.set_bind_group(0, &layer.bind_group, &[]);
                    pass.set_bind_group(1, &layer.uniform_bind_group, &[]);
                    pass.draw(0..QUAD_VERTICES.len() as u32, 0..1);
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn raise_window(&self) {
        unsafe {
            SetWindowPos(
                self.window.hwnd(),
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }
}

#[cfg(target_os = "windows")]
fn align_to(value: u32, alignment: u32) -> u32 {
    if alignment == 0 {
        return value;
    }
    ((value + alignment - 1) / alignment) * alignment
}

#[cfg(not(target_os = "windows"))]
pub struct PreviewGpuSurface;

#[cfg(not(target_os = "windows"))]
impl PreviewGpuSurface {
    pub fn new<T>(
        _parent: &dioxus::desktop::tao::window::Window,
        _target: &dioxus::desktop::tao::event_loop::EventLoopWindowTarget<T>,
    ) -> Option<Self> {
        None
    }

    pub fn apply_bounds(&mut self, _bounds: PreviewBounds) -> bool {
        false
    }

    pub fn upload_layers(&mut self, _stack: &PreviewLayerStack) -> bool {
        false
    }

    pub fn clear_layers(&mut self) {}

    pub fn render_layers(&mut self) {}
}

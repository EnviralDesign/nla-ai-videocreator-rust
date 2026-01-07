use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use dioxus::desktop::tao::platform::windows::WindowExtWindows;
#[cfg(target_os = "windows")]
use std::num::NonZeroU64;
#[cfg(target_os = "windows")]
use wgpu::util::DeviceExt;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
};

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
        position: [-1.0, -1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0],
        uv: [0.0, 0.0],
    },
];

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScaleUniform {
    scale: [f32; 2],
    _pad: [f32; 2],
}

#[cfg(target_os = "windows")]
impl ScaleUniform {
    fn new(scale: [f32; 2]) -> Self {
        Self {
            scale,
            _pad: [0.0, 0.0],
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

struct ScaleUniform {
    scale: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var frame_tex: texture_2d<f32>;
@group(0) @binding(1)
var frame_sampler: sampler;
@group(1) @binding(0)
var<uniform> scale_uniform: ScaleUniform;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let scaled = input.position * scale_uniform.scale;
    out.position = vec4<f32>(scaled, 0.0, 1.0);
    out.uv = input.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(frame_tex, frame_sampler, input.uv);
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
pub struct PreviewGpuSurface {
    window: dioxus::desktop::tao::window::Window,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: dioxus::desktop::tao::dpi::PhysicalSize<u32>,
    position: dioxus::desktop::tao::dpi::PhysicalPosition<i32>,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    texture_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    scale_buffer: wgpu::Buffer,
    scale_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    frame_size: (u32, u32),
    has_frame: bool,
    last_version: u64,
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

        let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let (texture, texture_view) = Self::create_frame_texture(&device, 1, 1, texture_format);
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

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("preview_gpu_texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let scale_uniform = ScaleUniform::new([1.0, 1.0]);
        let scale_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("preview_gpu_scale_buffer"),
            contents: bytemuck::bytes_of(&scale_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let scale_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("preview_gpu_scale_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            std::mem::size_of::<ScaleUniform>() as u64,
                        ),
                    },
                    count: None,
                }],
            });
        let scale_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("preview_gpu_scale_bind_group"),
            layout: &scale_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scale_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("preview_gpu_shader"),
            source: wgpu::ShaderSource::Wgsl(PREVIEW_SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("preview_gpu_pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &scale_bind_group_layout],
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

        Some(Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            position: dioxus::desktop::tao::dpi::PhysicalPosition::new(0, 0),
            texture,
            texture_view,
            sampler,
            texture_bind_group,
            texture_bind_group_layout,
            scale_buffer,
            scale_bind_group,
            pipeline,
            vertex_buffer,
            frame_size: (1, 1),
            has_frame: false,
            last_version: 0,
            upload_scratch: Vec::new(),
        })
    }

    fn create_frame_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("preview_gpu_frame_texture"),
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

    fn update_texture(&mut self, width: u32, height: u32) {
        let (texture, view) =
            Self::create_frame_texture(&self.device, width, height, wgpu::TextureFormat::Rgba8UnormSrgb);
        self.texture = texture;
        self.texture_view = view;
        self.texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("preview_gpu_texture_bind_group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
    }

    pub fn apply_bounds(&mut self, bounds: PreviewBounds) -> bool {
        let size = bounds.to_physical_size();
        let position = bounds.to_physical_position();
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

    pub fn upload_frame(&mut self, version: u64, width: u32, height: u32, bytes: &[u8]) -> bool {
        if width == 0 || height == 0 {
            return false;
        }
        let expected = width as usize * height as usize * 4;
        if bytes.len() != expected {
            return false;
        }
        if self.last_version == version && self.frame_size == (width, height) {
            return false;
        }

        if self.frame_size != (width, height) {
            self.update_texture(width, height);
            self.frame_size = (width, height);
        }

        let row_bytes = width * 4;
        let aligned_row_bytes = align_to(row_bytes, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32);
        let data = if aligned_row_bytes == row_bytes {
            bytes
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
            &self.upload_scratch
        };

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
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

        self.last_version = version;
        self.has_frame = true;
        true
    }

    pub fn clear_frame(&mut self) {
        self.has_frame = false;
        self.last_version = 0;
    }

    pub fn render(&mut self) {
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

            if self.has_frame {
                let (frame_w, frame_h) = self.frame_size;
                let surface_w = self.size.width.max(1) as f32;
                let surface_h = self.size.height.max(1) as f32;
                let scale = if frame_w > 0 && frame_h > 0 {
                    let frame_w = frame_w as f32;
                    let frame_h = frame_h as f32;
                    let scale_factor = (surface_w / frame_w).min(surface_h / frame_h);
                    let scaled_w = frame_w * scale_factor;
                    let scaled_h = frame_h * scale_factor;
                    [
                        (scaled_w / surface_w).min(1.0),
                        (scaled_h / surface_h).min(1.0),
                    ]
                } else {
                    [0.0, 0.0]
                };

                let uniform = ScaleUniform::new(scale);
                self.queue
                    .write_buffer(&self.scale_buffer, 0, bytemuck::bytes_of(&uniform));

                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.texture_bind_group, &[]);
                pass.set_bind_group(1, &self.scale_bind_group, &[]);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.draw(0..QUAD_VERTICES.len() as u32, 0..1);
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

    pub fn upload_frame(&mut self, _version: u64, _width: u32, _height: u32, _bytes: &[u8]) -> bool {
        false
    }

    pub fn clear_frame(&mut self) {}

    pub fn render(&mut self) {}
}

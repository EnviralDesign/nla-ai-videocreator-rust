#[cfg(target_os = "windows")]
use crate::core::preview::PreviewLayerStack;
#[cfg(not(target_os = "windows"))]
use crate::core::preview::PreviewLayerStack;
#[cfg(target_os = "windows")]
use super::layers::{align_to, compute_layer_uniform, create_layer};
#[cfg(target_os = "windows")]
use super::shaders::{BORDER_COLOR_LINEAR, BORDER_SHADER, PREVIEW_CLEAR_COLOR, PREVIEW_SHADER};
#[cfg(target_os = "windows")]
use super::types::{BorderUniform, GpuLayer, LayerUniform, PreviewBounds, QUAD_VERTICES, Vertex};
#[cfg(not(target_os = "windows"))]
use super::types::PreviewBounds;
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
pub struct PreviewGpuSurface {
    window: dioxus::desktop::tao::window::Window,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: dioxus::desktop::tao::dpi::PhysicalSize<u32>,
    position: dioxus::desktop::tao::dpi::PhysicalPosition<i32>,
    max_surface_size: u32,
    over_limit: bool,
    sampler: wgpu::Sampler,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    layers: Vec<GpuLayer>,
    canvas_size: (u32, u32),
    upload_scratch: Vec<u8>,
    visible: bool,
    // Border rendering (4 edges: top, bottom, left, right)
    border_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    border_bind_group_layout: wgpu::BindGroupLayout,
    border_uniform_buffers: [wgpu::Buffer; 4],
    border_bind_groups: [wgpu::BindGroup; 4],
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

        let adapter_limits = adapter.limits();
        let mut requested_limits = wgpu::Limits::downlevel_defaults();
        requested_limits.max_texture_dimension_1d = adapter_limits.max_texture_dimension_1d;
        requested_limits.max_texture_dimension_2d = adapter_limits.max_texture_dimension_2d;
        requested_limits.max_texture_dimension_3d = adapter_limits.max_texture_dimension_3d;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: requested_limits,
            },
            None,
        ))
        .or_else(|_| {
            pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            ))
        })
        .ok()?;
        let max_surface_size = device.limits().max_texture_dimension_2d.max(1);

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
            width: size.width.max(1).min(max_surface_size),
            height: size.height.max(1).min(max_surface_size),
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

        // Create border pipeline
        let border_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("border_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            std::mem::size_of::<BorderUniform>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        // Create 4 uniform buffers and bind groups (one for each border edge)
        let border_uniform = BorderUniform {
            rect: [0.0, 0.0, 0.0, 0.0],
            color: BORDER_COLOR_LINEAR,
        };
        let border_uniform_buffers: [wgpu::Buffer; 4] = std::array::from_fn(|i| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("border_uniform_buffer_{}", i)),
                contents: bytemuck::bytes_of(&border_uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        });

        let border_bind_groups: [wgpu::BindGroup; 4] = std::array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("border_bind_group_{}", i)),
                layout: &border_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: border_uniform_buffers[i].as_entire_binding(),
                }],
            })
        });

        let border_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("border_shader"),
            source: wgpu::ShaderSource::Wgsl(BORDER_SHADER.into()),
        });
        let border_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("border_pipeline_layout"),
                bind_group_layouts: &[&border_bind_group_layout],
                push_constant_ranges: &[],
            });
        let border_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("border_pipeline"),
            layout: Some(&border_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &border_shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &border_shader,
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

        surface.configure(&device, &config);
        window.set_visible(false);

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
            max_surface_size,
            over_limit: false,
            sampler,
            texture_bind_group_layout,
            uniform_bind_group_layout,
            pipeline,
            vertex_buffer,
            layers: Vec::new(),
            canvas_size: (1, 1),
            upload_scratch: Vec::new(),
            visible: false,
            border_pipeline,
            border_bind_group_layout,
            border_uniform_buffers,
            border_bind_groups,
        })
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

        if size.width > self.max_surface_size || size.height > self.max_surface_size {
            self.over_limit = true;
            if self.visible {
                self.window.set_visible(false);
                self.visible = false;
            }
            return false;
        }

        if self.over_limit {
            self.over_limit = false;
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
        if self.over_limit {
            return false;
        }
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
                self.layers.push(create_layer(
                    &self.device,
                    &self.sampler,
                    &self.texture_bind_group_layout,
                    &self.uniform_bind_group_layout,
                    width,
                    height,
                    layer.placement,
                ));
            } else if self.layers[index].size != (width, height) {
                self.layers[index] = create_layer(
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

        if uploaded && !self.visible {
            self.window.set_visible(true);
            self.visible = true;
        }

        uploaded
    }

    pub fn clear_layers(&mut self) {
        self.layers.clear();
        self.canvas_size = (1, 1);
        if self.visible {
            self.window.set_visible(false);
            self.visible = false;
        }
    }

    pub fn over_limit(&self) -> bool {
        self.over_limit
    }

    pub fn render_layers(&mut self) {
        if self.over_limit {
            return;
        }
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

        // Compute canvas bounds in screen space for border drawing
        let surface_w = self.size.width.max(1) as f32;
        let surface_h = self.size.height.max(1) as f32;
        let canvas_w = self.canvas_size.0.max(1) as f32;
        let canvas_h = self.canvas_size.1.max(1) as f32;
        let preview_scale = (surface_w / canvas_w).min(surface_h / canvas_h).max(0.0);
        let preview_w = canvas_w * preview_scale;
        let preview_h = canvas_h * preview_scale;
        let offset_x = (surface_w - preview_w) * 0.5;
        let offset_y = (surface_h - preview_h) * 0.5;

        // Pre-compute border uniforms and write to buffers BEFORE the render pass
        // (buffer writes during a render pass aren't visible until the next submission)
        let should_draw_border = preview_scale > 0.0 && !self.layers.is_empty();
        if should_draw_border {
            // 1 pixel in NDC space
            let pixel_w = 2.0 / surface_w;
            let pixel_h = 2.0 / surface_h;

            // Canvas bounds in NDC (normalized device coordinates)
            // NDC: x in [-1, 1], y in [-1, 1], with y pointing up
            let left_ndc = (offset_x / surface_w) * 2.0 - 1.0;
            let right_ndc = ((offset_x + preview_w) / surface_w) * 2.0 - 1.0;
            let top_ndc = 1.0 - (offset_y / surface_h) * 2.0;
            let bottom_ndc = 1.0 - ((offset_y + preview_h) / surface_h) * 2.0;

            // 4 border edge rects: top, bottom, left, right
            let border_rects = [
                // Top edge: from (left, top-1px) with width=canvas_width, height=1px
                [left_ndc, top_ndc - pixel_h, right_ndc - left_ndc, pixel_h],
                // Bottom edge: from (left, bottom) with width=canvas_width, height=1px
                [left_ndc, bottom_ndc, right_ndc - left_ndc, pixel_h],
                // Left edge: from (left, bottom) with width=1px, height=canvas_height
                [left_ndc, bottom_ndc, pixel_w, top_ndc - bottom_ndc],
                // Right edge: from (right-1px, bottom) with width=1px, height=canvas_height
                [right_ndc - pixel_w, bottom_ndc, pixel_w, top_ndc - bottom_ndc],
            ];

            for (i, rect) in border_rects.iter().enumerate() {
                let uniform = BorderUniform {
                    rect: *rect,
                    color: BORDER_COLOR_LINEAR,
                };
                self.queue.write_buffer(
                    &self.border_uniform_buffers[i],
                    0,
                    bytemuck::bytes_of(&uniform),
                );
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("preview_gpu_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(PREVIEW_CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if !self.layers.is_empty() {
                // Set scissor rect to clip layers to canvas bounds
                // This prevents scaled-up layers from rendering outside the project canvas
                let scissor_x = offset_x.round() as u32;
                let scissor_y = offset_y.round() as u32;
                let scissor_w = preview_w.round().max(1.0) as u32;
                let scissor_h = preview_h.round().max(1.0) as u32;
                pass.set_scissor_rect(scissor_x, scissor_y, scissor_w, scissor_h);

                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

                let canvas_size = self.canvas_size;
                for layer in &self.layers {
                    let Some(uniform) =
                        compute_layer_uniform(self.size, layer.placement, canvas_size)
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

                // Reset scissor to full surface for border drawing
                pass.set_scissor_rect(0, 0, self.size.width, self.size.height);
            }

            // Draw screen-space border (1 pixel wide) around the canvas
            if should_draw_border {
                pass.set_pipeline(&self.border_pipeline);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

                // Draw each of the 4 edges using their pre-written buffers
                for i in 0..4 {
                    pass.set_bind_group(0, &self.border_bind_groups[i], &[]);
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

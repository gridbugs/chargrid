use crate::{input, ContextDescriptor, Dimensions, FontBytes, NumPixels};
use chargrid_app::{App, ControlFlow};
use chargrid_render::ViewContext;
use grid_2d::{Coord, Grid, Size};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use zerocopy::AsBytes;

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

fn font_bytes_to_fonts(FontBytes { normal, bold }: FontBytes) -> Vec<wgpu_glyph::SharedBytes<'static>> {
    vec![
        wgpu_glyph::SharedBytes::ByArc(normal.into()),
        wgpu_glyph::SharedBytes::ByArc(bold.into()),
    ]
}

const FONT_ID_NORMAL: wgpu_glyph::FontId = wgpu_glyph::FontId(0);
const FONT_ID_BOLD: wgpu_glyph::FontId = wgpu_glyph::FontId(1);

#[derive(Debug)]
pub enum ContextBuildError {
    FailedToBuildWindow(winit::error::OsError),
    FailedToRequestGraphicsAdapter,
}

const fn dimensions_from_logical_size(size: winit::dpi::LogicalSize<f64>) -> Dimensions<f64> {
    Dimensions {
        width: size.width,
        height: size.height,
    }
}

fn finish_create_buffer_mapped_from_slice<T>(
    create_buffer_mapped: wgpu::CreateBufferMapped<'_>,
    slice: &[T],
) -> wgpu::Buffer
where
    T: AsBytes,
{
    for (t, slot) in slice
        .iter()
        .zip(create_buffer_mapped.data.chunks_exact_mut(std::mem::size_of::<T>()))
    {
        slot.copy_from_slice(t.as_bytes());
    }
    create_buffer_mapped.finish()
}

struct WgpuContext {
    device: wgpu::Device,
    sc_desc: wgpu::SwapChainDescriptor,
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,
    background_cell_instance_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    queue: wgpu::Queue,
    background_cell_instance_data: Grid<BackgroundCellInstance>,
    render_buffer: chargrid_render::Buffer,
    glyph_brush: wgpu_glyph::GlyphBrush<'static, ()>,
    global_uniforms_buffer: wgpu::Buffer,
    window_size: winit::dpi::LogicalSize<f64>,
    scale_factor: f64,
    modifier_state: winit::event::ModifiersState,
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::AsBytes, zerocopy::FromBytes)]
struct BackgroundCellInstance {
    background_colour: [f32; 3],
    foreground_colour: [f32; 3],
    underline: u32,
}

impl Default for BackgroundCellInstance {
    fn default() -> Self {
        Self {
            background_colour: [0.; 3],
            foreground_colour: [1.; 3],
            underline: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, zerocopy::AsBytes, zerocopy::FromBytes)]
struct GlobalUniforms {
    cell_size_relative_to_window: [f32; 2],
    offset_to_centre: [f32; 2],
    grid_width: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, zerocopy::AsBytes, zerocopy::FromBytes)]
struct UnderlineUniforms {
    underline_width_cell_ratio: f32,
    underline_top_offset_cell_ratio: f32,
}

async fn init_device() -> Result<(wgpu::Device, wgpu::Queue), ContextBuildError> {
    let backend = if cfg!(feature = "force_vulkan") {
        wgpu::BackendBit::VULKAN
    } else {
        wgpu::BackendBit::PRIMARY
    };
    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: None,
        },
        backend,
    )
    .await
    .ok_or(ContextBuildError::FailedToRequestGraphicsAdapter)?;
    Ok(adapter
        .request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        })
        .await)
}

impl WgpuContext {
    fn u8_slice_to_u32_vec(bytes: &[u8]) -> Vec<u32> {
        let mut buffer = Vec::with_capacity(bytes.len() / 4);
        let mut chunks = bytes.chunks_exact(4);
        for chunk in &mut chunks {
            let mut array: [u8; 4] = Default::default();
            array.copy_from_slice(chunk);
            buffer.push(u32::from_le_bytes(array));
        }
        assert!(chunks.remainder().is_empty());
        buffer
    }
    fn new(
        window: &winit::window::Window,
        size_context: &SizeContext,
        grid_size: Size,
        font_bytes: FontBytes,
    ) -> Result<Self, ContextBuildError> {
        use std::mem;
        let num_background_cell_instances = grid_size.count();
        let background_cell_instance_data = Grid::new_default(grid_size);
        let render_buffer = chargrid_render::Buffer::new(grid_size);
        let scale_factor = window.scale_factor();
        let physical_size = window.inner_size();
        let window_size: winit::dpi::LogicalSize<f64> = physical_size.to_logical(scale_factor);
        let surface = wgpu::Surface::create(window);
        let (mut device, queue) = futures_executor::block_on(init_device())?;
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: TEXTURE_FORMAT,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let vs_module = device.create_shader_module(&Self::u8_slice_to_u32_vec(include_bytes!("./shader.vert.spv")));
        let fs_module = device.create_shader_module(&Self::u8_slice_to_u32_vec(include_bytes!("./shader.frag.spv")));
        let background_cell_instance_buffer = finish_create_buffer_mapped_from_slice(
            device.create_buffer_mapped(&wgpu::BufferDescriptor {
                label: None,
                size: num_background_cell_instances as u64 * std::mem::size_of::<BackgroundCellInstance>() as u64,
                usage: wgpu::BufferUsage::VERTEX,
            }),
            background_cell_instance_data.raw(),
        );
        let global_uniforms_size = mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress;
        let global_uniforms = size_context.global_uniforms(dimensions_from_logical_size(window_size));
        let global_uniforms_buffer = finish_create_buffer_mapped_from_slice(
            device.create_buffer_mapped(&wgpu::BufferDescriptor {
                label: None,
                size: 1 * global_uniforms_size,
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }),
            &[global_uniforms],
        );
        let underline_uniforms_size = mem::size_of::<UnderlineUniforms>() as wgpu::BufferAddress;
        let underline_uniforms = UnderlineUniforms {
            underline_width_cell_ratio: size_context.underline_width as f32,
            underline_top_offset_cell_ratio: size_context.underline_top_offset as f32,
        };
        let underline_uniforms_buffer = finish_create_buffer_mapped_from_slice(
            device.create_buffer_mapped(&wgpu::BufferDescriptor {
                label: None,
                size: 1 * underline_uniforms_size,
                usage: wgpu::BufferUsage::UNIFORM,
            }),
            &[underline_uniforms],
        );
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &global_uniforms_buffer,
                        range: 0..global_uniforms_size,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &underline_uniforms_buffer,
                        range: 0..underline_uniforms_size,
                    },
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.,
                depth_bias_clamp: 0.,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: TEXTURE_FORMAT,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: mem::size_of::<BackgroundCellInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float3,
                            offset: 12,
                            shader_location: 1,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Uint,
                            offset: 24,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_fonts_bytes(font_bytes_to_fonts(font_bytes))
            .expect("Failed to load font")
            .texture_filter_method(wgpu::FilterMode::Nearest)
            .build(&mut device, TEXTURE_FORMAT);
        let modifier_state = winit::event::ModifiersState::default();
        Ok(Self {
            device,
            sc_desc,
            surface,
            swap_chain,
            render_pipeline,
            background_cell_instance_buffer,
            bind_group,
            queue,
            background_cell_instance_data,
            render_buffer,
            glyph_brush,
            global_uniforms_buffer,
            window_size,
            scale_factor,
            modifier_state,
        })
    }
    fn render_background(&mut self) {
        for (buffer_cell, background_cell_instance) in self
            .render_buffer
            .iter()
            .zip(self.background_cell_instance_data.iter_mut())
        {
            background_cell_instance.background_colour = buffer_cell.background_colour.to_f32_rgb();
            background_cell_instance.foreground_colour = buffer_cell.foreground_colour.to_f32_rgb();
            background_cell_instance.underline = buffer_cell.underline as u32;
        }
        self.background_cell_instance_buffer = finish_create_buffer_mapped_from_slice(
            self.device.create_buffer_mapped(&wgpu::BufferDescriptor {
                label: None,
                size: self.render_buffer.size().count() as u64 * std::mem::size_of::<BackgroundCellInstance>() as u64,
                usage: wgpu::BufferUsage::VERTEX,
            }),
            self.background_cell_instance_data.raw(),
        );
    }

    fn resize(&mut self, size_context: &SizeContext, window_size: winit::dpi::LogicalSize<f64>) {
        use std::mem;
        let physical_size: winit::dpi::PhysicalSize<f64> = window_size.to_physical(self.scale_factor);
        self.window_size = window_size;
        self.sc_desc.width = physical_size.width.round() as u32;
        self.sc_desc.height = physical_size.height.round() as u32;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.background_cell_instance_buffer = finish_create_buffer_mapped_from_slice(
            self.device.create_buffer_mapped(&wgpu::BufferDescriptor {
                label: None,
                size: self.render_buffer.size().count() as u64 * std::mem::size_of::<BackgroundCellInstance>() as u64,
                usage: wgpu::BufferUsage::VERTEX,
            }),
            self.background_cell_instance_data.raw(),
        );
        let global_uniforms = size_context.global_uniforms(dimensions_from_logical_size(window_size));
        let temp_buffer = finish_create_buffer_mapped_from_slice(
            self.device.create_buffer_mapped(&wgpu::BufferDescriptor {
                label: None,
                size: 1 * std::mem::size_of::<GlobalUniforms>() as u64,
                usage: wgpu::BufferUsage::COPY_SRC,
            }),
            &[global_uniforms],
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(
            &temp_buffer,
            0,
            &self.global_uniforms_buffer,
            0,
            mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress,
        );
        self.queue.submit(&[encoder.finish()]);
    }
}

struct InputContext {
    last_mouse_coord: Coord,
    last_mouse_button: Option<chargrid_input::MouseButton>,
}

impl Default for InputContext {
    fn default() -> Self {
        Self {
            last_mouse_coord: Coord::new(0, 0),
            last_mouse_button: None,
        }
    }
}

#[derive(Debug)]
struct SizeContext {
    font_source_scale: wgpu_glyph::Scale,
    font_dimensions: Dimensions<NumPixels>,
    cell_dimensions: Dimensions<NumPixels>,
    underline_width: NumPixels,
    underline_top_offset: NumPixels,
    native_window_dimensions: Dimensions<NumPixels>,
}

impl SizeContext {
    fn grid_size(&self) -> Size {
        let width = (self.native_window_dimensions.width / self.cell_dimensions.width).floor();
        let height = (self.native_window_dimensions.height / self.cell_dimensions.height).floor();
        Size::new(width as u32, height as u32)
    }
    fn native_ratio(&self, window_dimensions: Dimensions<f64>) -> f64 {
        let ratio_x = window_dimensions.width / self.native_window_dimensions.width;
        let ratio_y = window_dimensions.height / self.native_window_dimensions.height;
        ratio_x.min(ratio_y)
    }
    fn pixel_offset_to_centre_native_window(&self, window_dimensions: Dimensions<f64>) -> Dimensions<f64> {
        let native_ratio = self.native_ratio(window_dimensions);
        let scaled_native_window_dimensions = Dimensions {
            width: self.native_window_dimensions.width * native_ratio,
            height: self.native_window_dimensions.height * native_ratio,
        };
        Dimensions {
            width: (window_dimensions.width - scaled_native_window_dimensions.width) / 2.0,
            height: (window_dimensions.height - scaled_native_window_dimensions.height) / 2.0,
        }
    }
    fn scaled_cell_dimensions(&self, window_dimensions: Dimensions<f64>) -> Dimensions<f64> {
        let ratio = self.native_ratio(window_dimensions);
        Dimensions {
            width: self.cell_dimensions.width * ratio,
            height: self.cell_dimensions.height * ratio,
        }
    }
    fn global_uniforms(&self, window_dimensions: Dimensions<f64>) -> GlobalUniforms {
        let ratio_x = window_dimensions.width / self.native_window_dimensions.width;
        let ratio_y = window_dimensions.height / self.native_window_dimensions.height;
        let (scale_x, scale_y) = if ratio_x < ratio_y {
            (1.0, ratio_y / ratio_x)
        } else {
            (ratio_x / ratio_y, 1.0)
        };
        let pixel_offset_to_centre = self.pixel_offset_to_centre_native_window(window_dimensions);
        GlobalUniforms {
            cell_size_relative_to_window: [
                self.cell_dimensions.width as f32
                    / ((scale_x as f32 * self.native_window_dimensions.width as f32) / 2.),
                self.cell_dimensions.height as f32
                    / ((scale_y as f32 * self.native_window_dimensions.height as f32) / 2.),
            ],
            offset_to_centre: [
                2. * (pixel_offset_to_centre.width as f32 / window_dimensions.width as f32),
                2. * (pixel_offset_to_centre.height as f32 / window_dimensions.height as f32),
            ],
            grid_width: self.grid_size().width(),
        }
    }
}

pub struct Context {
    window: Arc<winit::window::Window>,
    event_loop: winit::event_loop::EventLoop<()>,
    wgpu_context: WgpuContext,
    size_context: SizeContext,
    input_context: InputContext,
}

pub struct WindowHandle {
    window: Arc<winit::window::Window>,
}

impl WindowHandle {
    pub fn fullscreen(&self) -> bool {
        self.window.fullscreen().is_some()
    }
    pub fn set_fullscreen(&self, fullscreen: bool) {
        let fullscreen = if fullscreen {
            let current_monitor = self.window.current_monitor();
            Some(winit::window::Fullscreen::Borderless(current_monitor))
        } else {
            None
        };
        self.window.set_fullscreen(fullscreen);
    }
}

impl Context {
    pub fn new_returning_window_handle(
        ContextDescriptor {
            font_bytes,
            title,
            window_dimensions,
            cell_dimensions,
            font_dimensions,
            font_source_dimensions,
            underline_width,
            underline_top_offset,
        }: ContextDescriptor,
    ) -> Result<(Self, WindowHandle), ContextBuildError> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window_builder = winit::window::WindowBuilder::new().with_title(title);
        let window_builder = {
            let logical_size = winit::dpi::LogicalSize::new(window_dimensions.width, window_dimensions.height);
            window_builder
                .with_inner_size(logical_size)
                .with_min_inner_size(logical_size)
                .with_max_inner_size(logical_size)
        };
        let window = window_builder
            .build(&event_loop)
            .map_err(ContextBuildError::FailedToBuildWindow)?;
        let size_context = SizeContext {
            font_source_scale: wgpu_glyph::Scale {
                x: font_source_dimensions.width,
                y: font_source_dimensions.height,
            },
            font_dimensions,
            cell_dimensions,
            underline_width,
            underline_top_offset,
            native_window_dimensions: window_dimensions,
        };
        let grid_size = size_context.grid_size();
        let wgpu_context = WgpuContext::new(&window, &size_context, grid_size, font_bytes)?;
        log::info!("grid size: {:?}", grid_size);
        let window = Arc::new(window);
        Ok((
            Context {
                window: window.clone(),
                event_loop,
                wgpu_context,
                size_context,
                input_context: Default::default(),
            },
            WindowHandle { window: window.clone() },
        ))
    }
    pub fn new(context_descriptor: ContextDescriptor) -> Result<Self, ContextBuildError> {
        Self::new_returning_window_handle(context_descriptor).map(|(s, _)| s)
    }
    pub fn run_app<A>(self, mut app: A) -> !
    where
        A: App + 'static,
    {
        let Self {
            window,
            event_loop,
            mut wgpu_context,
            size_context,
            mut input_context,
        } = self;
        #[allow(unused_variables)]
        let window = window; // dropping the window will close it, so keep this in scope
        let mut frame_instant = Instant::now();
        let mut exited = false;
        log::info!("Entering main event loop");
        let mut current_window_dimensions = size_context.native_window_dimensions;
        event_loop.run(move |event, _, control_flow| {
            if exited {
                *control_flow = winit::event_loop::ControlFlow::Exit;
                return;
            } else {
                *control_flow = winit::event_loop::ControlFlow::Poll;
            };
            match event {
                winit::event::Event::WindowEvent {
                    event: window_event, ..
                } => match window_event {
                    winit::event::WindowEvent::ModifiersChanged(modifier_state) => {
                        wgpu_context.modifier_state = modifier_state;
                    }
                    other => {
                        if let Some(event) = input::convert_event(
                            other,
                            size_context.scaled_cell_dimensions(current_window_dimensions),
                            size_context.pixel_offset_to_centre_native_window(current_window_dimensions),
                            &mut input_context.last_mouse_coord,
                            &mut input_context.last_mouse_button,
                            wgpu_context.scale_factor,
                            wgpu_context.modifier_state,
                        ) {
                            match event {
                                input::Event::Input(input) => {
                                    if let Some(ControlFlow::Exit) = app.on_input(input) {
                                        exited = true;
                                        return;
                                    }
                                }
                                input::Event::Resize(size) => {
                                    wgpu_context.resize(&size_context, size);
                                    current_window_dimensions = dimensions_from_logical_size(size);
                                }
                            }
                        }
                    }
                },
                winit::event::Event::MainEventsCleared => {
                    let frame_duration = frame_instant.elapsed();
                    frame_instant = Instant::now();
                    let view_context = ViewContext::default_with_size(wgpu_context.render_buffer.size());
                    wgpu_context.render_buffer.clear();
                    if let Some(ControlFlow::Exit) =
                        app.on_frame(frame_duration, view_context, &mut wgpu_context.render_buffer)
                    {
                        exited = true;
                        return;
                    }
                    wgpu_context.render_background();
                    if let Ok(frame) = wgpu_context.swap_chain.get_next_texture() {
                        let mut encoder = wgpu_context
                            .device
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                        {
                            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                                    attachment: &frame.view,
                                    resolve_target: None,
                                    load_op: wgpu::LoadOp::Clear,
                                    store_op: wgpu::StoreOp::Store,
                                    clear_color: wgpu::Color::BLACK,
                                }],
                                depth_stencil_attachment: None,
                            });
                            render_pass.set_pipeline(&wgpu_context.render_pipeline);
                            render_pass.set_bind_group(0, &wgpu_context.bind_group, &[]);
                            render_pass.set_vertex_buffer(0, &wgpu_context.background_cell_instance_buffer, 0, 0);
                            render_pass.draw(0..6, 0..wgpu_context.render_buffer.size().count() as u32);
                        }
                        let mut buf: [u8; 4] = [0; 4];
                        let offset_to_centre =
                            size_context.pixel_offset_to_centre_native_window(current_window_dimensions);
                        let font_ratio = size_context.native_ratio(current_window_dimensions);
                        let font_scale = wgpu_glyph::Scale {
                            x: font_ratio as f32 * size_context.font_dimensions.width as f32,
                            y: font_ratio as f32 * size_context.font_dimensions.height as f32,
                        };
                        for (coord, cell) in wgpu_context.render_buffer.enumerate() {
                            if cell.character == ' ' && !cell.underline {
                                continue;
                            }
                            let text: &str = cell.character.encode_utf8(&mut buf);
                            let screen_position = (
                                offset_to_centre.width as f32
                                    + font_ratio as f32 * coord.x as f32 * size_context.cell_dimensions.width as f32,
                                offset_to_centre.height as f32
                                    + font_ratio as f32 * coord.y as f32 * size_context.cell_dimensions.height as f32,
                            );
                            let font_id = if cell.bold { FONT_ID_BOLD } else { FONT_ID_NORMAL };
                            let color = cell.foreground_colour.to_f32_rgba(1.);
                            let section = wgpu_glyph::Section {
                                text,
                                screen_position,
                                font_id,
                                color,
                                scale: font_scale,
                                ..Default::default()
                            };
                            wgpu_context.glyph_brush.queue(section);
                        }
                        wgpu_context
                            .glyph_brush
                            .draw_queued(
                                &mut wgpu_context.device,
                                &mut encoder,
                                &frame.view,
                                wgpu_context.window_size.width as u32,
                                wgpu_context.window_size.height as u32,
                            )
                            .unwrap();
                        wgpu_context.queue.submit(&[encoder.finish()]);
                    } else {
                        log::warn!("timeout when acquiring next swapchain texture");
                        thread::sleep(Duration::from_millis(100));
                    }
                }
                _ => (),
            }
        })
    }
}

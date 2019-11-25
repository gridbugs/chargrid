use crate::input;
use grid_2d::{Coord, Grid, Size};
use prototty_app::{App, ControlFlow};
use prototty_render::{Blend, Frame, Rgb24, ViewCell, ViewContext};
use std::thread;
use std::time::{Duration, Instant};

pub struct FontBytes {
    pub normal: Vec<u8>,
    pub bold: Vec<u8>,
}

#[derive(Clone, Copy)]
pub struct Dimensions<T> {
    pub width: T,
    pub height: T,
}

pub type NumPixels = f64;

impl Dimensions<NumPixels> {
    fn to_logical_size(self) -> winit::dpi::LogicalSize {
        winit::dpi::LogicalSize::new(self.width, self.height)
    }
    fn to_f32(self) -> Dimensions<f32> {
        Dimensions {
            width: self.width as f32,
            height: self.height as f32,
        }
    }
}

pub enum WindowDimensions {
    Windowed(Dimensions<NumPixels>),
    Fullscreen,
}

pub mod defaults {
    use super::{Dimensions, NumPixels, WindowDimensions};
    pub const CELL_DIMENSIONS: Dimensions<NumPixels> = Dimensions {
        width: 16.,
        height: 16.,
    };
    pub const FONT_DIMENSIONS: Dimensions<NumPixels> = Dimensions {
        width: 16.,
        height: 16.,
    };
    pub const WINDOW_DIMENSINOS: WindowDimensions = WindowDimensions::Windowed(Dimensions {
        width: 640.,
        height: 480.,
    });
    pub const UNDERLINE_WIDTH: NumPixels = 2.;
    pub const UNDERLINE_BOTTOM_OFFSET: NumPixels = 2.;
}

pub struct ContextBuilder {
    font_bytes: FontBytes,
    window_builder: winit::window::WindowBuilder,
    cell_dimensions: Dimensions<NumPixels>,
    font_dimensions: Dimensions<NumPixels>,
    underline_width: NumPixels,
    underline_bottom_offset: NumPixels,
}

#[derive(Debug)]
pub enum ContextBuildError {
    FailedToBuildWindow(winit::error::OsError),
    FailedToRequestGraphicsAdapter,
}

impl ContextBuilder {
    pub fn new_with_font_bytes(font_bytes: FontBytes) -> Self {
        Self {
            font_bytes,
            window_builder: winit::window::WindowBuilder::new(),
            cell_dimensions: defaults::CELL_DIMENSIONS,
            font_dimensions: defaults::FONT_DIMENSIONS,
            underline_width: defaults::UNDERLINE_WIDTH,
            underline_bottom_offset: defaults::UNDERLINE_BOTTOM_OFFSET,
        }
    }
    pub fn with_title(self, title: &str) -> Self {
        Self {
            window_builder: self.window_builder.with_title(title),
            ..self
        }
    }
    pub fn with_cell_dimensions(self, cell_dimensions: Dimensions<NumPixels>) -> Self {
        Self {
            cell_dimensions,
            ..self
        }
    }
    pub fn with_font_dimensions(self, font_dimensions: Dimensions<NumPixels>) -> Self {
        Self {
            font_dimensions,
            ..self
        }
    }
    pub fn with_window_dimensions(self, window_dimensions: WindowDimensions) -> Self {
        let window_builder = match window_dimensions {
            WindowDimensions::Fullscreen => self.window_builder.with_fullscreen(None),
            WindowDimensions::Windowed(dimensions) => {
                let logical_size = dimensions.to_logical_size();
                self.window_builder
                    .with_inner_size(logical_size)
                    .with_min_inner_size(logical_size)
                    .with_max_inner_size(logical_size)
            }
        };
        Self { window_builder, ..self }
    }
    pub fn with_underline_width(self, underline_width: NumPixels) -> Self {
        Self {
            underline_width,
            ..self
        }
    }
    pub fn with_underline_bottom_offset(self, underline_bottom_offset: NumPixels) -> Self {
        Self {
            underline_bottom_offset,
            ..self
        }
    }
    pub fn build(self) -> Result<Context, ContextBuildError> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = self
            .window_builder
            .build(&event_loop)
            .map_err(ContextBuildError::FailedToBuildWindow)?;
        let grid_size = Size::new(24, 16);
        let wgpu_context = WgpuContext::new(&window, grid_size)?;
        Ok(Context {
            event_loop,
            window,
            wgpu_context,
            size_context: SizeContext {
                cell_dimensions: self.cell_dimensions,
                grid_size,
            },
            input_context: Default::default(),
        })
    }
}

struct WgpuContext {
    device: wgpu::Device,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,
    global_uniform_buffer: wgpu::Buffer,
    background_cell_instance_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    queue: wgpu::Queue,
    num_background_cell_instances: u32,
    background_cell_instance_data: Grid<BackgroundCellInstance>,
    render_buffer: prototty_render::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::AsBytes, zerocopy::FromBytes)]
struct BackgroundCellInstance {
    background_colour: [f32; 3],
    foreground_colour: [f32; 3],
}

impl Default for BackgroundCellInstance {
    fn default() -> Self {
        Self {
            background_colour: [0.; 3],
            foreground_colour: [1.; 3],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, zerocopy::AsBytes, zerocopy::FromBytes)]
struct GlobalUniforms {
    cell_size_relative_to_window: [f32; 2],
    grid_width: u32,
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
    fn copy_global_uniforms(&mut self) {
        let global_uniforms = GlobalUniforms {
            grid_width: 16,
            cell_size_relative_to_window: [0.1, 0.1],
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        let temp_buf = self
            .device
            .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&[global_uniforms]);
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.global_uniform_buffer, 0, 12);
        encoder.finish();
    }
    fn new(window: &winit::window::Window, grid_size: Size) -> Result<Self, ContextBuildError> {
        use std::iter;
        use std::mem;
        let num_background_cell_instances = grid_size.count() as u32;
        let background_cell_instance_data = Grid::new_default(grid_size);
        let render_buffer = prototty_render::Buffer::new(grid_size);
        let logical_size = window.inner_size();
        let physical_size = logical_size.to_physical(window.hidpi_factor());
        let surface = wgpu::Surface::create(window);
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
            },
            wgpu::BackendBit::PRIMARY,
        )
        .ok_or(ContextBuildError::FailedToRequestGraphicsAdapter)?;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: physical_size.width.round() as u32,
            height: physical_size.height.round() as u32,
            present_mode: wgpu::PresentMode::Vsync,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let vs_module = device.create_shader_module(&Self::u8_slice_to_u32_vec(include_bytes!("./shader.vert.spv")));
        let fs_module = device.create_shader_module(&Self::u8_slice_to_u32_vec(include_bytes!("./shader.frag.spv")));
        let background_cell_instance_buffer = device
            .create_buffer_mapped(num_background_cell_instances as usize, wgpu::BufferUsage::VERTEX)
            .fill_from_slice(background_cell_instance_data.raw());
        let global_uniforms_size = mem::size_of::<GlobalUniforms>() as u64;
        let global_uniforms = GlobalUniforms {
            grid_width: grid_size.width(),
            cell_size_relative_to_window: [
                32. / (logical_size.width as f32 / 2.),
                48. / (logical_size.height as f32 / 2.),
            ],
        };
        log::info!("global uniforms: {:?}", global_uniforms);
        assert!(global_uniforms_size == 12);
        /*
        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: global_uniforms_size,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        }); */
        /*
        {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
            let temp_buf = device
                .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
                .fill_from_slice(&[global_uniforms]);
            encoder.copy_buffer_to_buffer(&temp_buf, 0, &global_uniform_buffer, 0, global_uniforms_size);
            encoder.finish();
        }*/

        let global_uniform_buffer = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM)
            .fill_from_slice(&[global_uniforms]);
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[wgpu::BindGroupLayoutBinding {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &global_uniform_buffer,
                    range: 0..global_uniforms_size,
                },
            }],
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
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
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
                ],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        Ok(Self {
            device,
            swap_chain,
            render_pipeline,
            global_uniform_buffer,
            background_cell_instance_buffer,
            bind_group,
            queue,
            num_background_cell_instances,
            background_cell_instance_data,
            render_buffer,
        })
    }
    fn render_internal(&mut self) {
        //self.copy_global_uniforms();
        for (buffer_cell, background_cell_instance) in self
            .render_buffer
            .iter()
            .zip(self.background_cell_instance_data.iter_mut())
        {
            background_cell_instance.background_colour = buffer_cell.background_colour.to_f32_rgb();
            background_cell_instance.foreground_colour = buffer_cell.foreground_colour.to_f32_rgb();
        }
        self.background_cell_instance_buffer = self
            .device
            .create_buffer_mapped(self.num_background_cell_instances as usize, wgpu::BufferUsage::VERTEX)
            .fill_from_slice(self.background_cell_instance_data.raw());
    }
}

struct InputContext {
    closing: bool,
    last_mouse_coord: Coord,
    last_mouse_button: Option<prototty_input::MouseButton>,
}

impl Default for InputContext {
    fn default() -> Self {
        Self {
            closing: false,
            last_mouse_coord: Coord::new(0, 0),
            last_mouse_button: None,
        }
    }
}

struct SizeContext {
    cell_dimensions: Dimensions<NumPixels>,
    grid_size: Size,
}

pub struct Context {
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
    wgpu_context: WgpuContext,
    size_context: SizeContext,
    input_context: InputContext,
}

impl Context {
    pub fn run_app<A>(self, mut app: A) -> !
    where
        A: App + 'static,
    {
        let Self {
            event_loop,
            window: _,
            mut wgpu_context,
            size_context,
            mut input_context,
        } = self;
        let mut frame_instant = Instant::now();
        event_loop.run(move |event, _, control_flow| match event {
            winit::event::Event::WindowEvent {
                event: window_event, ..
            } => {
                if let Some(event) = input::convert_event(
                    window_event,
                    size_context.cell_dimensions,
                    &mut input_context.last_mouse_coord,
                    &mut input_context.last_mouse_button,
                ) {
                    match event {
                        input::Event::Input(input) => {
                            if let Some(ControlFlow::Exit) = app.on_input(input) {
                                println!("exit");
                                *control_flow = winit::event_loop::ControlFlow::Exit;
                            }
                        }
                        input::Event::Resize(size) => println!("{:?}", size),
                    }
                }
            }
            winit::event::Event::EventsCleared => {
                let frame_duration = frame_instant.elapsed();
                frame_instant = Instant::now();
                let view_context = ViewContext::default_with_size(size_context.grid_size);
                wgpu_context.render_buffer.clear();
                if let Some(ControlFlow::Exit) =
                    app.on_frame(frame_duration, view_context, &mut wgpu_context.render_buffer)
                {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                wgpu_context.render_internal();
                if let Ok(frame) = wgpu_context.swap_chain.get_next_texture() {
                    let mut encoder = wgpu_context
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: &frame.view,
                                resolve_target: None,
                                load_op: wgpu::LoadOp::Clear,
                                store_op: wgpu::StoreOp::Store,
                                clear_color: wgpu::Color::GREEN,
                            }],
                            depth_stencil_attachment: None,
                        });
                        render_pass.set_pipeline(&wgpu_context.render_pipeline);
                        render_pass.set_bind_group(0, &wgpu_context.bind_group, &[]);
                        render_pass.set_vertex_buffers(0, &[(&wgpu_context.background_cell_instance_buffer, 0)]);
                        render_pass.draw(0..6, 0..wgpu_context.num_background_cell_instances);
                    }
                    wgpu_context.queue.submit(&[encoder.finish()]);
                } else {
                    log::warn!("timeout when acquiring next swapchain texture");
                    thread::sleep(Duration::from_millis(100));
                }
            }
            _ => (),
        })
    }
}

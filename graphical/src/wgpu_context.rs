use crate::{input, ContextDescriptor, Dimensions, FontBytes, NumPixels};
use chargrid_app::{App, ControlFlow};
#[cfg(feature = "gamepad")]
use chargrid_gamepad::GamepadContext;
use chargrid_render::ViewContext;
use grid_2d::{Coord, Grid, Size};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use wgpu_glyph::ab_glyph;
use zerocopy::AsBytes;

fn rgb_to_srgb_channel(c: f32) -> f32 {
    c.powf(2.2)
}

fn rgb_to_srgb([r, g, b]: [f32; 3]) -> [f32; 3] {
    [
        rgb_to_srgb_channel(r),
        rgb_to_srgb_channel(g),
        rgb_to_srgb_channel(b),
    ]
}

fn rgba_to_srgb([r, g, b, a]: [f32; 4]) -> [f32; 4] {
    [
        rgb_to_srgb_channel(r),
        rgb_to_srgb_channel(g),
        rgb_to_srgb_channel(b),
        a,
    ]
}

fn font_bytes_to_fonts(FontBytes { normal, bold }: FontBytes) -> Vec<ab_glyph::FontVec> {
    vec![
        ab_glyph::FontVec::try_from_vec(normal).unwrap(),
        ab_glyph::FontVec::try_from_vec(bold).unwrap(),
    ]
}

const FONT_ID_NORMAL: wgpu_glyph::FontId = wgpu_glyph::FontId(0);
const FONT_ID_BOLD: wgpu_glyph::FontId = wgpu_glyph::FontId(1);

#[derive(Debug)]
pub enum ContextBuildError {
    FailedToBuildWindow(winit::error::OsError),
    FailedToRequestGraphicsAdapter,
    FailedToRequestDevice(wgpu::RequestDeviceError),
}

const fn dimensions_from_logical_size(size: winit::dpi::LogicalSize<f64>) -> Dimensions<f64> {
    Dimensions {
        width: size.width,
        height: size.height,
    }
}

fn populate_and_finish_buffer<T>(buffer: wgpu::Buffer, slice: &[T]) -> wgpu::Buffer
where
    T: AsBytes,
{
    {
        use std::ops::DerefMut;
        let mut buffer_slice_view = buffer.slice(..).get_mapped_range_mut();
        let buffer_slice_view_slice = buffer_slice_view.deref_mut();
        for (t, slot) in slice
            .iter()
            .zip(buffer_slice_view_slice.chunks_exact_mut(std::mem::size_of::<T>()))
        {
            slot.copy_from_slice(t.as_bytes());
        }
    }
    buffer.unmap();
    buffer
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
    glyph_brush: wgpu_glyph::GlyphBrush<(), ab_glyph::FontVec>,
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
    underline_width_cell_ratio: f32,
    underline_top_offset_cell_ratio: f32,
}

struct Setup {
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

async fn request_adapter_for_backend(
    backend: wgpu::BackendBit,
    window: &winit::window::Window,
) -> Result<
    (
        wgpu::Adapter,
        wgpu::Instance,
        wgpu::Surface,
        wgpu::Device,
        wgpu::Queue,
    ),
    String,
> {
    let power_preference = wgpu::PowerPreference::default();
    let instance = wgpu::Instance::new(backend);
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            compatible_surface: Some(&surface),
        })
        .await
        .ok_or_else(|| "No suitable GPU adapters found on the system!".to_string())?;
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .map_err(|e| format!("Unable to find a suitable GPU adapter! ({:?})", e))?;
    Ok((adapter, instance, surface, device, queue))
}

async fn setup(title: &str, window_dimensions: Dimensions<NumPixels>, resizable: bool) -> Setup {
    let event_loop = winit::event_loop::EventLoop::new();
    let window_builder = winit::window::WindowBuilder::new().with_title(title);
    let window_builder = {
        let logical_size =
            winit::dpi::LogicalSize::new(window_dimensions.width, window_dimensions.height);
        window_builder
            .with_inner_size(logical_size)
            .with_min_inner_size(logical_size)
            .with_max_inner_size(logical_size)
            .with_resizable(resizable)
    };
    let window = window_builder.build(&event_loop).unwrap();
    let (adapter, instance, surface, device, queue) = match request_adapter_for_backend(
        wgpu::BackendBit::PRIMARY,
        &window,
    )
    .await
    {
        Ok(x) => x,
        Err(message) => {
            log::error!(
                "Failed to initialize primary backend: {}\nFalling back to secondary backend....",
                message
            );
            request_adapter_for_backend(wgpu::BackendBit::SECONDARY, &window)
                .await
                .expect("Failed to initialize secondary backend!")
        }
    };
    Setup {
        window,
        event_loop,
        instance,
        surface,
        adapter,
        device,
        queue,
    }
}

impl WgpuContext {
    fn spirv_slice_to_shader_module_source(spirv_slice: &[u8]) -> wgpu::ShaderSource<'_> {
        use std::borrow::Cow;
        assert!(spirv_slice.len() % 4 == 0);
        let mut buffer = Vec::with_capacity(spirv_slice.len() / 4);
        let mut chunks = spirv_slice.chunks_exact(4);
        for chunk in &mut chunks {
            let mut array: [u8; 4] = Default::default();
            array.copy_from_slice(chunk);
            buffer.push(u32::from_le_bytes(array));
        }
        assert!(chunks.remainder().is_empty());
        wgpu::ShaderSource::SpirV(Cow::Owned(buffer))
    }
    fn new(
        window: &winit::window::Window,
        mut device: wgpu::Device,
        queue: wgpu::Queue,
        adapter: &wgpu::Adapter,
        surface: wgpu::Surface,
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
        let swapchain_format = adapter
            .get_swap_chain_preferred_format(&surface)
            .expect("Failed to find compatible texture format");
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: Self::spirv_slice_to_shader_module_source(include_bytes!("./shader.vert.spv")),
            flags: wgpu::ShaderFlags::VALIDATION,
        });
        let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: Self::spirv_slice_to_shader_module_source(include_bytes!("./shader.frag.spv")),
            flags: wgpu::ShaderFlags::VALIDATION,
        });
        let background_cell_instance_buffer = populate_and_finish_buffer(
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: num_background_cell_instances as u64
                    * std::mem::size_of::<BackgroundCellInstance>() as u64,
                usage: wgpu::BufferUsage::VERTEX,
                mapped_at_creation: true,
            }),
            background_cell_instance_data.raw(),
        );
        let global_uniforms_size = mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress;
        let global_uniforms =
            size_context.global_uniforms(dimensions_from_logical_size(window_size));
        let global_uniforms_buffer = populate_and_finish_buffer(
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 1 * global_uniforms_size,
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                mapped_at_creation: true,
            }),
            &[global_uniforms],
        );
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &global_uniforms_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<BackgroundCellInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 12,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Sint32,
                            offset: 24,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[swapchain_format.into()],
            }),
        });
        let glyph_brush =
            wgpu_glyph::GlyphBrushBuilder::using_fonts(font_bytes_to_fonts(font_bytes))
                .texture_filter_method(wgpu::FilterMode::Nearest)
                .build(&mut device, swapchain_format);
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
            background_cell_instance.background_colour =
                rgb_to_srgb(buffer_cell.background_colour.to_f32_rgb());
            background_cell_instance.foreground_colour =
                rgb_to_srgb(buffer_cell.foreground_colour.to_f32_rgb());
            background_cell_instance.underline = buffer_cell.underline as u32;
        }
        self.background_cell_instance_buffer = populate_and_finish_buffer(
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.render_buffer.size().count() as u64
                    * std::mem::size_of::<BackgroundCellInstance>() as u64,
                usage: wgpu::BufferUsage::VERTEX,
                mapped_at_creation: true,
            }),
            self.background_cell_instance_data.raw(),
        );
    }

    fn resize(&mut self, size_context: &SizeContext, physical_size: winit::dpi::PhysicalSize<u32>) {
        use std::mem;
        let logical_size = physical_size.to_logical(self.scale_factor);
        self.window_size = logical_size;
        self.sc_desc.width = physical_size.width;
        self.sc_desc.height = physical_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.background_cell_instance_buffer = populate_and_finish_buffer(
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.render_buffer.size().count() as u64
                    * std::mem::size_of::<BackgroundCellInstance>() as u64,
                usage: wgpu::BufferUsage::VERTEX,
                mapped_at_creation: true,
            }),
            self.background_cell_instance_data.raw(),
        );
        let global_uniforms =
            size_context.global_uniforms(dimensions_from_logical_size(logical_size));
        let temp_buffer = populate_and_finish_buffer(
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 1 * std::mem::size_of::<GlobalUniforms>() as u64,
                usage: wgpu::BufferUsage::COPY_SRC,
                mapped_at_creation: true,
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
        self.queue.submit(Some(encoder.finish()));
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
    font_source_scale: ab_glyph::PxScale,
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
    fn pixel_offset_to_centre_native_window(
        &self,
        window_dimensions: Dimensions<f64>,
    ) -> Dimensions<f64> {
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
            underline_width_cell_ratio: self.underline_width as f32,
            underline_top_offset_cell_ratio: self.underline_top_offset as f32,
        }
    }
}

pub struct Context {
    window: Arc<winit::window::Window>,
    event_loop: winit::event_loop::EventLoop<()>,
    wgpu_context: WgpuContext,
    size_context: SizeContext,
    input_context: InputContext,
    text_buffer: String,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    #[cfg(feature = "gamepad")]
    gamepad: GamepadContext,
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
    pub fn new(
        ContextDescriptor {
            font_bytes,
            title,
            window_dimensions,
            cell_dimensions,
            font_dimensions,
            font_source_dimensions,
            underline_width,
            underline_top_offset,
            resizable,
        }: ContextDescriptor,
    ) -> Result<Self, ContextBuildError> {
        let Setup {
            window,
            event_loop,
            instance,
            surface,
            adapter,
            device,
            queue,
        } = pollster::block_on(setup(title.as_str(), window_dimensions, resizable));
        let size_context = SizeContext {
            font_source_scale: ab_glyph::PxScale {
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
        let wgpu_context = WgpuContext::new(
            &window,
            device,
            queue,
            &adapter,
            surface,
            &size_context,
            grid_size,
            font_bytes,
        )?;
        log::info!("grid size: {:?}", grid_size);
        let window = Arc::new(window);
        Ok(Context {
            window,
            event_loop,
            wgpu_context,
            size_context,
            input_context: Default::default(),
            text_buffer: String::new(),
            instance,
            adapter,
            #[cfg(feature = "gamepad")]
            gamepad: GamepadContext::new(),
        })
    }
    pub fn window_handle(&self) -> WindowHandle {
        WindowHandle {
            window: self.window.clone(),
        }
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
            mut text_buffer,
            instance,
            adapter,
            #[cfg(feature = "gamepad")]
            mut gamepad,
        } = self;
        let mut frame_instant = Instant::now();
        let mut last_update_inst = Instant::now();
        let mut exited = false;
        log::info!("Entering main event loop");
        let mut current_window_dimensions = size_context.native_window_dimensions;
        let mut staging_belt = wgpu::util::StagingBelt::new(1024);
        let executor = async_executor::LocalExecutor::new();
        event_loop.run(move |event, _, control_flow| {
            let _ = (&instance, &adapter); // force ownership by the closure
            if exited {
                *control_flow = winit::event_loop::ControlFlow::Exit;
                return;
            } else {
                *control_flow = winit::event_loop::ControlFlow::Poll;
            };
            #[cfg(feature = "gamepad")]
            for input in gamepad.drain_input() {
                if let Some(ControlFlow::Exit) = app.on_input(chargrid_input::Input::Gamepad(input))
                {
                    exited = true;
                    return;
                }
            }
            match event {
                winit::event::Event::WindowEvent {
                    event: window_event,
                    ..
                } => match window_event {
                    winit::event::WindowEvent::ModifiersChanged(modifier_state) => {
                        wgpu_context.modifier_state = modifier_state;
                    }
                    other => {
                        if let Some(event) = input::convert_event(
                            other,
                            size_context.scaled_cell_dimensions(current_window_dimensions),
                            size_context
                                .pixel_offset_to_centre_native_window(current_window_dimensions),
                            &mut input_context.last_mouse_coord,
                            &mut input_context.last_mouse_button,
                            &mut wgpu_context.scale_factor,
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
                                    current_window_dimensions =
                                        dimensions_from_logical_size(wgpu_context.window_size);
                                }
                            }
                        }
                    }
                },
                winit::event::Event::RedrawRequested(_) => {
                    let frame_duration = frame_instant.elapsed();
                    frame_instant = Instant::now();
                    let view_context =
                        ViewContext::default_with_size(wgpu_context.render_buffer.size());
                    wgpu_context.render_buffer.clear();
                    if let Some(ControlFlow::Exit) = app.on_frame(
                        frame_duration,
                        view_context,
                        &mut wgpu_context.render_buffer,
                    ) {
                        exited = true;
                        return;
                    }
                    wgpu_context.render_background();
                    if let Ok(frame) = wgpu_context.swap_chain.get_current_frame() {
                        let mut encoder = wgpu_context.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor { label: None },
                        );
                        {
                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: None,
                                    color_attachments: &[wgpu::RenderPassColorAttachment {
                                        view: &frame.output.view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                            store: true,
                                        },
                                    }],
                                    depth_stencil_attachment: None,
                                });
                            render_pass.set_pipeline(&wgpu_context.render_pipeline);
                            render_pass.set_bind_group(0, &wgpu_context.bind_group, &[]);
                            render_pass.set_vertex_buffer(
                                0,
                                wgpu_context.background_cell_instance_buffer.slice(..),
                            );
                            render_pass
                                .draw(0..6, 0..wgpu_context.render_buffer.size().count() as u32);
                        }
                        let offset_to_centre = size_context
                            .pixel_offset_to_centre_native_window(current_window_dimensions);
                        let font_ratio = size_context.native_ratio(current_window_dimensions);
                        let font_scale = ab_glyph::PxScale {
                            x: font_ratio as f32 * size_context.font_dimensions.width as f32,
                            y: font_ratio as f32 * size_context.font_dimensions.height as f32,
                        };
                        text_buffer.clear();
                        for row in wgpu_context.render_buffer.rows() {
                            for cell in row {
                                text_buffer.push(cell.character);
                            }
                        }
                        let mut section = wgpu_glyph::Section::default().with_screen_position((
                            offset_to_centre.width as f32,
                            offset_to_centre.height as f32,
                        ));
                        let mut char_start = 0;
                        for (ch, (coord, cell)) in text_buffer
                            .chars()
                            .zip(wgpu_context.render_buffer.enumerate())
                        {
                            let char_end = char_start + ch.len_utf8();
                            let str_slice = &text_buffer[char_start..char_end];
                            let font_id = if cell.bold {
                                FONT_ID_BOLD
                            } else {
                                FONT_ID_NORMAL
                            };
                            section = section.add_text(
                                wgpu_glyph::Text::new(str_slice)
                                    .with_scale(font_scale)
                                    .with_font_id(font_id)
                                    .with_color(rgba_to_srgb(
                                        cell.foreground_colour.to_f32_rgba(1.),
                                    )),
                            );
                            char_start = char_end;
                            if coord.x as u32 == wgpu_context.render_buffer.size().width() - 1 {
                                section = section
                                    .add_text(wgpu_glyph::Text::new("\n").with_scale(font_scale));
                            }
                        }
                        wgpu_context.glyph_brush.queue(section);
                        wgpu_context
                            .glyph_brush
                            .draw_queued(
                                &wgpu_context.device,
                                &mut staging_belt,
                                &mut encoder,
                                &frame.output.view,
                                wgpu_context.window_size.width as u32,
                                wgpu_context.window_size.height as u32,
                            )
                            .unwrap();
                        staging_belt.finish();
                        wgpu_context.queue.submit(std::iter::once(encoder.finish()));
                        executor.spawn(staging_belt.recall()).detach();
                    } else {
                        log::warn!("timeout when acquiring next swapchain texture");
                        thread::sleep(Duration::from_millis(100));
                    }
                }
                winit::event::Event::RedrawEventsCleared => {
                    let target_frametime = Duration::from_secs_f64(1.0 / 60.0);
                    let time_since_last_frame = last_update_inst.elapsed();
                    if time_since_last_frame >= target_frametime {
                        window.request_redraw();
                        last_update_inst = Instant::now();
                    } else {
                        *control_flow = winit::event_loop::ControlFlow::WaitUntil(
                            Instant::now() + target_frametime - time_since_last_frame,
                        );
                    }

                    while executor.try_tick() {}
                }
                _ => (),
            }
        })
    }
}

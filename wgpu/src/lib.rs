use crate::text_renderer::{TextRenderer, TextRendererArgs};
use anyhow::anyhow;
#[cfg(feature = "gamepad")]
use chargrid_gamepad::GamepadContext;
use chargrid_runtime::{Component, FrameBuffer, app, on_frame, on_input};
use grid_2d::{Grid, ICoord, UCoord};
use std::{
    borrow::Cow,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::application::ApplicationHandler;

mod input;
mod text_renderer;

#[derive(Clone, Debug)]
pub struct FontBytes {
    pub normal: Arc<Vec<u8>>,
    pub bold: Arc<Vec<u8>>,
}

impl FontBytes {
    pub fn new(normal: impl Into<Vec<u8>>, bold: impl Into<Vec<u8>>) -> Self {
        Self {
            normal: Arc::new(normal.into()),
            bold: Arc::new(bold.into()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Dimensions<T> {
    pub width: T,
    pub height: T,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub title: String,
    pub dimensions_px: Dimensions<f64>,
    pub resizable: bool,
    pub font_bytes: FontBytes,
    pub font_size_px: f32,
    pub cell_dimensions_px: Dimensions<f64>,
    pub underline_width_cell_ratio: f64,
    pub underline_top_offset_cell_ratio: f64,
    pub force_secondary_adapter: bool,
}

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

const fn dimensions_from_logical_size(size: winit::dpi::LogicalSize<f64>) -> Dimensions<f64> {
    Dimensions {
        width: size.width,
        height: size.height,
    }
}

fn populate_and_finish_buffer<T>(buffer: wgpu::Buffer, slice: &[T]) -> wgpu::Buffer
where
    T: zerocopy::IntoBytes + zerocopy::Immutable,
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

struct WgpuState {
    device: wgpu::Device,
    surface_configuration: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    render_pipeline: wgpu::RenderPipeline,
    background_cell_instance_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    queue: wgpu::Queue,
    background_cell_instance_data: Grid<BackgroundCellInstance>,
    chargrid_frame_buffer: FrameBuffer,
    global_uniforms_buffer: wgpu::Buffer,
    window_size: winit::dpi::LogicalSize<f64>,
    scale_factor: f64,
    modifier_state: winit::keyboard::ModifiersState,
    global_uniforms_to_sync: Option<GlobalUniforms>,
    text_renderer: TextRenderer,
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::IntoBytes, zerocopy::FromZeros, zerocopy::Immutable)]
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
#[derive(Debug, Clone, Copy, zerocopy::IntoBytes, zerocopy::FromZeros, zerocopy::Immutable)]
struct GlobalUniforms {
    cell_size_relative_to_window: [f32; 2],
    offset_to_centre: [f32; 2],
    grid_width: u32,
    underline_width_cell_ratio: f32,
    underline_top_offset_cell_ratio: f32,
    pad0: u32, // pad the type to 32 bytes
}

struct Setup {
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

async fn request_adapter_for_backend(
    backends: wgpu::Backends,
    window: Arc<winit::window::Window>,
) -> anyhow::Result<(
    wgpu::Adapter,
    wgpu::Surface<'static>,
    wgpu::Device,
    wgpu::Queue,
)> {
    let instance_descriptor = wgpu::InstanceDescriptor {
        backends,
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::default(),
    };
    let instance = wgpu::Instance::new(&instance_descriptor);
    let surface = instance
        .create_surface(window)
        .map_err(|e| anyhow!("Unable to create surface! ({:?})", e))?;
    let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, None)
        .await
        .map_err(|e| anyhow!("No suitable GPU adapters found on the system: {}", e))?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .map_err(|e| anyhow!("Unable to find a suitable GPU adapter! ({:?})", e))?;
    Ok((adapter, surface, device, queue))
}

async fn setup(window: Arc<winit::window::Window>, force_secondary_adapter: bool) -> Setup {
    let mut backends_to_try_reverse_order = vec![
        (wgpu::Backends::SECONDARY, "secondary"),
        (wgpu::Backends::PRIMARY, "primary"),
    ];
    if force_secondary_adapter {
        let (backend, _) = backends_to_try_reverse_order.pop().unwrap();
        assert!(backend == wgpu::Backends::PRIMARY);
    }
    if let Some(env_backends) = wgpu::Backends::from_env() {
        backends_to_try_reverse_order.push((env_backends, "environment"));
    }
    let (adapter, surface, device, queue) = loop {
        if let Some((backends, name)) = backends_to_try_reverse_order.pop() {
            match request_adapter_for_backend(backends, window.clone()).await {
                Ok(x) => {
                    log::info!(
                        "Initialized one of the \"{}\" WGPU backends ({:?})!",
                        name,
                        backends,
                    );
                    break x;
                }
                Err(message) => {
                    log::error!(
                        "Failed to initialize one of the \"{}\" WGPU backends ({:?}):\n{}",
                        name,
                        backends,
                        message,
                    );
                }
            }
        } else {
            panic!("Failed to initialize any WGPU backend!")
        }
    };
    Setup {
        surface,
        adapter,
        device,
        queue,
    }
}

struct WgpuStateArgs<'a> {
    window: Arc<winit::window::Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter: &'a wgpu::Adapter,
    surface: wgpu::Surface<'static>,
    sizes: &'a Sizes,
    grid_size: UCoord,
    font_bytes: FontBytes,
}

impl WgpuState {
    fn spirv_slice_to_shader_module_source(spirv_slice: &[u8]) -> wgpu::ShaderSource<'_> {
        assert!(spirv_slice.len().is_multiple_of(4));
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
        WgpuStateArgs {
            window,
            device,
            queue,
            adapter,
            surface,
            sizes,
            grid_size,
            font_bytes,
        }: WgpuStateArgs,
    ) -> Self {
        use std::mem;
        let num_background_cell_instances = grid_size.count();
        let background_cell_instance_data = Grid::new_default(grid_size);
        let chargrid_frame_buffer = FrameBuffer::new(grid_size);
        let scale_factor = window.scale_factor();
        let physical_size = window.inner_size();
        let window_size: winit::dpi::LogicalSize<f64> = physical_size.to_logical(scale_factor);
        let caps = surface.get_capabilities(adapter);
        let texture_format = caps.formats[0];
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        enum Shaders {
            Spv {
                vertex: wgpu::ShaderModule,
                fragment: wgpu::ShaderModule,
                entry_point: &'static str,
            },
            Wgsl {
                module: wgpu::ShaderModule,
                vertex_entry_point: &'static str,
                fragment_entry_point: &'static str,
            },
        }
        impl Shaders {
            fn vertex_module(&self) -> &wgpu::ShaderModule {
                match self {
                    Self::Spv { vertex, .. } => vertex,
                    Self::Wgsl { module, .. } => module,
                }
            }
            fn fragment_module(&self) -> &wgpu::ShaderModule {
                match self {
                    Self::Spv { fragment, .. } => fragment,
                    Self::Wgsl { module, .. } => module,
                }
            }
            fn vertex_entry_point(&self) -> &str {
                match self {
                    Self::Spv { entry_point, .. } => entry_point,
                    Self::Wgsl {
                        vertex_entry_point, ..
                    } => vertex_entry_point,
                }
            }
            fn fragment_entry_point(&self) -> &str {
                match self {
                    Self::Spv { entry_point, .. } => entry_point,
                    Self::Wgsl {
                        fragment_entry_point,
                        ..
                    } => fragment_entry_point,
                }
            }
        }
        let shaders = match adapter.get_info().backend {
            wgpu::Backend::Gl => {
                log::warn!("Using SPV shaders");
                Shaders::Spv {
                    vertex: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: Self::spirv_slice_to_shader_module_source(include_bytes!(
                            "./shader.vert.spv"
                        )),
                    }),
                    fragment: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: Self::spirv_slice_to_shader_module_source(include_bytes!(
                            "./shader.frag.spv"
                        )),
                    }),
                    entry_point: "main",
                }
            }
            _other => Shaders::Wgsl {
                module: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
                }),
                vertex_entry_point: "vs_main",
                fragment_entry_point: "fs_main",
            },
        };
        surface.configure(&device, &surface_configuration);
        let background_cell_instance_buffer = populate_and_finish_buffer(
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: num_background_cell_instances as u64
                    * std::mem::size_of::<BackgroundCellInstance>() as u64,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: true,
            }),
            background_cell_instance_data.raw(),
        );
        let global_uniforms_size = mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress;
        let global_uniforms = sizes.global_uniforms(dimensions_from_logical_size(window_size));
        let global_uniforms_buffer = populate_and_finish_buffer(
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: global_uniforms_size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: true,
            }),
            &[global_uniforms],
        );
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::all(),
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
            bind_group_layouts: &[&bind_group_layout],
            ..Default::default()
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shaders.vertex_module(),
                entry_point: Some(shaders.vertex_entry_point()),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<BackgroundCellInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
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
                            format: wgpu::VertexFormat::Uint32,
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
                module: shaders.fragment_module(),
                entry_point: Some(shaders.fragment_entry_point()),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(surface_configuration.format.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
        let modifier_state = winit::keyboard::ModifiersState::default();
        let text_renderer = TextRenderer::new(TextRendererArgs {
            font_bytes,
            device: &device,
            queue: &queue,
            texture_format,
            font_size_px: sizes.font_size_px,
            cell_dimensions: sizes.cell_dimensions,
            grid_size,
            window_scale_factor: scale_factor,
        });
        Self {
            device,
            surface_configuration,
            surface,
            render_pipeline,
            background_cell_instance_buffer,
            bind_group,
            queue,
            background_cell_instance_data,
            chargrid_frame_buffer,
            global_uniforms_buffer,
            window_size,
            scale_factor,
            modifier_state,
            global_uniforms_to_sync: None,
            text_renderer,
        }
    }
    fn render_background(&mut self) {
        for (buffer_cell, background_cell_instance) in self
            .chargrid_frame_buffer
            .iter()
            .zip(self.background_cell_instance_data.iter_mut())
        {
            background_cell_instance.background_colour =
                rgb_to_srgb(buffer_cell.background.to_f32_array_rgb_01());
            background_cell_instance.foreground_colour =
                rgb_to_srgb(buffer_cell.foreground.to_f32_array_rgb_01());
            background_cell_instance.underline = buffer_cell.underline as u32;
        }

        self.background_cell_instance_buffer = populate_and_finish_buffer(
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.chargrid_frame_buffer.size().count() as u64
                    * std::mem::size_of::<BackgroundCellInstance>() as u64,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: true,
            }),
            self.background_cell_instance_data.raw(),
        );
    }

    fn resize(&mut self, sizes: &Sizes, physical_size: winit::dpi::PhysicalSize<u32>) {
        let logical_size = physical_size.to_logical(self.scale_factor);
        self.window_size = logical_size;
        self.background_cell_instance_buffer = populate_and_finish_buffer(
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.chargrid_frame_buffer.size().count() as u64
                    * std::mem::size_of::<BackgroundCellInstance>() as u64,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: true,
            }),
            self.background_cell_instance_data.raw(),
        );
        let global_uniforms = sizes.global_uniforms(dimensions_from_logical_size(logical_size));
        self.global_uniforms_to_sync = Some(global_uniforms);
    }

    fn sync_global_uniforms(&mut self, encoder: &mut wgpu::CommandEncoder) {
        use std::mem;
        if let Some(global_uniforms) = self.global_uniforms_to_sync.take() {
            let temp_buffer = populate_and_finish_buffer(
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: std::mem::size_of::<GlobalUniforms>() as u64,
                    usage: wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: true,
                }),
                &[global_uniforms],
            );
            encoder.copy_buffer_to_buffer(
                &temp_buffer,
                0,
                &self.global_uniforms_buffer,
                0,
                mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress,
            );
        }
    }

    fn render_foreground(&mut self) {
        if let Ok(frame) = self.surface.get_current_texture() {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            self.sync_global_uniforms(&mut encoder);
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.background_cell_instance_buffer.slice(..));
                render_pass.draw(0..6, 0..self.chargrid_frame_buffer.size().count() as u32);
                self.text_renderer
                    .render(
                        &self.chargrid_frame_buffer,
                        &self.surface_configuration,
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                    )
                    .unwrap();
            }
            self.queue.submit(std::iter::once(encoder.finish()));
            frame.present();
        } else {
            log::warn!("timeout when acquiring next swapchain texture");
        }
    }

    fn render(&mut self) {
        self.render_background();
        self.render_foreground();
    }
}

struct InputState {
    last_mouse_coord: ICoord,
    last_mouse_button: Option<chargrid_input::MouseButton>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            last_mouse_coord: ICoord::new(0, 0),
            last_mouse_button: None,
        }
    }
}

#[derive(Debug)]
struct Sizes {
    font_size_px: f32,
    cell_dimensions: Dimensions<f64>,
    underline_width: f64,
    underline_top_offset: f64,
    native_window_dimensions: Dimensions<f64>,
}

impl Sizes {
    fn grid_size(&self) -> UCoord {
        let width = (self.native_window_dimensions.width / self.cell_dimensions.width).floor();
        let height = (self.native_window_dimensions.height / self.cell_dimensions.height).floor();
        UCoord::new(width as u32, height as u32)
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
            pad0: 0,
        }
    }
}

struct Context {
    window: Arc<winit::window::Window>,
    wgpu_state: WgpuState,
    sizes: Sizes,
    input_state: InputState,
    #[cfg(feature = "gamepad")]
    gamepad: GamepadContext,
}

impl Context {
    fn new(
        window: Arc<winit::window::Window>,
        Config {
            dimensions_px,
            font_bytes,
            font_size_px,
            cell_dimensions_px,
            underline_width_cell_ratio,
            underline_top_offset_cell_ratio,
            force_secondary_adapter,
            ..
        }: Config,
    ) -> Self {
        let Setup {
            surface,
            adapter,
            device,
            queue,
        } = pollster::block_on(setup(window.clone(), force_secondary_adapter));
        let sizes = Sizes {
            font_size_px,
            cell_dimensions: cell_dimensions_px,
            underline_width: underline_width_cell_ratio,
            underline_top_offset: underline_top_offset_cell_ratio,
            native_window_dimensions: dimensions_px,
        };
        let grid_size = sizes.grid_size();
        let wgpu_state = WgpuState::new(WgpuStateArgs {
            window: window.clone(),
            device,
            queue,
            adapter: &adapter,
            surface,
            sizes: &sizes,
            grid_size,
            font_bytes,
        });
        log::info!("grid size: {:?}", grid_size);
        Context {
            window: window.clone(),
            wgpu_state,
            sizes,
            input_state: Default::default(),
            #[cfg(feature = "gamepad")]
            gamepad: GamepadContext::new(),
        }
    }
}

struct AppState {
    exited: bool,
    frame_instant: Instant,
    last_update_instant: Instant,
    current_window_dimensions: Dimensions<f64>,
}

impl AppState {
    fn new() -> Self {
        Self {
            exited: false,
            frame_instant: Instant::now(),
            last_update_instant: Instant::now(),
            current_window_dimensions: Dimensions {
                width: 0.,
                height: 0.,
            },
        }
    }
}

struct App<C>
where
    C: 'static + Component<State = (), Output = app::Output>,
{
    config: Config,
    context: Option<Context>,
    state: AppState,
    component: C,
}

impl<C> App<C>
where
    C: 'static + Component<State = (), Output = app::Output>,
{
    fn new(component: C, config: Config) -> Self {
        Self {
            config,
            context: None,
            state: AppState::new(),
            component,
        }
    }
}

impl<C> ApplicationHandler for App<C>
where
    C: 'static + Component<State = (), Output = app::Output>,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let logical_size = winit::dpi::LogicalSize::new(
            self.config.dimensions_px.width,
            self.config.dimensions_px.height,
        );
        let window_attributes = winit::window::Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(logical_size)
            .with_min_inner_size(logical_size)
            .with_max_inner_size(logical_size)
            .with_resizable(self.config.resizable);
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let context = Context::new(window.clone(), self.config.clone());
        self.state.frame_instant = Instant::now();
        self.state.last_update_instant = self.state.frame_instant;
        self.state.current_window_dimensions = context.sizes.native_window_dimensions;
        self.context = Some(context);
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.state.exited {
            event_loop.exit();
            return;
        }
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let context = if let Some(context) = self.context.as_mut() {
            context
        } else {
            return;
        };

        #[cfg(feature = "gamepad")]
        for input in context.gamepad.drain_input() {
            if let Some(app::Exit) = on_input(
                &mut self.component,
                chargrid_input::Input::Gamepad(input),
                &context.wgpu_state.chargrid_frame_buffer,
            ) {
                self.state.exited = true;
                return;
            }
        }
        match event {
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                context.wgpu_state.modifier_state = modifiers.state();
            }
            winit::event::WindowEvent::RedrawRequested => {
                let target_frametime = Duration::from_secs_f64(1.0 / 60.0);
                let time_since_last_frame = self.state.last_update_instant.elapsed();
                let update_on_this_frame = time_since_last_frame >= target_frametime;
                if update_on_this_frame {
                    self.state.last_update_instant = Instant::now();
                    let frame_duration = self.state.frame_instant.elapsed();
                    self.state.frame_instant = Instant::now();
                    if let Some(app::Exit) = on_frame(
                        &mut self.component,
                        frame_duration,
                        &mut context.wgpu_state.chargrid_frame_buffer,
                    ) {
                        self.state.exited = true;
                        return;
                    }
                }
                context.wgpu_state.render();
                context.window.request_redraw();
            }
            other => {
                if let Some(event) = input::convert_event(
                    other,
                    context
                        .sizes
                        .scaled_cell_dimensions(self.state.current_window_dimensions),
                    context
                        .sizes
                        .pixel_offset_to_centre_native_window(self.state.current_window_dimensions),
                    &mut context.input_state.last_mouse_coord,
                    &mut context.input_state.last_mouse_button,
                    &mut context.wgpu_state.scale_factor,
                    context.wgpu_state.modifier_state,
                ) {
                    match event {
                        input::Event::Input(input) => {
                            if let Some(app::Exit) = on_input(
                                &mut self.component,
                                input,
                                &context.wgpu_state.chargrid_frame_buffer,
                            ) {
                                self.state.exited = true;
                            }
                        }
                        input::Event::Resize(size) => {
                            context.wgpu_state.resize(&context.sizes, size);
                            self.state.current_window_dimensions =
                                dimensions_from_logical_size(context.wgpu_state.window_size);
                        }
                    }
                }
            }
        }
    }
}

/**
 * Runs a component. Each frame the given component is rendered by invoking its `render` method,
 * and a `Event::Tick` event is passed to the component's `update` method set to the time since the
 * previous frame. Each time an input event is received an `Event::Input` event is passed to the
 * component's `update` method. When the component yields `Some(app::Exit)`, the program will exit.
 */
pub fn run<C>(component: C, config: Config) -> anyhow::Result<()>
where
    C: 'static + Component<State = (), Output = app::Output>,
{
    let event_loop = winit::event_loop::EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(component, config);
    event_loop.run_app(&mut app)?;
    Ok(())
}

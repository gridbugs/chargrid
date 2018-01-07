use gfx;

use prototty::Size;
use formats::*;

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
const QUAD_COORDS: [[f32; 2]; 4] = [[0.0, 0.0],
                                    [0.0, 1.0],
                                    [1.0, 1.0],
                                    [1.0, 0.0]];

gfx_vertex_struct!( Vertex {
    corner_multiplier: [f32; 2] = "a_CornerMultiplier",
});

gfx_vertex_struct!( Instance {
    top_left_corner_pix_pos: [f32; 2] = "a_TopLeftCornerPixPos",
    background_colour: [f32; 4] = "a_BackgroundColour",
    foreground_colour: [f32; 4] = "a_ForegroundColour",
    underline: u32 = "a_Underline",
});

gfx_constant_struct!( Properties {
    cell_pix_size: [f32; 2] = "u_CellPixSize",
    window_pix_size: [f32; 2] = "u_WindowPixSize",
    underline_pix_width: f32 = "u_UnderlinePixWidth",
    underline_pix_pos: f32 = "u_UnderlinePixPos",
});

gfx_pipeline!( pipe {
    properties: gfx::ConstantBuffer<Properties> = "Properties",
    vertex: gfx::VertexBuffer<Vertex> = (),
    instance: gfx::InstanceBuffer<Instance> = (),
    out_colour: gfx::BlendTarget<ColourFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
});

pub struct BackgroundRenderer<R: gfx::Resources> {
    bundle: gfx::pso::bundle::Bundle<R, pipe::Data<R>>,
    instance_upload: gfx::handle::Buffer<R, Instance>,
    num_instances: usize,
}

impl<R: gfx::Resources> BackgroundRenderer<R> {
    pub fn new<F, C>(
        window_width: u32,
        window_height: u32,
        cell_width: u32,
        cell_height: u32,
        size: Size,
        underline_width: u32,
        underline_position: u32,
        rtv: gfx::handle::RenderTargetView<R, ColourFormat>,
        factory: &mut F,
        encoder: &mut gfx::Encoder<R, C>) -> Self
        where F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
              C: gfx::CommandBuffer<R>,
    {

        let pso = factory.create_pipeline_simple(
            include_bytes!("shaders/background.150.vert"),
            include_bytes!("shaders/background.150.frag"),
            pipe::new()).expect("Failed to create pipeline");

        let vertex_data: Vec<Vertex> = QUAD_COORDS.iter()
            .map(|v| {
                Vertex {
                    corner_multiplier: *v,
                }
            }).collect();

        let (vertex_buffer, mut slice) =
            factory.create_vertex_buffer_with_slice(
                &vertex_data,
                &QUAD_INDICES[..]);

        let num_instances = size.count();

        slice.instances = Some((num_instances as u32, 0));

        let instance_buffer = factory.create_buffer(
            num_instances,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Data,
            gfx::memory::Bind::TRANSFER_DST)
            .expect("Failed to create instance buffer");

        let data = pipe::Data {
            properties: factory.create_constant_buffer(1),
            vertex: vertex_buffer,
            instance: instance_buffer,
            out_colour: rtv,
        };

        let instance_upload: gfx::handle::Buffer<R, Instance> =
            factory.create_upload_buffer(num_instances).expect("Failed to create instance upload buffer");

        for (coord, instance) in izip!(
            size.coords(),
            factory.write_mapping(&instance_upload).expect("Failed to map instance upload buffer").iter_mut()
        ) {
            let x = (coord.x as u32 * cell_width) as f32;
            let y = (coord.y as u32 * cell_height) as f32;
            instance.top_left_corner_pix_pos = [x, y];
        }

        encoder.update_constant_buffer(&data.properties, &Properties {
            cell_pix_size: [cell_width as f32, cell_height as f32],
            window_pix_size: [window_width as f32, window_height as f32],
            underline_pix_width: underline_width as f32,
            underline_pix_pos: underline_position as f32,
        });

        Self {
            bundle: gfx::pso::bundle::Bundle::new(slice, pso, data),
            instance_upload,
            num_instances,
        }
    }

    pub fn map_cells<F: gfx::Factory<R>>(&mut self, factory: &mut F) -> gfx::mapping::Writer<R, Instance> {
        factory.write_mapping(&self.instance_upload).expect("Failed to map instance upload buffer")
    }

    pub fn draw<C>(&self, encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>,
    {
        encoder.copy_buffer(&self.instance_upload, &self.bundle.data.instance, 0, 0, self.num_instances)
            .expect("Failed to copy instances");
        encoder.draw(&self.bundle.slice, &self.bundle.pso, &self.bundle.data);
    }
}

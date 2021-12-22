struct Globals {
    cell_size_relative_to_window: vec2<f32>;
    offset_to_centre: vec2<f32>;
    grid_width: u32;
    underline_width_cell_ratio: f32;
    underline_top_offset_cell_ratio: f32;
    pad0: u32; // pad the type to 32 bytes
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0), interpolate(flat)]] background_colour: vec3<f32>;
    [[location(1), interpolate(flat)]] foreground_colour: vec3<f32>;
    [[location(2)]] cell_ratio: f32;
    [[location(3), interpolate(flat)]] underline: bool;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] background_colour: vec3<f32>,
    [[location(1)]] foreground_colour: vec3<f32>,
    [[location(2)]] underline: u32,
    [[builtin(vertex_index)]] in_vertex_index: u32,
    [[builtin(instance_index)]] in_instance_index: u32,
) -> VertexOutput {

    // The magic numbers have the binary representation such that the shift and
    // mask operations choose the appropriate cell corner for a vertex index.
    let corner_offset_x = f32((22 >> in_vertex_index) & 1);
    let corner_offset_y = f32((52 >> in_vertex_index) & 1);
    let corner_offset = vec2<f32>(corner_offset_x, corner_offset_y);
    let cell_size = globals.cell_size_relative_to_window;
    let grid_width = globals.grid_width;
    let coord_x = f32(in_instance_index % grid_width);
    let coord_y = f32(in_instance_index / grid_width);
    let coord = vec2<f32>(coord_x, coord_y);
    let scaled_corner_offset =  corner_offset * cell_size;
    let top_left_corner = coord * cell_size;
    let absolute =
        vec2<f32>(-1.0, -1.0) + top_left_corner + scaled_corner_offset + globals.offset_to_centre;

    var out: VertexOutput;
    out.cell_ratio = corner_offset_y;
    out.background_colour = background_colour;
    out.foreground_colour = foreground_colour;
    out.underline = underline != 0u;
    out.position = vec4<f32>(absolute.x, -absolute.y, 0.0, 1.0);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let pixel_is_underline =
        in.underline &&
        in.cell_ratio >= globals.underline_top_offset_cell_ratio &&
        in.cell_ratio <= globals.underline_top_offset_cell_ratio + globals.underline_width_cell_ratio;
    if (pixel_is_underline) {
        return vec4<f32>(in.foreground_colour, 1.0);
    } else {
        return vec4<f32>(in.background_colour, 1.0);
    }
}

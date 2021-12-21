struct Globals {
    cell_size_relative_to_window: vec2<f32>;
    offset_to_centre: vec2<f32>;
    grid_width: u32;
    underline_width_cell_ratio: f32;
    underline_top_offset_cell_ratio: f32;
    pad0: u32;
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] background_colour: vec3<f32>;
    [[location(1), interpolate(flat)]] foreground_colour: vec3<f32>;

    // We only need the y component of this ratio, but opengl complains if this is a float.
    // Error is: "initializer of type int cannot be assigned to variable of type float"
    [[location(2)]] cell_ratio: vec2<f32>;

    // When the fragment shader reads this uniform in opengl it seems to get a value other
    // than what was placed in the constant buffer, so make it a varying instead???
    [[location(3), interpolate(flat)]] underline_x_top_off_cell_ratio_y_width_cell_ratio: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] background_colour: vec3<f32>,
    [[location(1)]] foreground_colour: vec3<f32>,
    [[location(2)]] underline: i32,
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

    out.cell_ratio = corner_offset;
    out.background_colour = background_colour;
    out.foreground_colour = foreground_colour;

    if (underline == 0) {
        // setting the underline ratio > 1 means that no pixel will be within it, disablihng the underline
        out.underline_x_top_off_cell_ratio_y_width_cell_ratio = vec2<f32>(2.0, 0.0);
    } else {
        out.underline_x_top_off_cell_ratio_y_width_cell_ratio = vec2<f32>(
            globals.underline_top_offset_cell_ratio,
            globals.underline_width_cell_ratio,
        );
    }

    out.position = vec4<f32>(absolute.x, -absolute.y, 0.0, 1.0);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    if (in.cell_ratio.y >= in.underline_x_top_off_cell_ratio_y_width_cell_ratio.x &&
        in.cell_ratio.y >= in.underline_x_top_off_cell_ratio_y_width_cell_ratio.x + in.underline_x_top_off_cell_ratio_y_width_cell_ratio.y)
    {
        return vec4<f32>(in.foreground_colour, 1.0);
    } else {
        return vec4<f32>(in.background_colour, 1.0);
    }
}

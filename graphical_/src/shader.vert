#version 450

layout(location = 0) in vec3 a_BackgroundColour;
layout(location = 1) in vec3 a_ForegroundColour;

layout(location = 0) flat out vec3 v_BackgroundColour;
layout(location = 1) flat out vec3 v_ForegroundColour;

layout(set = 0, binding = 0) uniform Globals {
    vec2 u_CellSizeRelativeToWindow;
    uint u_GridWidth;
};

out gl_PerVertex {
    vec4 gl_Position;
};

const vec2 corner_offsets[6] = vec2[6](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0)
);

void main() {
    v_BackgroundColour = a_BackgroundColour;
    v_ForegroundColour = a_ForegroundColour;
    vec2 cell_size = u_CellSizeRelativeToWindow;
    //vec2 cell_size = vec2(0.1, 0.1);
    uint grid_width = u_GridWidth;
    uint coord_x = gl_InstanceIndex % grid_width;
    uint coord_y = gl_InstanceIndex / grid_width;
    vec2 coord = vec2(coord_x, coord_y);
    vec2 corner_offset = corner_offsets[gl_VertexIndex] * cell_size;
    vec2 top_left_corner = coord * cell_size;
    vec2 absolute = vec2(-1.0, -1.0) + top_left_corner + corner_offset;
    gl_Position = vec4(absolute, 0.0, 1.0);
}

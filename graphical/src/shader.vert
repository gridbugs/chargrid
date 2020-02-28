#version 450

layout(location = 0) in vec3 a_BackgroundColour;
layout(location = 1) in vec3 a_ForegroundColour;
layout(location = 2) in uint a_Underline;

layout(location = 0) flat out vec3 v_BackgroundColour;
layout(location = 1) flat out vec3 v_ForegroundColour;
layout(location = 2) flat out uint v_Underline;
layout(location = 3) out float v_CellRatioY;

layout(set = 0, binding = 0) uniform Globals {
    vec2 u_CellSizeRelativeToWindow;
    vec2 u_OffsetTtoCentre;
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
    v_Underline = a_Underline;
    vec2 cell_size = u_CellSizeRelativeToWindow;
    uint grid_width = u_GridWidth;
    uint coord_x = gl_InstanceIndex % grid_width;
    uint coord_y = gl_InstanceIndex / grid_width;
    vec2 coord = vec2(coord_x, coord_y);
    vec2 corner_offset = corner_offsets[gl_VertexIndex];
    vec2 scaled_corner_offset = corner_offsets[gl_VertexIndex] * cell_size;
    vec2 top_left_corner = coord * cell_size;
    vec2 absolute = vec2(-1.0, -1.0) + top_left_corner + scaled_corner_offset + u_OffsetTtoCentre;
    v_CellRatioY = corner_offset.y;
    gl_Position = vec4(absolute, 0.0, 1.0);
}

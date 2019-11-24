#version 450

layout(location = 0) in float a_Size;
layout(location = 1) in vec3 a_Col;

layout(location = 0) flat out vec3 v_Col;

layout(set = 0, binding = 0) uniform Globals {
    uint u_PerRow;
};

out gl_PerVertex {
    vec4 gl_Position;
};

const vec2 positions[3] = vec2[3](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

const float rows[2] = float[2](
    -0.5,
    0.5
);

const float cols[3] = float[3](
    -0.5,
    0.0,
    0.5
);

void main() {
    v_Col = a_Col;

    vec2 relative = positions[gl_VertexIndex] * a_Size;
    uint row = gl_InstanceIndex / u_PerRow;
    uint col = gl_InstanceIndex % u_PerRow;
    vec2 mid = vec2(cols[col], rows[row]);
    vec2 absolute = relative + mid;
    gl_Position = vec4(absolute, 0.0, 1.0);
}

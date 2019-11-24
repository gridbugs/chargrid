#version 450

layout(location = 0) out vec4 outColor;

layout(location = 0) flat in vec3 v_Col;

void main() {
    outColor = vec4(v_Col, 1.0);
}

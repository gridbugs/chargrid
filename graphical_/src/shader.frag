#version 450

layout(location = 0) out vec4 outColor;

layout(location = 0) flat in vec3 v_BackgroundColour;
layout(location = 1) flat in vec3 v_ForegroundColour;

void main() {
    outColor = vec4(v_BackgroundColour, 1.0);
}

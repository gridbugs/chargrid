#version 150 core

out vec4 Target0;

flat in vec4 v_BackgroundColour;
flat in vec4 v_ForegroundColour;
flat in uint v_Underline;

void main() {
    Target0 = v_BackgroundColour;
}

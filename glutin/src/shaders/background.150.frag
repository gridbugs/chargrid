#version 150 core

out vec4 Target0;

uniform Properties {
    vec2 u_CellPixSize;
    vec2 u_WindowPixSize;
    float u_UnderlinePixWidth;
    float u_UnderlinePixPos;
};

flat in vec4 v_BackgroundColour;
flat in vec4 v_ForegroundColour;
flat in uint v_Underline;

in float v_DistanceFromTopPix;

void main() {

    if (v_Underline == 1u &&
            v_DistanceFromTopPix > u_UnderlinePixPos &&
            v_DistanceFromTopPix <= u_UnderlinePixPos + u_UnderlinePixWidth) {
        Target0 = v_ForegroundColour;
    } else {
        Target0 = v_BackgroundColour;
    }
}

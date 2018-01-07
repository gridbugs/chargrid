#version 150 core

in vec2 a_CornerMultiplier;

in vec2 a_TopLeftCornerPixPos;
in vec4 a_BackgroundColour;
in vec4 a_ForegroundColour;
in uint a_Underline;

uniform Properties {
    vec2 u_CellPixSize;
    vec2 u_WindowPixSize;
    float u_UnderlinePixWidth;
    float u_UnderlinePixPos;
};

flat out vec4 v_BackgroundColour;
flat out vec4 v_ForegroundColour;
flat out uint v_Underline;

out float v_DistanceFromTopPix;

void main() {

    v_BackgroundColour = a_BackgroundColour;
    v_ForegroundColour = a_ForegroundColour;
    v_Underline = a_Underline;
    v_DistanceFromTopPix = a_CornerMultiplier.y * u_CellPixSize.y;

    vec2 output_pix = a_TopLeftCornerPixPos + a_CornerMultiplier * u_CellPixSize;
    vec2 output_coord = (2.0 * output_pix / u_WindowPixSize);
    output_coord.x -= 1.0;
    output_coord.y = 1.0 - output_coord.y;

    gl_Position = vec4(output_coord, 0.0, 1.0);
}

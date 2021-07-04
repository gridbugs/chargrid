#version 450

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec3 v_BackgroundColour;
layout(location = 1) flat in vec3 v_ForegroundColour;
layout(location = 2) in vec2 v_CellRatio;
layout(location = 3) flat in vec2 v_Underline_XTopOffCellRatio_YWidthCellRatio;

void main() {
  if (v_CellRatio.y >= v_Underline_XTopOffCellRatio_YWidthCellRatio.x &&
      v_CellRatio.y >= v_Underline_XTopOffCellRatio_YWidthCellRatio.x + v_Underline_XTopOffCellRatio_YWidthCellRatio.y) {
    outColor = vec4(v_ForegroundColour, 1.0);
  } else {
    outColor = vec4(v_BackgroundColour, 1.0);
  }
}

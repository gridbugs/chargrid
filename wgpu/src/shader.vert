#version 450

layout(location = 0) in vec3 a_BackgroundColour;
layout(location = 1) in vec3 a_ForegroundColour;
layout(location = 2) in int a_Underline;

layout(location = 0) out vec3 v_BackgroundColour;
layout(location = 1) flat out vec3 v_ForegroundColour;
// We only need the y component of this ratio, but opengl complains if this is a float.
// Error is: "initializer of type int cannot be assigned to variable of type float"
layout(location = 2) out vec2 v_CellRatio;
// When the fragment shader reads this uniform in opengl it seems to get a valud other
// than what was placed in the constant buffer.
layout(location = 3) flat out vec2 v_Underline_XTopOffCellRatio_YWidthCellRatio;

layout(set = 0, binding = 0) uniform Globals {
  vec2 u_CellSizeRelativeToWindow;
  vec2 u_OffsetTtoCentre;
  int u_GridWidth;
  float u_UnderlineWidthCellRatio;
  float u_UnderlineTopOffsetCellRatio;
};

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
  // The magic numbers have the binary representation such that the shift and
  // mask operations choose the appropriate cell corner for a vertex index.
  float corner_offset_x = float((22 >> gl_VertexIndex) & 1);
  float corner_offset_y = float((52 >> gl_VertexIndex) & 1);
  vec2 corner_offset = vec2(corner_offset_x, corner_offset_y);
  v_BackgroundColour = a_BackgroundColour;
  v_ForegroundColour = a_ForegroundColour;
  vec2 cell_size = u_CellSizeRelativeToWindow;
  uint grid_width = u_GridWidth;
  uint coord_x = gl_InstanceIndex % grid_width;
  uint coord_y = gl_InstanceIndex / grid_width;
  vec2 coord = vec2(coord_x, coord_y);
  vec2 scaled_corner_offset =  corner_offset * cell_size;
  vec2 top_left_corner = coord * cell_size;
  vec2 absolute = vec2(-1.0, -1.0) + top_left_corner + scaled_corner_offset + u_OffsetTtoCentre;
  absolute.y *= -1;
  v_CellRatio = corner_offset;
  if (a_Underline == 0) {
    // setting the underline ratio > 1 means that no pixel will be within it, disablihng the underline
    v_Underline_XTopOffCellRatio_YWidthCellRatio = vec2(2, 0);
  } else {
    v_Underline_XTopOffCellRatio_YWidthCellRatio = vec2(u_UnderlineTopOffsetCellRatio, u_UnderlineWidthCellRatio);
  }
  gl_Position = vec4(absolute, 0.0, 1.0);
}

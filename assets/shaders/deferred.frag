#version 450
layout(location = 0) in vec4 v_color;
layout(location = 1) in vec4 v_normal;

layout(location = 0) out vec4 f_color;
layout(location = 1) out vec4 f_normal;

void main() {
  f_color = v_color;
  f_normal = v_normal;
}

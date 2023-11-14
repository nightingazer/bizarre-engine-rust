#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;

layout(set = 0, binding = 0) uniform DeferredVertUniforms {
  mat4 view_projection;
} uniforms;

layout(location = 0) out vec3 v_color;
layout(location = 1) out vec3 v_normal;

void main() {
  gl_Position = uniforms.view_projection * vec4(position, 1.0);
  v_color = color;
  v_normal = normal;
}

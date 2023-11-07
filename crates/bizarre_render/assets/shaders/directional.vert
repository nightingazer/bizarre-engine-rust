#version 450

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 v_frag_position;

layout(set = 0, binding = 2) uniform MVP_Data {
  mat4 model;
  mat4 view;
  mat4 projection;
}
uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.model;
    v_frag_position = vec3(worldview * vec4(position, 1.0));
    gl_Position = uniforms.projection * worldview * vec4(position, 1.0);
}

#version 450

layout(location = 0) in vec2 position;

layout (set = 0, binding = 0) uniform UBO {
    mat4 view_projection;
} uniforms;

struct DirectionalData {
  vec3 direction;
  vec3 color;
};

layout(std140, set = 0, binding = 1) readonly buffer Directional_Data {
    DirectionalData lights[];
} lights;

layout(location = 0) out vec3 v_direction;
layout(location = 1) out vec3 v_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    DirectionalData light = lights.lights[gl_InstanceIndex];
    v_direction = light.direction;
    v_color = light.color;
}

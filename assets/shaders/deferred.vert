#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;

layout(set = 0, binding = 0) uniform UBO {
    mat4 view_projection;
    mat4 model[100];
} uniforms;

layout(push_constant, std430) uniform Constants {
    uint model_offset;
} constants;

layout(location = 0) out vec3 v_color;
layout(location = 1) out vec3 v_normal;

void main() {
    mat4 model = uniforms.model[constants.model_offset];
    gl_Position = uniforms.view_projection * model * vec4(position, 1.0);
    v_color = color;
    v_normal = normal;
}

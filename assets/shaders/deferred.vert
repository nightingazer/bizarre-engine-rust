#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;

layout(set = 0, binding = 0) uniform UBO {
    mat4 view_projection;
    mat4 model[100];
} uniforms;

// layout(push_constant, std430) uniform Constants {
//     uint model_offset;
// } constants;

layout(location = 0) out vec3 v_color;
layout(location = 1) out vec3 v_normal;

void main() {
    mat4 model = uniforms.model[gl_InstanceIndex];
    mat4 view_projection_model = uniforms.view_projection * model;
    gl_Position = view_projection_model * vec4(position, 1.0);
    vec3 VinView = normalize(-gl_Position.xyz);
    v_color = VinView;
    v_normal = (view_projection_model * vec4(normal, 0.0)).xyz;
}

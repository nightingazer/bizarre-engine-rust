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

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec4 v_normal;

void main() {
    mat4 model = uniforms.model[gl_InstanceIndex];
    mat4 view_projection_model = uniforms.view_projection * model;
    gl_Position = view_projection_model * vec4(position, 1.0);
    vec4 VinView = normalize(-gl_Position);
    v_color = vec4(1.0, 1.0, 1.0, 1.0);
    v_normal = (model * vec4(normal, 0.0));
    vec4 normInView = (uniforms.view_projection * v_normal);
    v_normal.w = dot(normInView, VinView) * 0.5 + 0.5;
}

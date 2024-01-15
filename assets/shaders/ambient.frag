#version 450

layout(input_attachment_index = 0, set = 0, binding = 1) uniform subpassInput u_color;
layout(input_attachment_index = 1, set = 0, binding = 2) uniform subpassInput u_normal;

layout(set = 0, binding = 0) uniform Ambient_Data {
    vec3 color;
    vec3 camera_forward;
} ubo;

layout(location = 0) out vec4 f_color;

void main() {
    vec4 color = subpassLoad(u_color);
    vec4 normal = subpassLoad(u_normal);
    float ambient_factor = min(mix(1.0, 0.25, dot(color, normal)), color.a);
    f_color = vec4(ubo.color * ambient_factor, 1.0);
}

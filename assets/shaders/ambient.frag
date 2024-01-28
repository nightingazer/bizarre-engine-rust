#version 450

layout(input_attachment_index = 0, set = 0, binding = 1) uniform subpassInput u_color;
layout(input_attachment_index = 1, set = 0, binding = 2) uniform subpassInput u_normal;

layout(set = 0, binding = 0) uniform Ambient_Data {
    vec3 color;
} ubo;

layout(location = 0) out vec4 f_color;

void main() {
    vec4 color = subpassLoad(u_color);
    vec4 normal = subpassLoad(u_normal);
    f_color = vec4(color * vec4(ubo.color, 1.0) * normal.w);
}

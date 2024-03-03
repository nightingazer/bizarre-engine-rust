#version 450

layout(set = 0, binding = 0) uniform Directional_Data {
  vec3 direction;
  vec3 color;
}
ubo;

layout(input_attachment_index = 0, set = 0, binding = 1) uniform subpassInputMS u_color;
layout(input_attachment_index = 1, set = 0, binding = 2) uniform subpassInputMS u_normal;
// layout(input_attachment_index = 0, set = 0, binding = 1) uniform subpassInput u_color;
// layout(input_attachment_index = 1, set = 0, binding = 2) uniform subpassInput u_normal;

layout(location = 0) out vec4 f_color;

void main() {
    vec4 color = (
        subpassLoad(u_color, 0) +
        subpassLoad(u_color, 1) +
        subpassLoad(u_color, 2) +
        subpassLoad(u_color, 3)
    ) / 4;

    vec4 normal_and_t = (
        subpassLoad(u_normal, 0) +
        subpassLoad(u_normal, 1) +
        subpassLoad(u_normal, 2) +
        subpassLoad(u_normal, 3)
    ) / 4;

    // vec4 color = subpassLoad(u_color);
    // vec4 normal_and_t = subpassLoad(u_normal);

    vec3 normal = normal_and_t.xyz;
    float view_tangent = normal_and_t.w;

    float light_intencity = dot(normal, ubo.direction) * view_tangent;

    f_color = vec4(color.xyz * ubo.color * light_intencity, 1.0);
}

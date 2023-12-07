#version 450

layout(set = 0, binding = 2) uniform Directional_Data {
  vec3 position;
  vec3 color;
}
directional;

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_color;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput u_normals;

layout(location = 0) out vec4 f_color;

void main() {
  vec3 normal = subpassLoad(u_normals).xyz;
  vec3 light_direction = normalize(directional.position.xyz - normal);
  float directional_intensity = max(dot(normal, light_direction), 0.0);
  vec3 directional_color = directional_intensity * directional.color;
  vec3 combined_color = directional_color * subpassLoad(u_color).rgb;
  f_color = vec4(combined_color, 1.0);
}

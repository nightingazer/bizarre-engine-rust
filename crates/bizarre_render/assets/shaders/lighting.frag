#version 450

layout(set = 0, binding = 3) uniform Ambient_Data {
  vec3 ambient_color;
  float ambient_intensity;
}
ambient;

layout(set = 0, binding = 4) uniform Directional_Data {
  vec3 position;
  vec3 color;
}
directional;

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_color;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput u_normals;

layout(location = 0) in vec3 v_frag_position;

layout(location = 0) out vec4 f_color;

void main() {
  vec3 ambient_color = ambient.ambient_color * ambient.ambient_intensity;
  vec3 light_direction = normalize(directional.position - v_frag_position);
  vec3 normal = normalize(subpassLoad(u_normals).rgb);
  float directional_intensity = max(dot(normal, light_direction), 0.0);
  vec3 directional_color = directional_intensity * directional.color;
  vec3 combined_color = (directional_color + ambient_color) * subpassLoad(u_color).rgb;
  f_color = vec4(combined_color, 1.0);
}

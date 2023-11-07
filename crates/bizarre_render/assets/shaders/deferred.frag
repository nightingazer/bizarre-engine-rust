#version 450
layout(location = 0) in vec3 v_color;
layout(location = 1) in vec3 v_normal;

layout(location = 0) out vec4 f_color;
layout(location = 1) out vec3 f_normal;

// layout(set = 0, binding = 1) uniform Ambient_Data {
//   vec3 ambient_color;
//   float ambient_intensity;
// }
// ambient;

// layout(set = 0, binding = 2) uniform Directional_Data {
//   vec3 position;
//   vec3 color;
// }
// directional;

void main() {
    f_color = vec4(v_color, 1.0);
    f_normal = v_normal;
}

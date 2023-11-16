#version 450

layout(location = 0) in vec3 v_eye_direction;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform samplerCube u_skybox;

void main() {
    f_color = texture(u_skybox, v_eye_direction);
}

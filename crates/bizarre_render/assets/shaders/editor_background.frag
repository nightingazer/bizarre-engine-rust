#version 450

layout(location = 0) in vec3 v_eye_direction;

layout(location = 0) out vec4 f_color;

void main() {
    float t = (v_eye_direction.y + 1.0) * 0.5;
    vec4 up_color = vec4(0.4, 0.7, 1.0, 1.0);
    vec4 down_color = vec4(0.2, 0.3, 0.4, 1.0);
    f_color = up_color * t + down_color * (1.0 - t);
}

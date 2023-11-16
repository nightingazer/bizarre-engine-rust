#version 450

layout(location = 0) in vec2 position;

layout(location = 0) out vec3 v_eye_direction;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 view;
    mat4 projection;
} uniforms;

void main() {
    mat4 inverse_projection = inverse(uniforms.projection);
    mat3 inverse_view = transpose(mat3(uniforms.view));
    vec3 unprojected = (inverse_projection * vec4(position, 1.0, 1.0)).xyz;
    v_eye_direction = normalize(inverse_view * unprojected);

    gl_Position = vec4(position, 1.0, 1.0);
}

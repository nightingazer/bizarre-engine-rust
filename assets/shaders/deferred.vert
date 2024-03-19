#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;

layout(set = 0, binding = 0) uniform UBO {
    mat4 view_projection;
} uniforms;

layout(std140, set = 0, binding = 1) readonly buffer Transforms {
    mat4 transforms[];
} transforms;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec4 v_normal;

void main() {
    mat4 model = transforms.transforms[gl_InstanceIndex];
    mat4 view_projection_model = uniforms.view_projection * model;
    gl_Position = view_projection_model * vec4(position, 1.0);
    vec4 VinView = normalize(-gl_Position);
    v_color = vec4(1.0, 1.0, 1.0, 1.0);

    mat3 mat_n = mat3(model);
    mat_n[0] /= dot(mat_n[0], mat_n[0]);
    mat_n[1] /= dot(mat_n[1], mat_n[1]);
    mat_n[2] /= dot(mat_n[2], mat_n[2]);

    v_normal = vec4(normalize(mat_n * normal), 0.0);
    vec4 normInView = (uniforms.view_projection * v_normal);
    v_normal.w = dot(normInView, VinView) * 0.5 + 0.5;
}

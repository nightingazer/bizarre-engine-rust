#version 450

layout(set = 0, binding = 0) uniform ViewProjectionUniforms {
    mat4 view;
    mat4 projection;
} uniforms;

vec3 gridPlane[4] = vec3[](
        vec3(1, 0, 1),
        vec3(-1, 0, 1),
        vec3(-1, 0, -1),
        vec3(1, 0, -1)
    );

layout(location = 0) out vec2 v_frag_position;
layout(location = 1) out float v_camera_distance;

void main() {
    vec4 position = vec4(gridPlane[gl_VertexIndex], 0.001);

    v_frag_position = (gridPlane[gl_VertexIndex].xz) * 1000;
    v_camera_distance = -(uniforms.view * position).z;

    gl_Position = uniforms.projection * uniforms.view * position;
}

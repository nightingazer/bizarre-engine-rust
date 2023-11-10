#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 1) uniform MVP_Data {
    mat4 view;
    mat4 projection;
} mvp_data;

layout(location = 0) out vec3 frag_pos;

void main() {
    gl_Position = mvp_data.projection * mvp_data.view * vec4(position, 1.0);
    frag_pos = vec3(mvp_data.view * vec4(position, 1.0));
}

#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 1) uniform MVP_Data {
    mat4 model;
    mat4 view;
    mat4 projection;
} mvp_data;

layout(location = 0) out vec3 frag_pos;

void main() {
    mat4 worldview = mvp_data.view * mvp_data.model;
    gl_Position = mvp_data.projection * worldview * vec4(position, 1.0);
    frag_pos = vec3(worldview * vec4(position, 1.0));
}

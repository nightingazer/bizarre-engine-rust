#version 460

layout(location = 0) in vec3 v_color;
layout(location = 1) in vec2 v_uv;

layout(set = 0, binding = 0) uniform sampler2D u_texture;

layout(location = 0) out vec4 f_color;

void main() {
    float aa = 1.0 / 16.0;
    vec4 tex_color = texture(u_texture, v_uv);
    float alpha = tex_color.a;
    float sdf = smoothstep(0.5 - aa, 0.5 + aa, alpha);
    f_color = vec4(v_color, sdf);
}

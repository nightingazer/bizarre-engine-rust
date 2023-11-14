#version 450

layout(location = 0) in vec2 v_frag_position;
layout(location = 1) in float v_camera_distance;

layout(location = 0) out vec4 fragColor;

void main() {
    vec2 line_width = vec2(0.02);

    vec4 uv_ddxy = vec4(dFdx(v_frag_position), dFdy(v_frag_position));
    vec2 uv_deriv = vec2(length(uv_ddxy.xz), length(uv_ddxy.yw));
    vec2 line_aa = uv_deriv * 1.5;
    vec2 draw_width = clamp(line_width, uv_deriv, vec2(0.5));

    vec2 line_uv = 1.0 - abs(fract(v_frag_position) * 2.0 - 1.0);
    vec2 line = smoothstep(draw_width + line_aa, draw_width - line_aa, line_uv);
    line *= clamp(line_width / draw_width, 0.0, 1.0);
    line = mix(line, line_width, clamp(uv_deriv * 2.0 - 1.0, 0.0, 1.0));

    float alpha = mix(line.x, 1.0, line.y);

    fragColor = vec4(1.0, 1.0, 1.0, alpha);
}

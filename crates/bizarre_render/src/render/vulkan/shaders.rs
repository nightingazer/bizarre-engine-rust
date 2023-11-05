pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 color;
            layout(location = 2) in vec3 normal;

            layout(set = 0, binding = 0) uniform MVP_Data {
                mat4 model;
                mat4 view;
                mat4 projection;
            } uniforms;

            layout(location = 0) out vec3 v_color;
            layout(location = 1) out vec3 v_normal;
            layout(location = 2) out vec3 v_frag_pos;

            void main() {
                mat4 worldview = uniforms.view * uniforms.model;
                gl_Position = uniforms.projection * worldview * vec4(position, 1.0);
                v_color = color;
                v_normal = mat3(uniforms.model) * normal;
                v_frag_pos = vec3(uniforms.model * vec4(position, 1.0));
            }
        "
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450
            layout(location = 0) in vec3 v_color;
            layout(location = 1) in vec3 v_normal;
            layout(location = 2) in vec3 v_frag_position;

            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 1) uniform Ambient_Data {
                vec3 ambient_color;
                float ambient_intensity;
            } ambient;

            layout(set = 0, binding = 2) uniform Directional_Data {
                vec3 position;
                vec3 color;
            } directional;

            void main() {
                vec3 ambient_color = ambient.ambient_color * ambient.ambient_intensity;
                vec3 light_direction = normalize(directional.position - v_frag_position);
                float directional_intensity = max(dot(v_normal, light_direction), 0.0);
                vec3 directional_color = directional_intensity * directional.color;
                vec3 combined_color = (directional_color + ambient_color) * v_color;
                f_color = vec4(combined_color, 1.0);
            }
        "
    }
}

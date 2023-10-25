pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 color;

            layout(location = 0) out vec3 v_color;

            void main() {
                gl_Position = vec4(position, 1.0);
                v_color = color;
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
            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(v_color, 1.0);
            }
        "
    }
}

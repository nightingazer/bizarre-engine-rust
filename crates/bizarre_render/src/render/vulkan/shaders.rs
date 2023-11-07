pub mod deferred_vert {
    use vulkano_shaders::shader;

    shader! {
        ty: "vertex",
        path: "assets/shaders/deferred.vert",
    }
}

pub mod deferred_frag {
    use vulkano_shaders::shader;

    shader! {
        ty: "fragment",
        path: "assets/shaders/deferred.frag",
    }
}

pub mod lighting_vert {
    use vulkano_shaders::shader;

    shader! {
        ty: "vertex",
        path: "assets/shaders/lighting.vert",
    }
}

pub mod lighting_frag {
    use vulkano_shaders::shader;

    shader! {
        ty: "fragment",
        path: "assets/shaders/lighting.frag",
    }
}

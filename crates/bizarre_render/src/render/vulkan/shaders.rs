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

pub mod ambient_vert {
    use vulkano_shaders::shader;

    shader! {
        ty: "vertex",
        path: "assets/shaders/ambient.vert",
    }
}

pub mod ambient_frag {
    use vulkano_shaders::shader;

    shader! {
        ty: "fragment",
        path: "assets/shaders/ambient.frag",
    }
}

pub mod directional_vert {
    use vulkano_shaders::shader;

    shader! {
        ty: "vertex",
        path: "assets/shaders/directional.vert",
    }
}

pub mod directional_frag {
    use vulkano_shaders::shader;

    shader! {
        ty: "fragment",
        path: "assets/shaders/directional.frag",
    }
}

pub mod skybox_vert {
    use vulkano_shaders::shader;

    shader! {
        ty: "vertex",
        path: "assets/shaders/skybox.vert"
    }
}

pub mod skybox_frag {
    use vulkano_shaders::shader;

    shader! {
        ty: "fragment",
        path: "assets/shaders/skybox.frag"
    }
}

pub mod floor_vert {
    use vulkano_shaders::shader;

    shader! {
        ty: "vertex",
        path: "assets/shaders/floor.vert"
    }
}

pub mod floor_frag {
    use vulkano_shaders::shader;

    shader! {
        ty: "fragment",
        path: "assets/shaders/floor.frag"
    }
}

pub mod text_vert {
    use vulkano_shaders::shader;

    shader! {
        ty: "vertex",
        path: "assets/shaders/text.vert"
    }
}

pub mod text_frag {
    use vulkano_shaders::shader;

    shader! {
        ty: "fragment",
        path: "assets/shaders/text.frag"
    }
}

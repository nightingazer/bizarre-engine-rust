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

pub trait Updatable {
    fn update(&mut self, delta_time: f32) -> anyhow::Result<()>;
}

use std::time::Duration;

#[derive(Default, Clone)]
pub struct DeltaTime(pub Duration);

#[derive(Default, Clone)]
pub struct RunningTime(pub Duration);

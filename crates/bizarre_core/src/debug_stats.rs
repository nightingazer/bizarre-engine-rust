#[derive(Debug, Default)]
pub struct DebugStats {
    /// Last frame work time in milliseconds
    pub last_frame_work_time_ms: f64,
    /// Last frame idle time in milliseconds
    pub last_frame_idle_time_ms: f64,
    /// Last frame total time in milliseconds
    pub last_frame_total_time_ms: f64,
}

use self::pass::{MaterialPass, MaterialPassType};

pub mod binding;
pub mod pass;
pub mod pipeline_features;

pub struct Material {
    pub passes: [Option<MaterialPass>; std::mem::variant_count::<MaterialPassType>()],
}

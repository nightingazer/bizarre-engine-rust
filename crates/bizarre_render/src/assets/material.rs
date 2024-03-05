use self::pass::{MaterialPassType, MaterialPipeline};

pub mod binding;
// pub mod builtin_materials;
pub mod pass;
pub mod pipeline_features;

pub const MATERIAL_PASS_COUNT: usize = std::mem::variant_count::<MaterialPassType>();

pub struct Material {
    pub passes: [Option<Box<[MaterialPipeline]>>; MATERIAL_PASS_COUNT],
}

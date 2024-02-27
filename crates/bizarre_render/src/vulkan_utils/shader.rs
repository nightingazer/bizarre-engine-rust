use std::{
    fs::File,
    io::{Cursor, SeekFrom, Write},
    path::Path,
};

use anyhow::{bail, Result};
use ash::vk;
use bizarre_logger::core_info;

#[derive(Clone, Copy, Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl From<ShaderStage> for shaderc::ShaderKind {
    fn from(shader_type: ShaderStage) -> Self {
        match shader_type {
            ShaderStage::Vertex => shaderc::ShaderKind::Vertex,
            ShaderStage::Fragment => shaderc::ShaderKind::Fragment,
            ShaderStage::Compute => shaderc::ShaderKind::Compute,
        }
    }
}

impl From<ShaderStage> for vk::ShaderStageFlags {
    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
        }
    }
}

pub fn load_shader(path: &Path, shader_type: ShaderStage) -> Result<Vec<u32>> {
    if !path.is_file() {
        bail!(
            "Could not open shader file '{}': Not a file",
            path.to_string_lossy()
        );
    }

    let filename = path.file_name().unwrap().to_str().unwrap();
    let cached_path = Path::new("cache/shaders/vulkan").join(format!("{filename}.spv"));

    let invalid_cache = if cached_path.is_file() {
        let source_metadata = std::fs::metadata(path)?;
        let cached_metadata = std::fs::metadata(&cached_path)?;

        source_metadata.modified()? > cached_metadata.modified()?
    } else {
        true
    };

    let spv = if invalid_cache {
        core_info!("Compiling shader '{}'", path.to_str().unwrap());

        let mut file = File::open(path)?;
        let artifact = compile_shader(&mut file, shader_type, filename)?;

        validate_spv(&mut Cursor::new(&artifact.as_binary_u8()))?;

        let prefix = cached_path.parent().unwrap();
        if !prefix.is_dir() {
            std::fs::create_dir_all(prefix)?;
        }

        let mut cached_file = File::create(cached_path)?;
        cached_file.write_all(artifact.as_binary_u8())?;
        artifact.as_binary().to_vec()
    } else {
        let mut file = File::open(cached_path)?;
        validate_spv(&mut file)?;
        read_spv(&mut file)?
    };

    Ok(spv)
}

pub fn compile_shader<S>(
    stream: &mut S,
    shader_type: ShaderStage,
    filename: &str,
) -> Result<shaderc::CompilationArtifact>
where
    S: std::io::Read + std::io::Seek,
{
    let source_len = stream.seek(SeekFrom::End(0))? as usize;
    stream.rewind()?;

    let mut source = String::with_capacity(source_len);
    stream.read_to_string(&mut source)?;

    let compiler = shaderc::Compiler::new().unwrap();
    let options = shaderc::CompileOptions::new().unwrap();

    let result = compiler.compile_into_spirv(
        &source,
        shaderc::ShaderKind::from(shader_type),
        filename,
        "main",
        Some(&options),
    )?;

    Ok(result)
}

pub fn validate_spv<S>(stream: &mut S) -> Result<()>
where
    S: std::io::Seek + std::io::Read,
{
    let buf_len = stream.seek(SeekFrom::End(0))? as usize;

    if buf_len % 4 != 0 {
        bail!("Invalid SPIR-V file: Length is not a multiple of 4");
    }
    stream.rewind()?;

    let mut magic_number = [0u8; 4];
    stream.read_exact(&mut magic_number)?;

    if magic_number != [0x03, 0x02, 0x23, 0x07] {
        bail!("Invalid SPIR-V file: Invalid magic number");
    }

    Ok(())
}

pub fn read_spv<S>(stream: &mut S) -> Result<Vec<u32>>
where
    S: std::io::Seek + std::io::Read,
{
    let buf_len = stream.seek(SeekFrom::End(0))? as usize;
    stream.rewind()?;

    let mut buf = vec![0u32; buf_len / 4];
    unsafe {
        stream.read_exact(std::slice::from_raw_parts_mut(
            buf.as_mut_ptr().cast::<u8>(),
            buf_len,
        ))?;
    }

    Ok(buf)
}

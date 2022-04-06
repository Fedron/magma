extern crate spirv_reflect;

use ash::vk;
use spirv_reflect::{types::ReflectShaderStageFlags, *};
use std::{fmt::Debug, rc::Rc};

use crate::{core::device::LogicalDevice, VulkanError};

#[derive(thiserror::Error, Debug)]
pub enum ShaderBuilderError {
    #[error("The shader file could not be found")]
    FileNotFound,
    #[error("Failed to read the contents of the file")]
    CantRead,
    #[error("Failed to parse the shader spirv")]
    CantParseSpv(String),
    #[error("Can't create a shader as its shader stage is not supported")]
    UnsupportedShaderStage,
    #[error("Failed to create a Vulkan shader module")]
    BuildFail(VulkanError),
}

#[derive(Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl Into<vk::ShaderStageFlags> for ShaderStage {
    fn into(self) -> vk::ShaderStageFlags {
        match self {
            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
        }
    }
}

pub struct ShaderBuilder {
    file_path: &'static str,
}

impl ShaderBuilder {
    pub fn new(file_path: &'static str) -> ShaderBuilder {
        ShaderBuilder { file_path }
    }

    pub fn build(self, device: Rc<LogicalDevice>) -> Result<Shader, ShaderBuilderError> {
        use std::fs::File;
        use std::path::Path;

        let mut path = Path::new(self.file_path).to_path_buf();
        path.set_extension(format!(
            "{}.spv",
            path.extension().unwrap().to_str().unwrap()
        ));
        let shader_code = ash::util::read_spv(
            &mut File::open(path).map_err(|_| ShaderBuilderError::FileNotFound)?,
        )
        .map_err(|_| ShaderBuilderError::CantRead)?;

        let shader_module = ShaderModule::load_u32_data(&shader_code)
            .map_err(|err| ShaderBuilderError::CantParseSpv(err.to_string()))?;

        let shader_stage = shader_module.get_shader_stage();
        let shader_stage = if shader_stage.contains(ReflectShaderStageFlags::VERTEX) {
            ShaderStage::Vertex
        } else if shader_stage.contains(ReflectShaderStageFlags::FRAGMENT) {
            ShaderStage::Fragment
        } else if shader_stage.contains(ReflectShaderStageFlags::COMPUTE) {
            ShaderStage::Compute
        } else {
            return Err(ShaderBuilderError::UnsupportedShaderStage);
        };

        let create_info = vk::ShaderModuleCreateInfo::builder().code(&shader_code);
        let handle = unsafe {
            device
                .vk_handle()
                .create_shader_module(&create_info, None)
                .map_err(|err| ShaderBuilderError::BuildFail(err.into()))?
        };

        Ok(Shader {
            entry_point: shader_module.get_entry_point_name(),
            stage: shader_stage,
            handle,
            device,
        })
    }
}

pub struct Shader {
    entry_point: String,
    stage: ShaderStage,

    handle: vk::ShaderModule,
    device: Rc<LogicalDevice>,
}

impl Shader {
    pub fn builder(file_path: &'static str) -> ShaderBuilder {
        ShaderBuilder::new(file_path)
    }
}

impl Debug for Shader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shader")
            .field("entry_point", &self.entry_point)
            .field("stage", &self.stage)
            .field("handle", &self.handle)
            .finish()
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_handle()
                .destroy_shader_module(self.handle, None);
        };
    }
}

extern crate spirv_reflect;

use ash::vk;
use bitflags::bitflags;
use spirv_reflect::ShaderModule as SpirvShader;
use std::{fmt::Debug, rc::Rc};

use crate::{core::device::LogicalDevice, VulkanError};

/// Possible errors that could be returned by a [Shader]
#[derive(thiserror::Error, Debug)]
pub enum ShaderError {
    #[error("The shader file could not be found")]
    FileNotFound,
    #[error("Failed to read the contents of the file")]
    CantRead,
    #[error("Failed to parse the shader spirv")]
    CantParseSpv(String),
    #[error("Can't create a shader as its shader stage is not supported")]
    UnsupportedShaderStage,
    #[error("Failed to create a Vulkan shader module {0}")]
    BuildFail(VulkanError),
}

bitflags! {
    pub struct ShaderStageFlags: u32 {
        const VERTEX = 0b1;
        const FRAGMENT = 0b10000;
        const COMPUTE = 0b100000;
        const ALL_GRAPHICS = 0b11111;
    }
}

impl Into<vk::ShaderStageFlags> for ShaderStageFlags {
    fn into(self) -> vk::ShaderStageFlags {
        vk::ShaderStageFlags::from_raw(self.bits())
    }
}

pub struct Shader {
    pub file_path: &'static str,
    pub flags: ShaderStageFlags,
    pub entry_point: String,

    code: Vec<u32>,
    reflect: SpirvShader,
}

impl Shader {
    pub fn new(
        file_path: &'static str,
        stage_flags: ShaderStageFlags,
    ) -> Result<Shader, ShaderError> {
        use std::fs::File;
        use std::path::Path;

        let mut path = Path::new(file_path).to_path_buf();
        path.set_extension(format!(
            "{}.spv",
            path.extension().unwrap().to_str().unwrap()
        ));

        let code =
            ash::util::read_spv(&mut File::open(path).map_err(|_| ShaderError::FileNotFound)?)
                .map_err(|_| ShaderError::CantRead)?;
        let reflect = SpirvShader::load_u32_data(&code)
            .map_err(|err| ShaderError::CantParseSpv(err.to_string()))?;

        let entry_point = reflect.get_entry_point_name();

        Ok(Shader {
            file_path,
            flags: stage_flags,
            entry_point,

            code,
            reflect,
        })
    }
}

/// Wraps a Vulkan shader module
#[derive(Clone)]
pub struct ShaderModule {
    /// Entry point into the shader code
    entry_point: String,
    /// Shader stages the module belongs to
    stage_flags: ShaderStageFlags,

    /// Opaque handle to the Vulkan shader module
    handle: vk::ShaderModule,
    /// Logical device the shader belongs to
    device: Rc<LogicalDevice>,
}

impl ShaderModule {
    pub fn new(shader: &Shader, device: Rc<LogicalDevice>) -> Result<ShaderModule, ShaderError> {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(&shader.code);
        let handle = unsafe {
            device
                .vk_handle()
                .create_shader_module(&create_info, None)
                .map_err(|err| ShaderError::BuildFail(err.into()))?
        };

        Ok(ShaderModule {
            entry_point: shader.entry_point.clone(),
            stage_flags: shader.flags,

            handle,
            device,
        })
    }
}

impl ShaderModule {
    /// Returns the handle to the Vulkan shader module
    pub(crate) fn vk_handle(&self) -> vk::ShaderModule {
        self.handle
    }
}

impl Debug for ShaderModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shader")
            .field("entry_point", &self.entry_point)
            .field("stage", &self.stage_flags)
            .field("handle", &self.handle)
            .finish()
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_handle()
                .destroy_shader_module(self.handle, None);
        };
    }
}

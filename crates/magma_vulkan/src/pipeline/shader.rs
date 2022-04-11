extern crate spirv_reflect;

use ash::vk;
use bitflags::bitflags;
use spirv_reflect::ShaderModule as SpirvShader;
use std::{fmt::Debug, rc::Rc};

use crate::{core::device::LogicalDevice, VulkanError};

use super::vertex::{Vertex, VertexAttributeDescription, VertexBindingDescription};

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
    #[error("Invalid shader definition: {0}")]
    InvalidDefinition(&'static str),
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

    pub should_define_vertex: bool,
    pub vertex_attribute_descriptions: Option<Vec<VertexAttributeDescription>>,
    pub vertex_binding_descriptions: Option<Vec<VertexBindingDescription>>,

    code: Vec<u32>,
    reflect: SpirvShader,
}

impl Shader {
    pub fn new(file_path: &'static str) -> Result<Shader, ShaderError> {
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
        let shader_stage = ShaderStageFlags::from_bits(reflect.get_shader_stage().bits())
            .ok_or(ShaderError::InvalidDefinition("Invalid shader stage"))?;

        let mut should_define_vertex = false;
        if shader_stage.contains(ShaderStageFlags::VERTEX) {
            let input_variables = reflect.enumerate_input_variables(Some(&entry_point)).map_err(|err| ShaderError::CantParseSpv(err.to_string()))?;
            if input_variables.iter().any(|var| !var.decoration_flags.contains(spirv_reflect::types::variable::ReflectDecorationFlags::BUILT_IN)) {
                should_define_vertex = true;
            }
        }

        Ok(Shader {
            file_path,
            flags: shader_stage,
            entry_point,

            should_define_vertex,
            vertex_attribute_descriptions: None,
            vertex_binding_descriptions: None,

            code,
            reflect,
        })
    }

    pub fn build(&self, device: Rc<LogicalDevice>) -> Result<ShaderModule, ShaderError> {
        if self.should_define_vertex && (self.vertex_attribute_descriptions.is_none() || self.vertex_binding_descriptions.is_none()) {
            return Err(ShaderError::InvalidDefinition("The spirv defines vertex attributes, but you haven't linked any, use .with_vertex<V>() to link vertex attributes"));
        }

        let create_info = vk::ShaderModuleCreateInfo::builder().code(&self.code);
        let handle = unsafe {
            device
                .vk_handle()
                .create_shader_module(&create_info, None)
                .map_err(|err| ShaderError::BuildFail(err.into()))?
        };

        Ok(ShaderModule { handle, device })
    }
}

impl Shader {
    pub fn with_vertex<V>(mut self) -> Shader
    where
        V: Vertex,
    {
        self.vertex_attribute_descriptions = Some(V::get_attribute_descriptions());
        self.vertex_binding_descriptions = Some(V::get_binding_descriptions());
        self
    }
}

/// Wraps a Vulkan shader module
#[derive(Clone)]
pub struct ShaderModule {
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

        Ok(ShaderModule { handle, device })
    }
}

impl ShaderModule {
    /// Returns the handle to the Vulkan shader module
    pub(crate) fn vk_handle(&self) -> vk::ShaderModule {
        self.handle
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

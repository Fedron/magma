extern crate spirv_reflect;

use ash::vk;
use bitflags::bitflags;
use spirv_reflect::{types::ReflectFormat, ShaderModule as SpirvShader};
use std::{ffi::CString, fmt::Debug, rc::Rc};

use crate::{core::device::LogicalDevice, VulkanError};

use super::vertex::{Vertex, VertexAttributeDescription};

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
    InvalidDefinition(String),
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
    pub entry_point: CString,

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
        let shader_stage = ShaderStageFlags::from_bits(reflect.get_shader_stage().bits()).ok_or(
            ShaderError::InvalidDefinition("Invalid shader stage".to_string()),
        )?;

        Ok(Shader {
            file_path,
            flags: shader_stage,
            entry_point: CString::new(entry_point).expect("Failed to create CString"),

            code,
            reflect,
        })
    }

    pub fn build(&self, device: Rc<LogicalDevice>) -> Result<ShaderModule, ShaderError> {
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
    pub fn check_vertex_input<V>(&self) -> Result<(), ShaderError>
    where
        V: Vertex,
    {
        let input_variables = self
            .reflect
            .enumerate_input_variables(Some(
                &self
                    .entry_point
                    .to_str()
                    .expect("Failed to cast CString to str"),
            ))
            .map_err(|err| ShaderError::CantParseSpv(err.into()))?;
        let mut vertex_attribute_descriptions: Vec<VertexAttributeDescription> = Vec::new();
        let mut offset = 0;
        for input_variable in input_variables.iter() {
            if input_variable.storage_class == spirv_reflect::types::ReflectStorageClass::Input
                && input_variable.decoration_flags.is_empty()
            {
                let format = match input_variable.format {
                    ReflectFormat::R32_SINT => vk::Format::R32_SINT,

                    ReflectFormat::R32G32_SINT => vk::Format::R32G32_SINT,
                    ReflectFormat::R32G32B32_SINT => vk::Format::R32G32B32_SINT,
                    ReflectFormat::R32G32B32A32_SINT => vk::Format::R32G32B32A32_SINT,
                    ReflectFormat::R32_SFLOAT => vk::Format::R32_SFLOAT,
                    ReflectFormat::R32G32_SFLOAT => vk::Format::R32G32_SFLOAT,
                    ReflectFormat::R32G32B32_SFLOAT => vk::Format::R32G32B32_SFLOAT,
                    ReflectFormat::R32G32B32A32_SFLOAT => vk::Format::R32G32B32A32_SFLOAT,
                    _ => {
                        return Err(ShaderError::InvalidDefinition(format!(
                            "Input variable `{}` has an unsupported type `{:#?}`",
                            input_variable.name, input_variable.format
                        )))
                    }
                };

                vertex_attribute_descriptions.push(VertexAttributeDescription {
                    location: input_variable.location,
                    binding: 0,
                    format,
                    offset,
                });

                offset += input_variable.numeric.vector.component_count * 4;
            }
        }

        if vertex_attribute_descriptions.len() != V::get_attribute_descriptions().len() {
            return Err(
                ShaderError::InvalidDefinition(
                    format!("Shader contains {} input variable, but your Vertex struct only has {} fields", vertex_attribute_descriptions.len(), V::get_attribute_descriptions().len())
                ));
        }

        for vertex_attribute in vertex_attribute_descriptions.iter() {
            if let Some(user_vertex_attribute) =
                V::get_attribute_descriptions()
                .iter()
                .find(|attr| attr.location == vertex_attribute.location)
            {
                if vertex_attribute.ne(user_vertex_attribute) {
                    return Err(
                            ShaderError::InvalidDefinition(
                                format!("Shader input variable at location {} does not match the field in the Vertex struct you provided", vertex_attribute.location)
                        ));
                }
            } else {
                return Err(
                        ShaderError::InvalidDefinition(
                            format!("Shader contains input variable with definition: {:#?}\nbut the Vertex struct you provided doesn't contain a matching field", vertex_attribute)
                    ));
            }
        }

        Ok(())
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

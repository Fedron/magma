extern crate spirv_reflect;
use std::{fmt::Debug, rc::Rc};

use ash::vk;

use crate::prelude::LogicalDevice;

use self::builder::ShaderBuilder;

pub mod builder;

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

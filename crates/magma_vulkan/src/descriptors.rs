use ash::vk;
use std::rc::Rc;

use crate::core::device::{LogicalDevice, LogicalDeviceError};
use crate::pipeline::shader::ShaderStageFlags;

pub mod allocator;
pub mod cache;

use self::allocator::{DescriptorAllocator, DescriptorAllocatorError, DescriptorType};
use self::cache::{DescriptorLayoutCache, DescriptorCacheError};

#[derive(thiserror::Error, Debug)]
pub enum DescriptorBuilderError {
    #[error(transparent)]
    AllocatorError(#[from] DescriptorAllocatorError),
    #[error(transparent)]
    CacheError(#[from] DescriptorCacheError),
    #[error(transparent)]
    DeviceError(#[from] LogicalDeviceError),
}

pub struct DescriptorBuilder<'a> {
    writes: Vec<vk::WriteDescriptorSet>,
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
    cache: &'a mut DescriptorLayoutCache,
    allocator: &'a mut DescriptorAllocator,
}

impl<'a> DescriptorBuilder<'a> {
    pub fn new(
        cache: &'a mut DescriptorLayoutCache,
        allocator: &'a mut DescriptorAllocator,
    ) -> DescriptorBuilder<'a> {
        DescriptorBuilder {
            writes: Vec::new(),
            bindings: Vec::new(),
            cache,
            allocator,
        }
    }

    pub fn bind_buffer(
        mut self,
        binding: u32,
        buffer_info: vk::DescriptorBufferInfo,
        ty: DescriptorType,
        stage_flags: ShaderStageFlags,
    ) -> DescriptorBuilder<'a> {
        self.bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .descriptor_count(1)
                .descriptor_type(ty.into())
                .stage_flags(stage_flags.into())
                .binding(binding)
                .build(),
        );

        let buffer_infos = [buffer_info];
        self.writes.push(
            vk::WriteDescriptorSet::builder()
                .descriptor_type(ty.into())
                .buffer_info(&buffer_infos)
                .dst_binding(binding)
                .build(),
        );

        self
    }

    pub fn build(mut self, device: Rc<LogicalDevice>) -> Result<vk::DescriptorSet, DescriptorBuilderError> {
        let layout = self.cache.create_descriptor_layout(
            vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&self.bindings)
                .build(),
        )?;

        let set = self.allocator.allocate(layout)?;
        for write in self.writes.iter_mut() {
            write.dst_set = set;
        }

        unsafe {
            device.vk_handle().update_descriptor_sets(&self.writes, &[]);
        };

        Ok(set)
    }
}

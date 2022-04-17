use ash::vk;
use std::rc::Rc;

use crate::{
    core::device::{LogicalDevice, LogicalDeviceError},
    pipeline::shader::ShaderStageFlags,
    VulkanError,
};

#[derive(thiserror::Error, Debug)]
pub enum DescriptorError {
    #[error(transparent)]
    CantCreateLayout(VulkanError),
    #[error(transparent)]
    CantCreatePool(VulkanError),
    #[error(transparent)]
    DeviceError(LogicalDeviceError),
}

#[derive(Clone, Copy, Debug)]
pub enum DescriptorType {
    Sampler,
    CombinedImageSampler,
    SampledImage,
    StorageImage,
    UniformTexelBuffer,
    StorageTexelBuffer,
    UniformBuffer,
    StorageBuffer,
    UniformBufferDynamic,
    StorageBufferDynamic,
    InputAttachment,
}

impl Into<vk::DescriptorType> for DescriptorType {
    fn into(self) -> vk::DescriptorType {
        match self {
            DescriptorType::Sampler => vk::DescriptorType::SAMPLER,
            DescriptorType::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            DescriptorType::SampledImage => vk::DescriptorType::SAMPLED_IMAGE,
            DescriptorType::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
            DescriptorType::UniformTexelBuffer => vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
            DescriptorType::StorageTexelBuffer => vk::DescriptorType::STORAGE_TEXEL_BUFFER,
            DescriptorType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorType::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorType::UniformBufferDynamic => vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            DescriptorType::StorageBufferDynamic => vk::DescriptorType::STORAGE_BUFFER_DYNAMIC,
            DescriptorType::InputAttachment => vk::DescriptorType::INPUT_ATTACHMENT,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DescriptorSetBinding {
    pub binding: u32,
    pub ty: DescriptorType,
    pub count: u32,
    pub shader_stage_flags: ShaderStageFlags,
}

impl Into<vk::DescriptorSetLayoutBinding> for DescriptorSetBinding {
    fn into(self) -> vk::DescriptorSetLayoutBinding {
        vk::DescriptorSetLayoutBinding::builder()
            .binding(self.binding)
            .descriptor_type(self.ty.into())
            .descriptor_count(self.count)
            .stage_flags(self.shader_stage_flags.into())
            .build()
    }
}

pub struct DescriptorSetLayout {
    bindings: Vec<DescriptorSetBinding>,
    handle: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(
        device: Rc<LogicalDevice>,
        bindings: &[DescriptorSetBinding],
    ) -> Result<DescriptorSetLayout, DescriptorError> {
        let vk_bindings: Vec<vk::DescriptorSetLayoutBinding> =
            bindings.iter().map(|&binding| binding.into()).collect();

        let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&vk_bindings)
            .build();

        let handle = unsafe {
            device
                .vk_handle()
                .create_descriptor_set_layout(&create_info, None)
                .map_err(|err| DescriptorError::CantCreateLayout(err.into()))?
        };

        Ok(DescriptorSetLayout {
            bindings: bindings.to_vec(),
            handle,
        })
    }
}

impl DescriptorSetLayout {
    pub(crate) fn vk_handle(&self) -> vk::DescriptorSetLayout {
        self.handle
    }
}

pub struct DescriptorPoolBuilder {
    pool_sizes: Vec<vk::DescriptorPoolSize>,
    max_sets: u32,
}

impl DescriptorPoolBuilder {
    pub fn new() -> DescriptorPoolBuilder {
        DescriptorPoolBuilder {
            pool_sizes: Vec::new(),
            max_sets: 1000,
        }
    }

    pub fn add_pool_size(
        mut self,
        descriptor_type: DescriptorType,
        count: u32,
    ) -> DescriptorPoolBuilder {
        self.pool_sizes.push(vk::DescriptorPoolSize {
            ty: descriptor_type.into(),
            descriptor_count: count,
        });
        self
    }

    pub fn max_sets(mut self, count: u32) -> DescriptorPoolBuilder {
        self.max_sets = count;
        self
    }

    pub fn build(self, device: Rc<LogicalDevice>) -> Result<DescriptorPool, DescriptorError> {
        DescriptorPool::new(device, self.max_sets, &self.pool_sizes)
    }
}

pub struct DescriptorPool {
    handle: vk::DescriptorPool,
    device: Rc<LogicalDevice>,
}

impl DescriptorPool {
    pub fn builder() -> DescriptorPoolBuilder {
        DescriptorPoolBuilder::new()
    }

    pub(crate) fn new(
        device: Rc<LogicalDevice>,
        max_sets: u32,
        pool_sizes: &[vk::DescriptorPoolSize],
    ) -> Result<DescriptorPool, DescriptorError> {
        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .pool_sizes(pool_sizes)
            .max_sets(max_sets);

        let handle = unsafe {
            device
                .vk_handle()
                .create_descriptor_pool(&create_info, None)
                .map_err(|err| DescriptorError::CantCreatePool(err.into()))?
        };

        Ok(DescriptorPool { handle, device })
    }
}

impl DescriptorPool {
    pub fn allocate_descriptor_set(
        &self,
        set_layout: &DescriptorSetLayout,
    ) -> Result<vk::DescriptorSet, DescriptorError> {
        let set_layouts: [vk::DescriptorSetLayout; 1] = [set_layout.vk_handle()];
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.handle)
            .set_layouts(&set_layouts);

        Ok(unsafe {
            self.device
                .vk_handle()
                .allocate_descriptor_sets(&allocate_info)
                .map_err(|err| DescriptorError::DeviceError(LogicalDeviceError::Other(err.into())))?
                .first()
                .expect("Something went very wrong: Created a descriptor set but failed to get it")
                .clone()
        })
    }

    pub fn free_descriptor_sets(
        &self,
        descriptors: &[vk::DescriptorSet],
    ) -> Result<(), DescriptorError> {
        unsafe {
            self.device
                .vk_handle()
                .free_descriptor_sets(self.handle, descriptors)
                .map_err(|err| {
                    DescriptorError::DeviceError(LogicalDeviceError::Other(err.into()))
                })?
        };

        Ok(())
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_handle()
                .destroy_descriptor_pool(self.handle, None);
        };
    }
}

pub struct DescriptorWriter {
    layout: Rc<DescriptorSetLayout>,
    pool: Rc<DescriptorPool>,
    writes: Vec<vk::WriteDescriptorSet>,
}

impl DescriptorWriter {
    pub fn new(layout: Rc<DescriptorSetLayout>, pool: Rc<DescriptorPool>) -> DescriptorWriter {
        DescriptorWriter {
            layout,
            pool,
            writes: Vec::new(),
        }
    }
}

impl DescriptorWriter {
    pub fn write_buffer(
        mut self,
        binding: u32,
        buffer_info: vk::DescriptorBufferInfo,
    ) -> DescriptorWriter {
        if !self.layout.bindings.iter().any(|b| b.binding == binding) {
            log::warn!("Tried to write a buffer to a descriptor binding that doesn't exist");
            return self;
        }

        let binding = self
            .layout
            .bindings
            .iter()
            .find(|b| b.binding == binding)
            .unwrap();
        self.writes.push(
            vk::WriteDescriptorSet::builder()
                .descriptor_type(binding.ty.into())
                .dst_binding(binding.binding)
                .buffer_info(&[buffer_info])
                .build(),
        );
        self
    }

    pub fn write(mut self) -> Result<vk::DescriptorSet, DescriptorError> {
        let set = self.pool.allocate_descriptor_set(self.layout.as_ref())?;

        for write in self.writes.iter_mut() {
            write.dst_set = set;
        }

        unsafe {
            self.pool
                .device
                .vk_handle()
                .update_descriptor_sets(&self.writes, &[]);
        };

        Ok(set)
    }
}

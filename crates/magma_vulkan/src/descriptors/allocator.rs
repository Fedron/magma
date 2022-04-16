use ash::vk;
use std::{collections::VecDeque, rc::Rc};

use crate::{
    core::device::{LogicalDevice, LogicalDeviceError},
    VulkanError,
};

#[derive(thiserror::Error, Debug)]
pub enum DescriptorAllocatorError {
    #[error(transparent)]
    DeviceError(#[from] LogicalDeviceError),
}

#[derive(Clone, Copy)]
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

impl From<spirv_reflect::types::descriptor::ReflectDescriptorType> for DescriptorType {
    fn from(ty: spirv_reflect::types::descriptor::ReflectDescriptorType) -> Self {
        use spirv_reflect::types::descriptor::ReflectDescriptorType;
        match ty {
            ReflectDescriptorType::Undefined => DescriptorType::Sampler,
            ReflectDescriptorType::Sampler => DescriptorType::Sampler,
            ReflectDescriptorType::CombinedImageSampler => DescriptorType::CombinedImageSampler,
            ReflectDescriptorType::SampledImage => DescriptorType::SampledImage,
            ReflectDescriptorType::StorageImage => DescriptorType::StorageImage,
            ReflectDescriptorType::UniformTexelBuffer => DescriptorType::UniformTexelBuffer,
            ReflectDescriptorType::StorageTexelBuffer => DescriptorType::StorageTexelBuffer,
            ReflectDescriptorType::UniformBuffer => DescriptorType::UniformBuffer,
            ReflectDescriptorType::StorageBuffer => DescriptorType::StorageBuffer,
            ReflectDescriptorType::UniformBufferDynamic => DescriptorType::UniformBufferDynamic,
            ReflectDescriptorType::StorageBufferDynamic => DescriptorType::StorageBufferDynamic,
            ReflectDescriptorType::InputAttachment => DescriptorType::InputAttachment,
            ReflectDescriptorType::AccelerationStructureNV => panic!("Unsupported descriptor type")
        }
    }
}

struct PoolSizeMultiplier {
    ty: DescriptorType,
    multiplier: f32,
}

impl PoolSizeMultiplier {
    pub const MULTIPLIERS: [PoolSizeMultiplier; 11] = [
        PoolSizeMultiplier {
            ty: DescriptorType::Sampler,
            multiplier: 0.5,
        },
        PoolSizeMultiplier {
            ty: DescriptorType::CombinedImageSampler,
            multiplier: 4.0,
        },
        PoolSizeMultiplier {
            ty: DescriptorType::SampledImage,
            multiplier: 4.0,
        },
        PoolSizeMultiplier {
            ty: DescriptorType::StorageImage,
            multiplier: 1.0,
        },
        PoolSizeMultiplier {
            ty: DescriptorType::UniformTexelBuffer,
            multiplier: 1.0,
        },
        PoolSizeMultiplier {
            ty: DescriptorType::StorageTexelBuffer,
            multiplier: 1.0,
        },
        PoolSizeMultiplier {
            ty: DescriptorType::UniformBuffer,
            multiplier: 2.0,
        },
        PoolSizeMultiplier {
            ty: DescriptorType::StorageBuffer,
            multiplier: 2.0,
        },
        PoolSizeMultiplier {
            ty: DescriptorType::UniformBufferDynamic,
            multiplier: 1.0,
        },
        PoolSizeMultiplier {
            ty: DescriptorType::StorageBufferDynamic,
            multiplier: 1.0,
        },
        PoolSizeMultiplier {
            ty: DescriptorType::InputAttachment,
            multiplier: 1.0,
        },
    ];
}

pub struct DescriptorAllocator {
    current_pool: vk::DescriptorPool,
    used_pools: Vec<vk::DescriptorPool>,
    free_pools: VecDeque<vk::DescriptorPool>,
    device: Rc<LogicalDevice>,
}

impl DescriptorAllocator {
    pub fn new(device: Rc<LogicalDevice>) -> DescriptorAllocator {
        DescriptorAllocator {
            current_pool: vk::DescriptorPool::null(),
            used_pools: Vec::new(),
            free_pools: VecDeque::new(),
            device,
        }
    }
}

impl DescriptorAllocator {
    pub(crate) fn create_pool(
        &mut self,
        count: u32,
    ) -> Result<vk::DescriptorPool, DescriptorAllocatorError> {
        let mut sizes: Vec<vk::DescriptorPoolSize> =
            Vec::with_capacity(PoolSizeMultiplier::MULTIPLIERS.len());
        for size in PoolSizeMultiplier::MULTIPLIERS.iter() {
            sizes.push(vk::DescriptorPoolSize {
                ty: size.ty.clone().into(),
                descriptor_count: (count as f32 * size.multiplier) as u32,
            });
        }

        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .max_sets(count)
            .pool_sizes(&sizes);

        let handle = unsafe {
            self.device
                .vk_handle()
                .create_descriptor_pool(&create_info, None)
                .map_err(|err| {
                    DescriptorAllocatorError::DeviceError(LogicalDeviceError::Other(err.into()))
                })?
        };
        Ok(handle)
    }

    pub(crate) fn get_pool(&mut self) -> Result<vk::DescriptorPool, DescriptorAllocatorError> {
        if self.free_pools.len() > 0 {
            Ok(self.free_pools.pop_back().unwrap())
        } else {
            self.create_pool(1000)
        }
    }

    pub fn allocate(
        &mut self,
        layout: vk::DescriptorSetLayout,
    ) -> Result<vk::DescriptorSet, DescriptorAllocatorError> {
        if vk::Handle::as_raw(self.current_pool) == 0 {
            self.current_pool = self.get_pool()?;
            self.used_pools.push(self.current_pool);
        }

        let set_layouts = [layout];
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&set_layouts)
            .descriptor_pool(self.current_pool);

        match unsafe {
            self.device
                .vk_handle()
                .allocate_descriptor_sets(&allocate_info)
        } {
            Ok(handles) => Ok(handles.first().unwrap().clone()),
            Err(err) => {
                let vk_error: VulkanError = err.into();
                match vk_error {
                    VulkanError::OutOfPoolMemory => {
                        self.current_pool = self.get_pool()?;
                        self.used_pools.push(self.current_pool);

                        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
                            .set_layouts(&set_layouts)
                            .descriptor_pool(self.current_pool);
                        let handles = unsafe {
                            self.device
                                .vk_handle()
                                .allocate_descriptor_sets(&allocate_info)
                                .map_err(|err| {
                                    DescriptorAllocatorError::DeviceError(
                                        LogicalDeviceError::Other(err.into()),
                                    )
                                })?
                        };

                        Ok(handles.first().unwrap().clone())
                    }
                    _ => Err(DescriptorAllocatorError::DeviceError(
                        LogicalDeviceError::Other(vk_error),
                    )),
                }
            }
        }
    }
    
    pub(crate) fn reset_pools(&mut self) {
        for &pool in self.used_pools.iter() {
            unsafe {
                self.device.vk_handle().reset_descriptor_pool(pool, vk::DescriptorPoolResetFlags::empty()).unwrap();
            };
            self.free_pools.push_back(pool);
        }

        self.used_pools.clear();
        self.current_pool = vk::DescriptorPool::null();
    }
}

impl Drop for DescriptorAllocator {
    fn drop(&mut self) {
        unsafe {
            for &pool in self.used_pools.iter() {
                self.device.vk_handle().destroy_descriptor_pool(pool, None);
            }

            for &pool in self.free_pools.iter() {
                self.device.vk_handle().destroy_descriptor_pool(pool, None);
            }
        };
    }
}

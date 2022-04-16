use ash::vk;
use std::{collections::HashMap, rc::Rc};

use crate::core::device::{LogicalDevice, LogicalDeviceError};

#[derive(thiserror::Error, Debug)]
pub enum DescriptorCacheError {
    #[error(transparent)]
    DeviceError(#[from] LogicalDeviceError),
}

struct DescriptorLayoutInfo {
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
}

impl PartialEq for DescriptorLayoutInfo {
    fn eq(&self, other: &Self) -> bool {
        if self.bindings.len() != other.bindings.len() {
            false
        } else {
            for (self_binding, other_binding) in self.bindings.iter().zip(other.bindings.iter()) {
                if self_binding.binding != other_binding.binding {
                    return false;
                }
                if self_binding.descriptor_type != other_binding.descriptor_type {
                    return false;
                }
                if self_binding.descriptor_count != other_binding.descriptor_count {
                    return false;
                }
                if self_binding.stage_flags != other_binding.stage_flags {
                    return false;
                }
            }

            true
        }
    }
}

impl Eq for DescriptorLayoutInfo {
    fn assert_receiver_is_total_eq(&self) {}
}

impl std::hash::Hash for DescriptorLayoutInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for binding in self.bindings.iter() {
            let binding_hash = binding.binding as i32
                | binding.descriptor_type.as_raw() << 8
                | (binding.descriptor_count as i32) << 16
                | (binding.stage_flags.as_raw() as i32) << 24;
            state.write_i32(binding_hash);
        }
    }
}

pub struct DescriptorLayoutCache {
    layout_cache: HashMap<DescriptorLayoutInfo, vk::DescriptorSetLayout>,
    device: Rc<LogicalDevice>,
}

impl DescriptorLayoutCache {
    pub fn new(device: Rc<LogicalDevice>) -> DescriptorLayoutCache {
        DescriptorLayoutCache {
            layout_cache: HashMap::new(),
            device,
        }
    }
}

impl DescriptorLayoutCache {
    pub(crate) fn create_descriptor_layout(
        &mut self,
        create_info: vk::DescriptorSetLayoutCreateInfo,
    ) -> Result<vk::DescriptorSetLayout, DescriptorCacheError> {
        let mut layout_info = DescriptorLayoutInfo {
            bindings: Vec::with_capacity(create_info.binding_count as usize),
        };

        let mut is_sorted = true;
        let mut last_binding: i32 = -1;
        let mut p_binding = create_info.p_bindings.clone();
        for i in 0..create_info.binding_count as usize {
            layout_info
                .bindings
                .push(unsafe { p_binding.read() }.clone());
            unsafe {
                p_binding = p_binding.offset(1);
            };

            if layout_info.bindings[i].binding as i32 > last_binding {
                last_binding = layout_info.bindings[i].binding as i32;
            } else {
                is_sorted = false;
            }
        }

        if !is_sorted {
            layout_info
                .bindings
                .sort_by(|a, b| a.binding.cmp(&b.binding));
        }

        if !self.layout_cache.contains_key(&layout_info) {
            let handle = unsafe {
                self.device
                    .vk_handle()
                    .create_descriptor_set_layout(&create_info, None)
                    .map_err(|err| {
                        DescriptorCacheError::DeviceError(LogicalDeviceError::Other(err.into()))
                    })?
            };
            self.layout_cache.insert(layout_info, handle);

            Ok(handle)
        } else {
            Ok(self.layout_cache.get(&layout_info).unwrap().clone())
        }
    }
}

impl Drop for DescriptorLayoutCache {
    fn drop(&mut self) {
        for &layout in self.layout_cache.values() {
            unsafe {
                self.device
                    .vk_handle()
                    .destroy_descriptor_set_layout(layout, None);
            };
        }
    }
}

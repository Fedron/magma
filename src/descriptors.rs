use ash::vk;
use std::{collections::HashMap, rc::Rc};

use crate::device::Device;

pub struct DescriptorSetLayout {
    pub layout: vk::DescriptorSetLayout,
    pub bindings: HashMap<u32, vk::DescriptorSetLayoutBinding>,
}

impl DescriptorSetLayout {
    pub fn new(
        device: Rc<Device>,
        bindings: &[vk::DescriptorSetLayoutBinding],
    ) -> DescriptorSetLayout {
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();
        let layout = unsafe {
            device
                .vk()
                .create_descriptor_set_layout(&layout_info, None)
                .expect("Failed to create descriptor set layout")
        };

        let bindings: HashMap<u32, vk::DescriptorSetLayoutBinding> = HashMap::from_iter(
            bindings
                .iter()
                .enumerate()
                .map(|(index, &binding)| (index as u32, binding))
                .collect::<Vec<(u32, vk::DescriptorSetLayoutBinding)>>(),
        );

        DescriptorSetLayout { layout, bindings }
    }
}

pub struct DescriptorPoolBuilder {
    device: Rc<Device>,
    pool_sizes: Vec<vk::DescriptorPoolSize>,
    pool_flags: vk::DescriptorPoolCreateFlags,
    max_sets: u32,
}

impl DescriptorPoolBuilder {
    pub fn new(device: Rc<Device>) -> DescriptorPoolBuilder {
        DescriptorPoolBuilder {
            device,
            pool_sizes: Vec::new(),
            pool_flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: 1000,
        }
    }

    pub fn add_pool_size(
        mut self,
        descriptor_type: vk::DescriptorType,
        count: u32,
    ) -> DescriptorPoolBuilder {
        self.pool_sizes.push(vk::DescriptorPoolSize {
            ty: descriptor_type,
            descriptor_count: count,
        });
        self
    }

    pub fn pool_flags(mut self, flags: vk::DescriptorPoolCreateFlags) -> DescriptorPoolBuilder {
        self.pool_flags = flags;
        self
    }

    pub fn max_sets(mut self, count: u32) -> DescriptorPoolBuilder {
        self.max_sets = count;
        self
    }

    pub fn build(self) -> DescriptorPool {
        DescriptorPool::new(
            self.device,
            self.max_sets,
            self.pool_flags,
            &self.pool_sizes,
        )
    }
}

pub struct DescriptorPool {
    device: Rc<Device>,
    pool: vk::DescriptorPool,
}

impl DescriptorPool {
    pub fn new(
        device: Rc<Device>,
        max_sets: u32,
        pool_flags: vk::DescriptorPoolCreateFlags,
        pool_sizes: &[vk::DescriptorPoolSize],
    ) -> DescriptorPool {
        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .flags(pool_flags)
            .pool_sizes(pool_sizes)
            .max_sets(max_sets);

        let pool = unsafe {
            device
                .vk()
                .create_descriptor_pool(&pool_info, None)
                .expect("Failed to create descriptor pool")
        };

        DescriptorPool { device, pool }
    }

    pub fn builder(device: Rc<Device>) -> DescriptorPoolBuilder {
        DescriptorPoolBuilder::new(device)
    }

    pub fn allocate_descriptor_set(
        &self,
        set_layout: vk::DescriptorSetLayout,
    ) -> vk::DescriptorSet {
        let set_layouts = [set_layout];
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(&set_layouts);

        unsafe {
            self.device
                .vk()
                .allocate_descriptor_sets(&allocate_info)
                .expect("Failed to allocate descriptor set")
                .first()
                .expect("Failed to get allocated descriptor set")
                .clone()
        }
    }

    pub fn free_descriptor_sets(&self, descriptors: &[vk::DescriptorSet]) {
        unsafe {
            self.device
                .vk()
                .free_descriptor_sets(self.pool, &descriptors)
                .expect("Failed to free descriptor sets")
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

    pub fn write_buffer(
        mut self,
        binding: u32,
        buffer_info: vk::DescriptorBufferInfo,
    ) -> DescriptorWriter {
        if !self.layout.bindings.contains_key(&binding) {
            log::error!("Layout does not contain the binding {}", binding);
            panic!("Failed to write buffer to descriptor, see above");
        }

        let binding_description = self.layout.bindings.get(&binding).unwrap();

        self.writes.push(
            vk::WriteDescriptorSet::builder()
                .descriptor_type(binding_description.descriptor_type)
                .dst_binding(binding)
                .buffer_info(&[buffer_info])
                .build(),
        );
        self
    }

    pub fn overwrite(&mut self, set: vk::DescriptorSet) {
        for write in self.writes.iter_mut() {
            write.dst_set = set;
        }

        unsafe {
            self.pool
                .device
                .vk()
                .update_descriptor_sets(&self.writes, &[]);
        };
    }

    pub fn build(&mut self, set: vk::DescriptorSet) -> vk::DescriptorSet {
        let set = self.pool.allocate_descriptor_set(self.layout.layout);
        self.overwrite(set);
        set
    }
}

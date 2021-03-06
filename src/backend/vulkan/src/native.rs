use ash::vk;
use ash::version::DeviceV1_0;
use hal::pso;
use hal::image::SubresourceRange;
use std::borrow::Borrow;
use std::sync::Arc;
use {Backend, RawDevice};

#[derive(Debug, Hash)]
pub struct Semaphore(pub vk::Semaphore);

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Fence(pub vk::Fence);

#[derive(Debug, Hash)]
pub struct GraphicsPipeline(pub vk::Pipeline);

#[derive(Debug, Hash)]
pub struct ComputePipeline(pub vk::Pipeline);

#[derive(Debug, Hash)]
pub struct Memory {
    pub(crate) raw: vk::DeviceMemory,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Buffer {
    pub(crate) raw: vk::Buffer,
}

unsafe impl Sync for Buffer {}
unsafe impl Send for Buffer {}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BufferView {
    pub(crate) raw: vk::BufferView,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Image {
    pub(crate) raw: vk::Image,
    pub(crate) ty: vk::ImageType,
    pub(crate) flags: vk::ImageCreateFlags,
    pub(crate) extent: vk::Extent3D,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ImageView {
    pub(crate) image: vk::Image,
    pub(crate) view: vk::ImageView,
    pub(crate) range: SubresourceRange,
}

#[derive(Debug, Hash)]
pub struct Sampler(pub vk::Sampler);

#[derive(Debug, Hash)]
pub struct RenderPass {
    pub raw: vk::RenderPass,
}

#[derive(Debug, Hash)]
pub struct Framebuffer {
    pub(crate) raw: vk::Framebuffer,
}

#[derive(Debug)]
pub struct DescriptorSetLayout {
    pub(crate) raw: vk::DescriptorSetLayout,
    pub(crate) bindings: Arc<Vec<pso::DescriptorSetLayoutBinding>>,
}

#[derive(Debug)]
pub struct DescriptorSet {
    pub(crate) raw: vk::DescriptorSet,
    pub(crate) bindings: Arc<Vec<pso::DescriptorSetLayoutBinding>>,
}

#[derive(Debug, Hash)]
pub struct PipelineLayout {
    pub(crate) raw: vk::PipelineLayout,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct ShaderModule {
    pub(crate) raw: vk::ShaderModule,
}

#[derive(Debug)]
pub struct DescriptorPool {
    pub(crate) raw: vk::DescriptorPool,
    pub(crate) device: Arc<RawDevice>,
    /// This vec only exists to re-use allocations when `DescriptorSet`s are freed.
    pub(crate) set_free_vec: Vec<vk::DescriptorSet>,
}

impl pso::DescriptorPool<Backend> for DescriptorPool {
    fn allocate_sets<I>(&mut self, layout_iter: I) -> Vec<Result<DescriptorSet, pso::AllocationError>>
    where
        I: IntoIterator,
        I::Item: Borrow<DescriptorSetLayout>,
    {
        use std::ptr;

        let mut raw_layouts = Vec::new();
        let mut layout_bindinds = Vec::new();
        for layout in layout_iter {
            raw_layouts.push(layout.borrow().raw);
            layout_bindinds.push(layout.borrow().bindings.clone());
        }

        let info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DescriptorSetAllocateInfo,
            p_next: ptr::null(),
            descriptor_pool: self.raw,
            descriptor_set_count: raw_layouts.len() as u32,
            p_set_layouts: raw_layouts.as_ptr(),
        };

        let descriptor_sets = unsafe {
            self.device.0.allocate_descriptor_sets(&info)
        };

        match descriptor_sets {
            Ok(sets) => {
                sets.into_iter()
                    .zip(layout_bindinds.into_iter())
                    .map(|(raw, bindings)| {
                        Ok(DescriptorSet { raw, bindings })
                    })
                    .collect()
            }
            Err(err) => vec![Err(match err {
                vk::Result::ErrorOutOfHostMemory => pso::AllocationError::OutOfHostMemory,
                vk::Result::ErrorOutOfDeviceMemory => pso::AllocationError::OutOfDeviceMemory,
                // TODO: Uncomment when ash updates to include VK_ERROR_OUT_OF_POOL_MEMORY(_KHR)
                // vk::Result::ErrorOutOfPoolMemory => pso::AllocationError::OutOfPoolMemory,
                _ => pso::AllocationError::FragmentedPool,
            })]
        }
    }

    fn free_sets<I>(&mut self, descriptor_sets: I)
    where
        I: IntoIterator<Item = DescriptorSet>
    {
        self.set_free_vec.clear();
        self.set_free_vec.extend(descriptor_sets.into_iter().map(|d| d.raw));
        unsafe {
            self.device.0.free_descriptor_sets(self.raw, &self.set_free_vec);
        }
    }

    fn reset(&mut self) {
        assert_eq!(Ok(()), unsafe {
            self.device.0.reset_descriptor_pool(
                self.raw,
                vk::DescriptorPoolResetFlags::empty(),
            )
        });
    }
}

#[derive(Debug, Hash)]
pub struct QueryPool(pub vk::QueryPool);

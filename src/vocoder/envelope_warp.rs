use std::{
    sync::Arc,
};

use vulkano::{
    buffer::{BufferUsage, DeviceLocalBuffer},
    descriptor_set::{
        allocator::DescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet, layout::DescriptorSetLayout,
    },
    memory::allocator::{MemoryAllocator}, command_buffer::AutoCommandBufferBuilder,
};


pub struct EnvelopeWarpDescriptorSets<A: DescriptorSetAllocator + ?Sized> {
    pub descriptor_set: Arc<PersistentDescriptorSet<A::Alloc>>,
}

impl<A: DescriptorSetAllocator + ?Sized> EnvelopeWarpDescriptorSets<A> {
    pub fn new<L>(
        polynomial: [f32; 8],
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        descriptor_set_allocator: &A,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L>,
        set_layout: Arc<DescriptorSetLayout>,
    ) -> EnvelopeWarpDescriptorSets<A>  {
        let buffer = {
            DeviceLocalBuffer::from_data(
                memory_allocator,
                polynomial,
                BufferUsage {storage_buffer: true, ..BufferUsage::empty()}, command_buffer_builder,
            ).unwrap()
        };

        let set = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            set_layout.clone(),
            [
                WriteDescriptorSet::buffer(0, buffer.clone()),
            ],
        ).unwrap();

        EnvelopeWarpDescriptorSets {
            descriptor_set: set,
        }
    }
}

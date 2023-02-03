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


pub struct PitchShiftDescriptorSets<A: DescriptorSetAllocator + ?Sized> {
    pub descriptor_set_ft: Arc<PersistentDescriptorSet<A::Alloc>>,
    pub descriptor_set_ift: Arc<PersistentDescriptorSet<A::Alloc>>,
}

impl<A: DescriptorSetAllocator + ?Sized> PitchShiftDescriptorSets<A> {
    pub fn new<L>(
        pitch_ratio: f32,
        delay: f32,
        mix_span: f32,
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        descriptor_set_allocator: &A,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L>,
        set_layout_ft: Arc<DescriptorSetLayout>,
        set_layout_ift: Arc<DescriptorSetLayout>,
    ) -> PitchShiftDescriptorSets<A>  {
        let buffer = {
            DeviceLocalBuffer::from_data(
                memory_allocator,
                [pitch_ratio, delay, mix_span],
                BufferUsage {storage_buffer: true, ..BufferUsage::empty()}, command_buffer_builder,
            ).unwrap()
        };
    
        let set_ft = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            set_layout_ft.clone(),
            [
                WriteDescriptorSet::buffer(0, buffer.clone()),
            ],
        ).unwrap();

        let set_ift = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            set_layout_ift.clone(),
            [
                WriteDescriptorSet::buffer(0, buffer.clone()),
            ],
        ).unwrap();

        PitchShiftDescriptorSets {
            descriptor_set_ft: set_ft,
            descriptor_set_ift: set_ift,
        }
    }
}

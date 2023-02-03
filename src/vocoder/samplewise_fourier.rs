
use std::{
    sync::Arc,
};

use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer},
    descriptor_set::{
        allocator::DescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet, layout::DescriptorSetLayout,
    },
    memory::allocator::{MemoryAllocator}, command_buffer::AutoCommandBufferBuilder,
};


const MAX_WAVE_LENGTH: usize = 1024;

pub struct SamplewiseFourierDescriptorSets<A: DescriptorSetAllocator + ?Sized> {
    pub descriptor_set_ft: Arc<PersistentDescriptorSet<A::Alloc>>,
    pub descriptor_set_ift: Arc<PersistentDescriptorSet<A::Alloc>>,
    pub result_ft: Arc<DeviceLocalBuffer<[[[f32; 2]; MAX_WAVE_LENGTH]]>>,
    pub result_ift: Arc<CpuAccessibleBuffer<[f32]>>,
    input: Arc<CpuAccessibleBuffer<[f32]>>,
}

impl<A: DescriptorSetAllocator + ?Sized> SamplewiseFourierDescriptorSets<A> {
    pub fn new<L>(
        input_buffer_length: usize,
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        descriptor_set_allocator: &A,
        command_buffer_builder: &mut AutoCommandBufferBuilder<L>,
        set_layout_ft: Arc<DescriptorSetLayout>,
        set_layout_ift: Arc<DescriptorSetLayout>,
    ) -> SamplewiseFourierDescriptorSets<A>  {
        let time_buffer = {
            DeviceLocalBuffer::from_data(
                memory_allocator,
                0,
                BufferUsage {storage_buffer: true, ..BufferUsage::empty()}, command_buffer_builder,
            ).unwrap()
        };
        let state_buffer = {
            let data_iter = (0..input_buffer_length).map(|_| [0, 0]);
            DeviceLocalBuffer::from_iter(
                memory_allocator,
                data_iter,
                BufferUsage {storage_buffer: true, ..BufferUsage::empty()}, command_buffer_builder,
            ).unwrap()
        };
        let input_buffer = {
            let data_iter = (0..input_buffer_length).map(|_| 0.0f32);
            CpuAccessibleBuffer::from_iter(
                memory_allocator, BufferUsage {storage_buffer: true, ..BufferUsage::empty()}, false,
                data_iter,
            ).unwrap()
        };
        let history_buffer = {
            let data_iter = (0..MAX_WAVE_LENGTH).map(|_| 0.0f32);
            DeviceLocalBuffer::from_iter(
                memory_allocator,
                data_iter, 
                BufferUsage {storage_buffer: true, ..BufferUsage::empty()}, command_buffer_builder,
            ).unwrap()
        };
        let ft_result_buffer = {
            let data_iter = (0..input_buffer_length).map(|_| [[0.0f32, 0.0f32]; MAX_WAVE_LENGTH]);
            DeviceLocalBuffer::from_iter(
                memory_allocator,
                data_iter,
                BufferUsage {storage_buffer: true, ..BufferUsage::empty()}, command_buffer_builder,
            ).unwrap()
        };
        let ift_result_buffer = {
            let data_iter = (0..input_buffer_length).map(|_| 0.0f32);
            CpuAccessibleBuffer::from_iter(
                memory_allocator, BufferUsage {storage_buffer: true, ..BufferUsage::empty()}, false,
                data_iter,
            ).unwrap()
        };
    
        let set_ft = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            set_layout_ft.clone(),
            [
                WriteDescriptorSet::buffer(0, time_buffer.clone()),
                WriteDescriptorSet::buffer(1, state_buffer.clone()),
                WriteDescriptorSet::buffer(2, input_buffer.clone()),
                WriteDescriptorSet::buffer(3, history_buffer.clone()),
                WriteDescriptorSet::buffer(4, ft_result_buffer.clone()),
            ],
        ).unwrap();
    
        let set_ift = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            set_layout_ift.clone(),
            [
                WriteDescriptorSet::buffer(0, time_buffer.clone()),
                WriteDescriptorSet::buffer(1, ft_result_buffer.clone()),
                WriteDescriptorSet::buffer(2, ift_result_buffer.clone()),
            ],
        ).unwrap();

        SamplewiseFourierDescriptorSets {
            descriptor_set_ft: set_ft,
            descriptor_set_ift: set_ift,
            result_ft: ft_result_buffer,
            result_ift: ift_result_buffer,
            input: input_buffer,
        }
    }
    pub fn update_input(&mut self, src: &[f32]) {
        let mut input_buffer_content = self.input.write().unwrap();
        input_buffer_content.clone_from_slice(src);
    }
}

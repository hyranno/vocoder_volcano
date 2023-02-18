

mod samplewise_fourier;
use samplewise_fourier::SamplewiseFourierDescriptorSets;

mod pitch_shift;
use pitch_shift::PitchShiftDescriptorSets;

mod equalizer;
use equalizer::EqualizerDescriptorSets;

mod envelope_warp;
use envelope_warp::EnvelopeWarpDescriptorSets;


use std::{
    sync::Arc,
};

use vulkano::{
    command_buffer::{
        allocator::{StandardCommandBufferAllocator}, AutoCommandBufferBuilder, CommandBufferUsage,
    },
    descriptor_set::{
        allocator::{StandardDescriptorSetAllocator}, PersistentDescriptorSet,
    },
    device::{
        Device, DeviceOwned, Queue,
    },
    memory::allocator::{StandardMemoryAllocator},
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};


pub trait AudioFilter {
    fn process(&mut self, src: &[f32], dest: &mut [f32]);
}

#[derive(Clone)]
pub struct VocoderSettings {
    pub pitch_shift_ratio: f32,
    pub delay: f32,
    pub mix_span: f32,
    pub equalizer: [f32; 8],
    pub envelope_warp: [f32; 8],
}
impl Default for VocoderSettings {
    fn default() -> Self {
        Self {
            pitch_shift_ratio: 1.0,
            delay: 341.0,
            mix_span: 0.9,
            equalizer: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            envelope_warp: [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }
}


pub struct Vocoder {
    vulkan_device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    pipeline_ft: Arc<ComputePipeline>,
    pipeline_vocoder: Arc<ComputePipeline>,
    descriptor_sets_ft: Vec<Arc<PersistentDescriptorSet>>,
    descriptor_sets_vocoder: Vec<Arc<PersistentDescriptorSet>>,
    samplewise_fourier_descriptor_sets: SamplewiseFourierDescriptorSets<StandardDescriptorSetAllocator>,
}

unsafe impl DeviceOwned for Vocoder {
    fn device(&self) -> &Arc<Device> {
        return &self.vulkan_device
    }
}

impl AudioFilter for Vocoder {
    fn process(&mut self, src: &[f32], dest: &mut [f32]) {
        self.samplewise_fourier_descriptor_sets.update_input(src);
        self.process_gpu();
        let dest_buffer_content = self.samplewise_fourier_descriptor_sets.result_ift.read().unwrap();
        for i in 0..1024 {
            dest[i] = dest_buffer_content[i];
        }
    }
}

impl Vocoder {
    pub fn new(queue: Arc<Queue>, settings: VocoderSettings) -> Vocoder {
        let device = queue.device();
        let memory_allocator = StandardMemoryAllocator::new_default(device.clone());
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());
        let (pipeline_ft, pipeline_vocoder) = Self::create_pipelines(device);
        let set_layouts_ft = pipeline_ft.layout().set_layouts();
        let set_layouts_vocoder= pipeline_vocoder.layout().set_layouts();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        let samplewise_fourier_descriptor_sets = SamplewiseFourierDescriptorSets::new(
            1024,
            &memory_allocator, &descriptor_set_allocator, &mut command_buffer_builder,
            set_layouts_ft.get(0).unwrap().clone(), set_layouts_vocoder.get(0).unwrap().clone(),
        );
        let pitch_shift_descriptor_sets = PitchShiftDescriptorSets::new(
            settings.pitch_shift_ratio, settings.delay, settings.mix_span,
            &memory_allocator, &descriptor_set_allocator, &mut command_buffer_builder,
            set_layouts_ft.get(1).unwrap().clone(), set_layouts_vocoder.get(1).unwrap().clone(),
        );
        let equalizer_descriptor_sets = EqualizerDescriptorSets::new(
            settings.equalizer.clone(),
            &memory_allocator, &descriptor_set_allocator, &mut command_buffer_builder,
            set_layouts_vocoder.get(2).unwrap().clone(),
        );
        let envelope_warp_descriptor_sets = EnvelopeWarpDescriptorSets::new(
            settings.envelope_warp.clone(),
            &memory_allocator, &descriptor_set_allocator, &mut command_buffer_builder,
            set_layouts_vocoder.get(3).unwrap().clone(),
        );

        sync::now(device.clone())
            .then_execute(queue.clone(), command_buffer_builder.build().unwrap()).unwrap()
            .then_signal_fence_and_flush().unwrap()
            .wait(None).unwrap()
        ;

        let descriptor_sets_ft = vec![
            samplewise_fourier_descriptor_sets.descriptor_set_ft.clone(),
            pitch_shift_descriptor_sets.descriptor_set_ft.clone(),
        ];
        let descriptor_sets_vocoder = vec![
            samplewise_fourier_descriptor_sets.descriptor_set_ift.clone(),
            pitch_shift_descriptor_sets.descriptor_set_ift.clone(),
            equalizer_descriptor_sets.descriptor_set.clone(),
            envelope_warp_descriptor_sets.descriptor_set.clone(),
        ];

        Vocoder {
            vulkan_device: device.clone(),
            queue: queue,
            command_buffer_allocator: command_buffer_allocator,
            pipeline_ft: pipeline_ft, pipeline_vocoder: pipeline_vocoder,
            descriptor_sets_ft: descriptor_sets_ft, descriptor_sets_vocoder: descriptor_sets_vocoder,
            samplewise_fourier_descriptor_sets: samplewise_fourier_descriptor_sets
        }
    }
    fn create_pipelines(device: &Arc<Device>) -> (Arc<ComputePipeline>, Arc<ComputePipeline>) {
        let pipeline_ft = {
            mod cs {
                vulkano_shaders::shader! {
                    ty: "compute",
                    path: "src/vocoder/samplewise-fourier.glsl.comp",
                }
            }
            let shader = cs::load(device.clone()).unwrap();
            ComputePipeline::new(
                device.clone(),
                shader.entry_point("main").unwrap(),
                &(), None, |_| {},
            ).unwrap()
        };
    
        let pipeline_ift = {
            mod cs {
                vulkano_shaders::shader! {
                    ty: "compute",
                    path: "src/vocoder/vocoder.glsl.comp",
                }
            }
            let shader = cs::load(device.clone()).unwrap();
            ComputePipeline::new(
                device.clone(),
                shader.entry_point("main").unwrap(),
                &(), None, |_| {},
            ).unwrap()
        };
    
        (pipeline_ft, pipeline_ift)
    }

    fn process_gpu(&self) {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        builder
            .bind_pipeline_compute(self.pipeline_ft.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pipeline_ft.layout().clone(),
                0,
                self.descriptor_sets_ft.clone(),
            )
            .dispatch([1, 1, 1]).expect("failed to dispatch")
            .bind_pipeline_compute(self.pipeline_vocoder.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pipeline_vocoder.layout().clone(),
                0,
                self.descriptor_sets_vocoder.clone(),
            )
            .dispatch([1024, 1, 1]).expect("failed to dispatch");
        let command_buffer = builder.build().unwrap();
    

        let future = sync::now(self.vulkan_device.clone())
            .then_execute(self.queue.clone(), command_buffer).unwrap()
            .then_signal_fence_and_flush().unwrap();
    
        future.wait(None).unwrap();
    }
}


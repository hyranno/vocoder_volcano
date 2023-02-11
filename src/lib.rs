// use core::slice::SlicePattern;

use lv2::prelude::*;

pub mod vocoder;
pub mod vulcan_helper;

use vocoder::{Vocoder, VocoderSettings, AudioFilter};
use vulcan_helper::{create_vulcan_device};

#[derive(PortCollection)]
struct Ports {
    // gain: InputPort<Control>,
    input: InputPort<Audio>,
    output: OutputPort<Audio>,
}

#[uri("http://uncotechhack.net/plugins/vocoder_volcano")]
struct VocoderVolcano {
    filter: Vocoder,
    buffer_in: Vec<f32>,
    buffer_out: Vec<f32>,
}

impl Plugin for VocoderVolcano {
    type Ports = Ports;
    type InitFeatures = ();
    type AudioFeatures = ();

    fn new(_plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
        let (_, mut queues) = create_vulcan_device();
        let capacity = 2048;
        Some(Self {
            filter: Vocoder::new(queues.next().unwrap(), VocoderSettings::default()),
            buffer_in: Vec::with_capacity(capacity),
            buffer_out: Vec::with_capacity(capacity),
        })
    }

    fn run(&mut self, ports: &mut Ports, _features: &mut (), sample_count: u32) {
        // self.filter.process(ports.input.as_slice(), ports.output.as_mut_slice());
        let mut it = ports.input.iter();
        self.buffer_in.clear();
        self.buffer_in.resize_with(sample_count as usize, || *it.next().unwrap());
        self.buffer_out.resize(sample_count as usize, 0.0);
        self.filter.process(self.buffer_in.as_slice(), self.buffer_out.as_mut_slice());
        self.buffer_out.iter().zip(ports.output.iter_mut()).for_each(|(src, dest)| *dest = *src);
    }
}

lv2_descriptors!(VocoderVolcano);
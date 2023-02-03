use vocoder_volcano::vulcan_helper::{create_vulcan_device};
use vocoder_volcano::vocoder::{Vocoder, VocoderSettings, AudioFilter};

const INPUT_BUFFER_LENGTH: usize = 1024;

fn main() {
    let (_, mut queues) = create_vulcan_device();
    let vocoder_settings = VocoderSettings {
        pitch_shift_ratio: 0.8,
        ..VocoderSettings::default()
    };
    let mut vocoder = Vocoder::new(queues.next().unwrap(), vocoder_settings);

    let src: Vec<f32> = (0..INPUT_BUFFER_LENGTH).map(|i| (i as f32).sin()).collect();
    let mut dest: Vec<f32> = (0..INPUT_BUFFER_LENGTH).map(|_| 0.0f32).collect();

    vocoder.process(src.as_slice(), dest.as_mut_slice());

    for i in 0..INPUT_BUFFER_LENGTH {
        println!("{}, {}", src[i], dest[i]);
    }
}
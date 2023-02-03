use hound;

use vocoder_volcano::vulcan_helper::{create_vulcan_device};
use vocoder_volcano::vocoder::{Vocoder, VocoderSettings, AudioFilter};

const INPUT_BUFFER_LENGTH: usize = 1024;

struct Chunks<I: Iterator<Item = f32>> {
    src: I,
}
impl<I: Iterator<Item = f32>> Iterator for Chunks<I> {
    type Item = Vec<f32>;
    #[inline]
    fn next(&mut self) -> Option<Vec<f32>> {
        // self.src.next_chunk::<INPUT_BUFFER_LENGTH>()
        let first = self.src.next();
        if first.is_none() {
            return None;
        }
        let mut dest = Vec::with_capacity(INPUT_BUFFER_LENGTH);
        dest.push(first.unwrap());
        (0..INPUT_BUFFER_LENGTH-1).for_each(|_| dest.push(self.src.next().unwrap_or_default()));
        Some(dest)
    }
}

fn main() {
    let (_, mut queues) = create_vulcan_device();
    let vocoder_settings = VocoderSettings {
        pitch_shift_ratio: 1.10,
        ..VocoderSettings::default()
    };
    let mut vocoder = Vocoder::new(queues.next().unwrap(), vocoder_settings);

    let mut reader = hound::WavReader::open("examples/voice_mono.wav").unwrap();
    let spec = reader.spec();
    let source = reader.samples::<i16>().map(|s| s.unwrap() as f32 / (i16::MAX as f32));
    // let source = (0..48000*8).map(|i| (std::f32::consts::PI * (i as f32) * 440.0 / 48000.0).sin());
    let transformed = Chunks{src: source}
        .map(move |src| {
            let mut dest = src.clone();
            vocoder.process(&dest.clone(), &mut dest);
            dest
        })
        .flatten()
    ;

    print!("{:?}", spec);

    let mut writer = hound::WavWriter::create("examples/transformed.wav", spec).unwrap();
    transformed.for_each(|sample| writer.write_sample((sample * (i16::MAX as f32)) as i16).unwrap());
}
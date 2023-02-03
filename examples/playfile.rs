use std::fs::File;
use std::io::BufReader;
use core::time::Duration;
// use rodio::cpal::FrameCount;
use rodio::{Decoder, OutputStream, Sink, Source};

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

struct MappedSource<I: Iterator<Item = f32>> {
    iter: I,
    frame_len: Option<usize>,
    rate: u32,
    duration: Option<Duration>,
}
impl<I: Iterator<Item = f32>> Iterator for MappedSource<I> {
    type Item = f32;
    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.iter.next()
    }
}
impl<I: Iterator<Item = f32>> Source for MappedSource<I> {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.frame_len
    }
    #[inline]
    fn channels(&self) -> u16 {
        1
    }
    #[inline]
    fn sample_rate(&self) -> u32 {
        self.rate
    }
    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.duration
    }
}


fn main() {
    let (_, mut queues) = create_vulcan_device();
    let vocoder_settings = VocoderSettings {
        pitch_shift_ratio: 1.20,
        ..VocoderSettings::default()
    };
    let mut vocoder = Vocoder::new(queues.next().unwrap(), vocoder_settings);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let file = BufReader::new(File::open("examples/voice_mono.wav").unwrap());
    let source = Decoder::new(file).unwrap();

    let frame_len = source.current_frame_len();
    let sample_rate = source.sample_rate();
    let total_duration = source.total_duration();

    let transformed = Chunks{src: source.convert_samples::<f32>()}
        .map(move |src| {
            let mut dest = src.clone();
            vocoder.process(&dest.clone(), &mut dest);
            dest
        })
        .flatten()
    ;

    let mapped_source = MappedSource {
        iter: transformed,
        frame_len: frame_len,
        rate: sample_rate,
        duration: total_duration,
    };

    sink.append(mapped_source);

    sink.sleep_until_end();
}
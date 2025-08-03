use std::f64::consts::PI;
use std::fs::File;
use std::io::prelude::Write;
use std::io::{Seek, SeekFrom};
use std::path::Path;


const SAMPLE_RATE: i32 = 44100;
const BIT_DEPTH: i32 = 16;


#[derive(Debug)]
struct SineOscillator {
    amplitude: f64,
    index: f64,
    increment: f64,
    cycle_counter: i128,
}

impl SineOscillator {
    fn new(frequency: f64, amplitude: f64) -> SineOscillator {
        SineOscillator {
            amplitude: amplitude,
            index: 0.0,
            increment: (2.0 * PI * frequency) / SAMPLE_RATE as f64,
            cycle_counter: 0,
        }
    }

    fn sample(self: &mut Self) -> f64 {
        let sample = self.amplitude * self.index.sin();

        self.index += self.increment;
        self.cycle_counter += 1;

        return sample
    }
}

#[derive(Debug)]
struct SawOscillator {
    amplitude: f64,
    index: f64,
    increment: f64,
    cycle_counter: i128,
}

impl SawOscillator {
    fn new(frequency: f64, amplitude: f64) -> SawOscillator {
        SawOscillator {
            amplitude: amplitude,
            index: 0.0,
            increment: frequency / SAMPLE_RATE as f64,
            cycle_counter: 0,
        }
    }

    fn sample(self: &mut Self) -> f64 {
         let mut sample = -self.amplitude + self.index;

         if sample > self.amplitude {
            sample = -self.amplitude;
            self.index = self.increment;
         }
         else {
            self.index += self.increment;
         }

         self.cycle_counter += 1;

         return sample
    }
}

fn scale_to_bit_depth(sample: f64) -> i16 {
    let base: i64 = 2;
    let max_amplitude = base.pow(BIT_DEPTH as u32 - 1) - 1;
    let scaled_float: f64 = sample * max_amplitude as f64;
    return scaled_float as i16
}

fn write_2_bytes(file: &mut File, val: i16) {
    let as_le_bytes = val.to_le_bytes();
    file.write(&as_le_bytes).unwrap();
}

fn write_4_bytes(file: &mut File, val: i32) {
    let as_le_bytes = val.to_le_bytes();
    file.write(&as_le_bytes).unwrap();
}

fn main() {
    // let mut oscillator = SineOscillator::new(440.0, 0.5);
    let mut oscillator = SawOscillator::new(880.0, 0.5);

    let duration = 2.0;
    let path = Path::new("waveform.wav");

    let mut file = File::create(path).unwrap();

    // Header chunk
    file.write(b"RIFF").unwrap();
    file.write(b"----").unwrap();
    file.write(b"WAVE").unwrap();

    // Format chunk
    file.write(b"fmt ").unwrap();
    write_4_bytes(&mut file, 16); // Chunk data size
    write_2_bytes(&mut file, 1); // Compression code: PCM/uncompressed
    write_2_bytes(&mut file, 1); // Number of channels
    write_4_bytes(&mut file, SAMPLE_RATE); // Sample rate
    write_4_bytes(&mut file, (SAMPLE_RATE * BIT_DEPTH) / 8); // Byte rate
    write_2_bytes(&mut file, (BIT_DEPTH / 8) as i16); // Block align
    write_2_bytes(&mut file, BIT_DEPTH as i16); // Significant bits per sample

    // Data chunk
    file.write(b"data").unwrap();
    file.write(b"----").unwrap();

    let pre_audio_position = file.stream_position().unwrap();

    for _ in 1..(SAMPLE_RATE as f64 * duration) as i64 + 1 {
        let sample = oscillator.sample();
        let scaled_sample = scale_to_bit_depth(sample) as i16;

        write_2_bytes(&mut file, scaled_sample);
    }

    let post_audio_position = file.stream_position().unwrap();

    file.seek(SeekFrom::Start(pre_audio_position - 4)).unwrap();
    write_4_bytes(&mut file, (post_audio_position - pre_audio_position) as i32);

    file.seek(SeekFrom::Start(4)).unwrap();
    write_4_bytes(&mut file, (post_audio_position - 8) as i32);
}

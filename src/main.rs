use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use std::thread;
use ringbuf::RingBuffer;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SampleFormat, StreamConfig,
};

// Audio effect types
#[derive(Clone, Copy)]
enum AudioEffect {
    Passthrough,
    Gain(f32),
    Distortion(f32),
    LowPassFilter(f32),
    HighPassFilter(f32),
}

struct AudioProcessor {
    effect: AudioEffect,
    sample_rate: f32,
    // Filter state
    last_sample: f32,
}

impl AudioProcessor {
    fn new(effect: AudioEffect, sample_rate: f32) -> Self {
        Self {
            effect,
            sample_rate,
            last_sample: 0.0,
        }
    }

    fn set_effect(&mut self, effect: AudioEffect) {
        self.effect = effect;
        self.last_sample = 0.0; // Reset filter state
    }

    fn process(&mut self, sample: f32) -> f32 {
        match self.effect {
            AudioEffect::Passthrough => sample,
            
            AudioEffect::Gain(amount) => {
                sample * amount
            }
            
            AudioEffect::Distortion(amount) => {
                // Simple soft clipping distortion
                let threshold = 0.5;
                if sample.abs() > threshold {
                    let sign = sample.signum();
                    let excess = sample.abs() - threshold;
                    let distortion = excess * amount;
                    sign * (threshold + distortion).min(1.0)
                } else {
                    sample
                }
            }
            
            AudioEffect::LowPassFilter(cutoff) => {
                // Simple first-order low-pass filter
                let alpha = cutoff / (cutoff + self.sample_rate);
                self.last_sample = alpha * sample + (1.0 - alpha) * self.last_sample;
                self.last_sample
            }
            
            AudioEffect::HighPassFilter(cutoff) => {
                // Simple first-order high-pass filter
                let alpha = self.sample_rate / (cutoff + self.sample_rate);
                let filtered = alpha * (self.last_sample + sample - self.last_sample);
                self.last_sample = sample;
                filtered
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    // 1. find host + default devices
    let host = cpal::default_host();
    let input_dev  = host.default_input_device()
        .expect("no input device");
    let output_dev = host.default_output_device()
        .expect("no output device");

    // 2. pick a matching config (we'll use the input's config)
    let in_cfg = input_dev.default_input_config()?;
    let cfg: StreamConfig = in_cfg.clone().into();

    println!("Audio format: {:?}", in_cfg.sample_format());
    println!("Sample rate: {}", in_cfg.sample_rate().0);
    println!("Channels: {}", in_cfg.channels());

    // 3. create a ring buffer for samples
    let rb = RingBuffer::<f32>::new(32_768);
    let (prod, cons) = rb.split();

    // 4. create audio processor with shared state
    let processor = Arc::new(Mutex::new(AudioProcessor::new(
        AudioEffect::Passthrough, // Start with passthrough
        in_cfg.sample_rate().0 as f32
    )));

    // 5. build input stream
    let input_stream = match in_cfg.sample_format() {
        SampleFormat::F32 => build_input::<f32>(&input_dev, &cfg, prod, processor.clone()),
        SampleFormat::I16 => build_input::<i16>(&input_dev, &cfg, prod, processor.clone()),
        SampleFormat::U16 => build_input::<u16>(&input_dev, &cfg, prod, processor.clone()),
    }?;

    // 6. build output stream
    let output_stream = match in_cfg.sample_format() {
        SampleFormat::F32 => build_output::<f32>(&output_dev, &cfg, cons),
        SampleFormat::I16 => build_output::<i16>(&output_dev, &cfg, cons),
        SampleFormat::U16 => build_output::<u16>(&output_dev, &cfg, cons),
    }?;

    // 7. start audio processing
    println!("Starting audio passthrough...");
    input_stream.play()?;
    output_stream.play()?;

    // 8. start interactive control thread
    let processor_clone = processor.clone();
    thread::spawn(move || {
        interactive_control(processor_clone);
    });

    // 9. keep main thread alive
    std::thread::park();
    Ok(())
}

fn interactive_control(processor: Arc<Mutex<AudioProcessor>>) {
    println!("\n=== Audio Effects Control ===");
    println!("1: Passthrough (no effect)");
    println!("2: Gain (amplify)");
    println!("3: Distortion");
    println!("4: Low-pass filter");
    println!("5: High-pass filter");
    println!("q: Quit");
    println!("============================\n");

    loop {
        print!("Select effect (1-5, q): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        let mut proc = processor.lock().unwrap();
        
        match input {
            "1" => {
                proc.set_effect(AudioEffect::Passthrough);
                println!("Effect: Passthrough");
            }
            "2" => {
                proc.set_effect(AudioEffect::Gain(2.0));
                println!("Effect: Gain (2x)");
            }
            "3" => {
                proc.set_effect(AudioEffect::Distortion(0.3));
                println!("Effect: Distortion");
            }
            "4" => {
                proc.set_effect(AudioEffect::LowPassFilter(1000.0));
                println!("Effect: Low-pass filter (1kHz)");
            }
            "5" => {
                proc.set_effect(AudioEffect::HighPassFilter(1000.0));
                println!("Effect: High-pass filter (1kHz)");
            }
            "q" => {
                println!("Quitting...");
                std::process::exit(0);
            }
            _ => {
                println!("Invalid option. Please try again.");
            }
        }
    }
}

// callback to read from mic, convert to f32, process, and enqueue
fn build_input<T>(
    device: &cpal::Device,
    cfg: &StreamConfig,
    mut producer: ringbuf::Producer<f32>,
    processor: Arc<Mutex<AudioProcessor>>,
) -> anyhow::Result<cpal::Stream>
where
    T: Sample,
{
    let err_fn = |e| eprintln!("input error: {}", e);
    let stream = device.build_input_stream(
        cfg,
        move |data: &[T], _| {
            let mut proc = processor.lock().unwrap();
            for &sample in data {
                let s = sample.to_f32();
                let processed = proc.process(s);
                let _ = producer.push(processed);
            }
        },
        err_fn,
    )?;
    Ok(stream)
}

// callback to dequeue, convert back, and send to speakers
fn build_output<T>(
    device: &cpal::Device,
    cfg: &StreamConfig,
    mut consumer: ringbuf::Consumer<f32>,
) -> anyhow::Result<cpal::Stream>
where
    T: Sample,
{
    let err_fn = |e| eprintln!("output error: {}", e);
    let stream = device.build_output_stream(
        cfg,
        move |out: &mut [T], _| {
            for slot in out {
                // if we're out of data, play silence
                let s = consumer.pop().unwrap_or(0.0);
                *slot = Sample::from::<f32>(&s);
            }
        },
        err_fn,
    )?;
    Ok(stream)
}
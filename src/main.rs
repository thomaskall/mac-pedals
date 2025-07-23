use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
use freeverb::Freeverb;
use ringbuf::{RingBuffer, Producer, Consumer};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

mod distortion;
use distortion::{Distortion, DistortionType};

// Function to print detailed device configuration
fn print_device_config(input_device: &cpal::Device, output_device: &cpal::Device, 
                      input_config: &cpal::SupportedStreamConfig, 
                      output_config: &cpal::SupportedStreamConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Detailed Device Configuration ===");
    println!("Input Device: {}", input_device.name()?);
    println!("  Sample Rate: {} Hz", input_config.sample_rate().0);
    println!("  Channels: {}", input_config.channels());
    println!("  Sample Format: {:?}", input_config.sample_format());
    println!("  Buffer Size: {:?}", input_config.buffer_size());
    
    println!("\nOutput Device: {}", output_device.name()?);
    println!("  Sample Rate: {} Hz", output_config.sample_rate().0);
    println!("  Channels: {}", output_config.channels());
    println!("  Sample Format: {:?}", output_config.sample_format());
    println!("  Buffer Size: {:?}", output_config.buffer_size());
    
    // Check for potential issues
    if input_config.sample_rate() != output_config.sample_rate() {
        println!("\n⚠️  WARNING: Sample rate mismatch!");
        println!("   Input: {} Hz, Output: {} Hz", 
                input_config.sample_rate().0, output_config.sample_rate().0);
    }
    
    if input_config.channels() != output_config.channels() {
        println!("\n⚠️  WARNING: Channel count mismatch!");
        println!("   Input: {} channels, Output: {} channels", 
                input_config.channels(), output_config.channels());
    }
    
    if input_config.sample_format() != output_config.sample_format() {
        println!("\n⚠️  WARNING: Sample format mismatch!");
        println!("   Input: {:?}, Output: {:?}", 
                input_config.sample_format(), output_config.sample_format());
    }
    
    println!("=====================================\n");
    Ok(())
}

fn input_thread(
    reverb_clone: Arc<Mutex<Freeverb>>, 
    distortion_clone: Arc<Mutex<Distortion>>, 
    effect_selection: Arc<AtomicBool>,
    running_clone: Arc<AtomicBool>
) {
    let stdin = std::io::stdin();
    let mut buffer = String::new();
    
    println!("\nControls:");
    println!("=== Reverb Controls (activate with any reverb parameter) ===");
    println!("  w <0-1> - Set wet level (e.g., w 0.5)");
    println!("  d <0-1> - Set dry level (e.g., d 0.5)");
    println!("  r <0-1> - Set room size (e.g., r 0.8)");
    println!("  p <0-1> - Set dampening (e.g., p 0.4)");
    println!("  x <0-1> - Set stereo width (e.g., x 0.5)");
    println!("\n=== Distortion Controls (activate with any distortion parameter) ===");
    println!("  dr <0-1> - Set drive (e.g., dr 0.5)");
    println!("  l <0-1> - Set level (e.g., l 0.5)");
    println!("  t <0-1> - Set tone (e.g., t 0.5)");
    println!("  bc <rate> <depth> - Set bit crusher params (e.g., bc 0.3 0.4)");
    println!("  soft - Switch to soft clipping");
    println!("  hard - Switch to hard clipping");
    println!("  bit - Switch to bit crusher");
    println!("  wave - Switch to wavefolder");
    println!("  over - Switch to overdrive");
    println!("\n=== Global Controls ===");
    println!("  dry - Set to dry only (no effects)");
    println!("  pass - Switch to passthrough mode");
    println!("  q - Quit");
    
    while running_clone.load(Ordering::Relaxed) {
        buffer.clear();
        if stdin.read_line(&mut buffer).is_ok() {
            let input = buffer.trim();
            let parts: Vec<&str> = input.split_whitespace().collect();
            
            if parts.len() == 2 {
                let value: Result<f64, _> = parts[1].parse();
                if let Ok(val) = value {
                    let val = val.clamp(0.0, 1.0);
                    
                    match parts[0] {
                        // Reverb controls - activate reverb
                        "w" => {
                            effect_selection.store(true, Ordering::Relaxed);
                            let mut reverb_guard = reverb_clone.lock().unwrap();
                            reverb_guard.set_wet(val);
                            println!("Reverb activated - Wet level set to {:.2}, Effect selection: {}", val, effect_selection.load(Ordering::Relaxed));
                        }
                        "d" => {
                            effect_selection.store(true, Ordering::Relaxed);
                            let mut reverb_guard = reverb_clone.lock().unwrap();
                            reverb_guard.set_dry(val);
                            println!("Reverb activated - Dry level set to {:.2}", val);
                        }
                        "r" => {
                            effect_selection.store(true, Ordering::Relaxed);
                            let mut reverb_guard = reverb_clone.lock().unwrap();
                            reverb_guard.set_room_size(val);
                            println!("Reverb activated - Room size set to {:.2}", val);
                        }
                        "p" => {
                            effect_selection.store(true, Ordering::Relaxed);
                            let mut reverb_guard = reverb_clone.lock().unwrap();
                            reverb_guard.set_dampening(val);
                            println!("Reverb activated - Dampening set to {:.2}", val);
                        }
                        "x" => {
                            effect_selection.store(true, Ordering::Relaxed);
                            let mut reverb_guard = reverb_clone.lock().unwrap();
                            reverb_guard.set_width(val);
                            println!("Reverb activated - Stereo width set to {:.2}", val);
                        }
                        // Distortion controls - activate distortion
                        "dr" => {
                            effect_selection.store(false, Ordering::Relaxed);
                            let mut distortion_guard = distortion_clone.lock().unwrap();
                            distortion_guard.set_drive(val);
                            println!("Distortion activated - Drive set to {:.2}, Effect selection: {}", val, effect_selection.load(Ordering::Relaxed));
                        }
                        "l" => {
                            effect_selection.store(false, Ordering::Relaxed);
                            let mut distortion_guard = distortion_clone.lock().unwrap();
                            distortion_guard.set_level(val);
                            println!("Distortion activated - Level set to {:.2}", val);
                        }
                        "t" => {
                            effect_selection.store(false, Ordering::Relaxed);
                            let mut distortion_guard = distortion_clone.lock().unwrap();
                            distortion_guard.set_tone(val);
                            println!("Distortion activated - Tone set to {:.2}", val);
                        }
                        _ => {}
                    }
                }
            } else if parts.len() == 3 && parts[0] == "bc" {
                // Bit crusher parameters (rate and depth)
                let rate: Result<f64, _> = parts[1].parse();
                let depth: Result<f64, _> = parts[2].parse();
                
                if let (Ok(rate_val), Ok(depth_val)) = (rate, depth) {
                    effect_selection.store(false, Ordering::Relaxed);
                    let mut distortion_guard = distortion_clone.lock().unwrap();
                    distortion_guard.set_distortion_type(DistortionType::BitCrusher);
                    distortion_guard.set_bit_crusher_params(rate_val, depth_val);
                    println!("Distortion activated - Bit crusher: rate={:.2}, depth={:.2}", rate_val, depth_val);
                }
            } else if parts.len() == 1 {
                match parts[0] {
                    // Distortion type selection
                    "soft" => {
                        effect_selection.store(false, Ordering::Relaxed);
                        let mut distortion_guard = distortion_clone.lock().unwrap();
                        distortion_guard.set_distortion_type(DistortionType::Soft);
                        println!("Distortion activated - Soft clipping selected");
                    }
                    "hard" => {
                        effect_selection.store(false, Ordering::Relaxed);
                        let mut distortion_guard = distortion_clone.lock().unwrap();
                        distortion_guard.set_distortion_type(DistortionType::Hard);
                        println!("Distortion activated - Hard clipping selected");
                    }
                    "bit" => {
                        effect_selection.store(false, Ordering::Relaxed);
                        let mut distortion_guard = distortion_clone.lock().unwrap();
                        distortion_guard.set_distortion_type(DistortionType::BitCrusher);
                        println!("Distortion activated - Bit crusher selected");
                    }
                    "wave" => {
                        effect_selection.store(false, Ordering::Relaxed);
                        let mut distortion_guard = distortion_clone.lock().unwrap();
                        distortion_guard.set_distortion_type(DistortionType::Wavefolder);
                        println!("Distortion activated - Wavefolder selected");
                    }
                    "over" => {
                        effect_selection.store(false, Ordering::Relaxed);
                        let mut distortion_guard = distortion_clone.lock().unwrap();
                        distortion_guard.set_distortion_type(DistortionType::Overdrive);
                        println!("Distortion activated - Overdrive selected");
                    }
                    // Global controls
                    "dry" => {
                        let mut reverb_guard = reverb_clone.lock().unwrap();
                        reverb_guard.set_wet(0.0);
                        reverb_guard.set_dry(1.0);
                        let mut distortion_guard = distortion_clone.lock().unwrap();
                        distortion_guard.set_level(0.0);
                        println!("Set to dry only (no effects)");
                    }
                    "pass" => {
                        let mut reverb_guard = reverb_clone.lock().unwrap();
                        reverb_guard.set_wet(0.0);
                        reverb_guard.set_dry(1.0);
                        reverb_guard.set_room_size(0.0);
                        reverb_guard.set_dampening(0.0);
                        reverb_guard.set_width(0.5);
                        let mut distortion_guard = distortion_clone.lock().unwrap();
                        distortion_guard.set_level(0.0);
                        println!("Switched to passthrough mode (no effects)");
                    }
                    "q" => {
                        running_clone.store(false, Ordering::Relaxed);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the default host
    let host = cpal::default_host();

    // Get the default input and output devices
    let input_device = host.default_input_device()
        .ok_or("No input device found")?;
    let output_device = host.default_output_device()
        .ok_or("No output device found")?;

    println!("Input device: {}", input_device.name()?);
    println!("Output device: {}", output_device.name()?);

    // Get the default input and output configs
    let input_config = input_device.default_input_config().unwrap();
    let output_config = output_device.default_output_config().unwrap();

    println!("Input config: {:?}", input_config);
    println!("Output config: {:?}", output_config);
    
    // Print detailed device configuration
    print_device_config(&input_device, &output_device, &input_config, &output_config)?;

    // Create ring buffers for audio data
    let ring_buffer = RingBuffer::<f32>::new(8192);
    let (producer, consumer) = ring_buffer.split();

    // Create reverb instance
    let sample_rate = output_config.sample_rate().0 as usize;
    let reverb = Arc::new(Mutex::new(Freeverb::new(sample_rate)));
    let distortion = Arc::new(Mutex::new(Distortion::new(sample_rate)));
    
    // Configure reverb settings - start with mostly dry signal
    {
        let mut reverb_guard = reverb.lock().unwrap();
        reverb_guard.set_wet(0.1);      // 10% wet signal (start conservative)
        reverb_guard.set_dry(0.9);      // 90% dry signal
        reverb_guard.set_room_size(0.5); // Medium room
        reverb_guard.set_dampening(0.5); // Moderate dampening
        reverb_guard.set_width(0.5);    // Stereo width
    }
    // Configure distortion settings
    {
        let mut distortion_guard = distortion.lock().unwrap();
        distortion_guard.set_distortion_type(DistortionType::Soft);
        distortion_guard.set_drive(0.5);
        distortion_guard.set_level(0.8);
        distortion_guard.set_tone(0.5);
        distortion_guard.set_bit_crusher_params(0.1, 0.5);
    }

    // Effect selection state (true = reverb active, false = distortion active)
    let effect_selection = Arc::new(AtomicBool::new(true)); // Start with reverb
    let effect_selection_clone = effect_selection.clone();

    // Flag to control the audio processing
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Spawn a thread to handle user input for real-time parameter adjustment
    let reverb_clone = reverb.clone();
    let distortion_clone = distortion.clone();
    thread::spawn(move || input_thread(reverb_clone, distortion_clone, effect_selection_clone, running_clone));

    // Build the input stream
    let input_stream = build_input_stream(
        input_device,
        input_config,
        producer,
        running.clone(),
    )?;

    // Build the output stream
    let output_stream = build_output_stream(
        output_device,
        output_config,
        consumer,
        reverb.clone(),
        distortion.clone(),
        effect_selection.clone(),
        running.clone(),
    )?;

    // Play the streams
    input_stream.play()?;
    output_stream.play()?;

    // Wait for the user to stop the program
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }

    println!("\nShutting down...");
    Ok(())
}

fn build_input_stream(
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    mut producer: Producer<f32>,
    running: Arc<AtomicBool>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
    let err_fn = |err| eprintln!("Input stream error: {}", err);
    
    // Capture channel count for the callback
    let input_channels = config.channels() as usize;

    let stream = match config.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !running.load(Ordering::Relaxed) {
                    return;
                }
                
                // Process audio based on actual channel configuration
                let samples = if input_channels == 1 {
                    // Mono input - direct processing
                    data.to_vec()
                } else if input_channels == 2 {
                    // Stereo input - convert to mono by averaging
                    data.chunks(2)
                        .map(|chunk| (chunk[0] + chunk[1]) * 0.5)
                        .collect()
                } else {
                    // Multi-channel input - average all channels
                    data.chunks(input_channels)
                        .map(|chunk| chunk.iter().sum::<f32>() / input_channels as f32)
                        .collect()
                };

                for &sample in &samples {
                    if producer.push(sample).is_err() {
                        // Buffer is full, skip this sample
                        break;
                    }
                }
            },
            err_fn,
        )?,
        SampleFormat::I16 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if !running.load(Ordering::Relaxed) {
                    return;
                }
                
                // Convert i16 to f32 with proper channel handling
                let samples: Vec<f32> = if input_channels == 1 {
                    // Mono input - direct conversion
                    data.iter()
                        .map(|&sample| f32::from(sample) / f32::from(i16::MAX))
                        .collect()
                } else if input_channels == 2 {
                    // Stereo input - convert to mono by averaging
                    data.chunks(2)
                        .map(|chunk| {
                            let left = f32::from(chunk[0]) / f32::from(i16::MAX);
                            let right = f32::from(chunk[1]) / f32::from(i16::MAX);
                            (left + right) * 0.5
                        })
                        .collect()
                } else {
                    // Multi-channel input - average all channels
                    data.chunks(input_channels)
                        .map(|chunk| {
                            chunk.iter()
                                .map(|&sample| f32::from(sample) / f32::from(i16::MAX))
                                .sum::<f32>() / input_channels as f32
                        })
                        .collect()
                };

                for &sample in &samples {
                    if producer.push(sample).is_err() {
                        break;
                    }
                }
            },
            err_fn,
        )?,
        SampleFormat::U16 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                if !running.load(Ordering::Relaxed) {
                    return;
                }
                
                // Convert u16 to f32 with proper channel handling
                let samples: Vec<f32> = if input_channels == 1 {
                    // Mono input - direct conversion
                    data.iter()
                        .map(|&sample| (f32::from(sample) / f32::from(u16::MAX)) * 2.0 - 1.0)
                        .collect()
                } else if input_channels == 2 {
                    // Stereo input - convert to mono by averaging
                    data.chunks(2)
                        .map(|chunk| {
                            let left = (f32::from(chunk[0]) / f32::from(u16::MAX)) * 2.0 - 1.0;
                            let right = (f32::from(chunk[1]) / f32::from(u16::MAX)) * 2.0 - 1.0;
                            (left + right) * 0.5
                        })
                        .collect()
                } else {
                    // Multi-channel input - average all channels
                    data.chunks(input_channels)
                        .map(|chunk| {
                            chunk.iter()
                                .map(|&sample| (f32::from(sample) / f32::from(u16::MAX)) * 2.0 - 1.0)
                                .sum::<f32>() / input_channels as f32
                        })
                        .collect()
                };

                for &sample in &samples {
                    if producer.push(sample).is_err() {
                        break;
                    }
                }
            },
            err_fn,
        )?,
        _ => return Err("Unsupported sample format".into()),
    };

    Ok(stream)
}

fn build_output_stream(
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    mut consumer: Consumer<f32>,
    reverb: Arc<Mutex<Freeverb>>,
    distortion: Arc<Mutex<Distortion>>,
    effect_selection: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
    let err_fn = |err| eprintln!("Output stream error: {}", err);
    
    // Capture channel count for the callback
    let output_channels = config.channels() as usize;

    let stream = match config.sample_format() {
        SampleFormat::F32 => device.build_output_stream(
            &config.clone().into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if !running.load(Ordering::Relaxed) {
                    return;
                }

                for frame in data.chunks_mut(output_channels) {
                    // Get input sample from ring buffer
                    let input_sample = consumer.pop().unwrap_or(0.0);
                    
                    // Apply effect based on selection
                    let (left, right) = if effect_selection.load(Ordering::Relaxed) {
                        // Use reverb
                        let mut reverb_guard = reverb.lock().unwrap();
                        reverb_guard.tick((input_sample as f64, input_sample as f64))
                    } else {
                        // Use distortion
                        let mut distortion_guard = distortion.lock().unwrap();
                        distortion_guard.tick((input_sample as f64, input_sample as f64))
                    };
                    
                    // Fill output frame based on channel configuration
                    match output_channels {
                        1 => {
                            // Mono output - mix stereo reverb to mono
                            frame[0] = (left + right) as f32 * 0.5;
                        }
                        2 => {
                            // Stereo output - use reverb stereo output
                            frame[0] = left as f32;
                            frame[1] = right as f32;
                        }
                        _ => {
                            // Multi-channel output - distribute stereo reverb
                            frame[0] = left as f32;
                            frame[1] = right as f32;
                            // Duplicate stereo signal to remaining channels
                            for i in 2..frame.len() {
                                frame[i] = if i % 2 == 0 { left as f32 } else { right as f32 };
                            }
                        }
                    }
                }
            },
            err_fn,
        )?,
        SampleFormat::I16 => device.build_output_stream(
            &config.clone().into(),
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                if !running.load(Ordering::Relaxed) {
                    return;
                }

                for frame in data.chunks_mut(output_channels) {
                    let input_sample = consumer.pop().unwrap_or(0.0);
                    
                    // Apply effect based on selection
                    let (left, right) = if effect_selection.load(Ordering::Relaxed) {
                        // Use reverb
                        let mut reverb_guard = reverb.lock().unwrap();
                        reverb_guard.tick((input_sample as f64, input_sample as f64))
                    } else {
                        // Use distortion
                        let mut distortion_guard = distortion.lock().unwrap();
                        distortion_guard.tick((input_sample as f64, input_sample as f64))
                    };
                    
                    // Fill output frame based on channel configuration
                    match output_channels {
                        1 => {
                            // Mono output - mix stereo reverb to mono
                            let mono_sample = (left + right) as f32 * 0.5;
                            frame[0] = (mono_sample * f32::from(i16::MAX)) as i16;
                        }
                        2 => {
                            // Stereo output - use reverb stereo output
                            frame[0] = (left as f32 * f32::from(i16::MAX)) as i16;
                            frame[1] = (right as f32 * f32::from(i16::MAX)) as i16;
                        }
                        _ => {
                            // Multi-channel output - distribute stereo reverb
                            frame[0] = (left as f32 * f32::from(i16::MAX)) as i16;
                            frame[1] = (right as f32 * f32::from(i16::MAX)) as i16;
                            // Duplicate stereo signal to remaining channels
                            for i in 2..frame.len() {
                                let sample = if i % 2 == 0 { left as f32 } else { right as f32 };
                                frame[i] = (sample * f32::from(i16::MAX)) as i16;
                            }
                        }
                    }
                }
            },
            err_fn,
        )?,
        SampleFormat::U16 => device.build_output_stream(
            &config.clone().into(),
            move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                if !running.load(Ordering::Relaxed) {
                    return;
                }

                for frame in data.chunks_mut(output_channels) {
                    let input_sample = consumer.pop().unwrap_or(0.0);
                    
                    // Apply effect based on selection
                    let (left, right) = if effect_selection.load(Ordering::Relaxed) {
                        // Use reverb
                        let mut reverb_guard = reverb.lock().unwrap();
                        reverb_guard.tick((input_sample as f64, input_sample as f64))
                    } else {
                        // Use distortion
                        let mut distortion_guard = distortion.lock().unwrap();
                        distortion_guard.tick((input_sample as f64, input_sample as f64))
                    };
                    
                    // Fill output frame based on channel configuration
                    match output_channels {
                        1 => {
                            // Mono output - mix stereo reverb to mono
                            let mono_sample = (left + right) as f32 * 0.5;
                            let normalized = (mono_sample + 1.0) * 0.5;
                            frame[0] = (normalized * f32::from(u16::MAX)) as u16;
                        }
                        2 => {
                            // Stereo output - use reverb stereo output
                            let left_normalized = (left as f32 + 1.0) * 0.5;
                            let right_normalized = (right as f32 + 1.0) * 0.5;
                            frame[0] = (left_normalized * f32::from(u16::MAX)) as u16;
                            frame[1] = (right_normalized * f32::from(u16::MAX)) as u16;
                        }
                        _ => {
                            // Multi-channel output - distribute stereo reverb
                            let left_normalized = (left as f32 + 1.0) * 0.5;
                            let right_normalized = (right as f32 + 1.0) * 0.5;
                            frame[0] = (left_normalized * f32::from(u16::MAX)) as u16;
                            frame[1] = (right_normalized * f32::from(u16::MAX)) as u16;
                            // Duplicate stereo signal to remaining channels
                            for i in 2..frame.len() {
                                let sample = if i % 2 == 0 { left as f32 } else { right as f32 };
                                let normalized = (sample + 1.0) * 0.5;
                                frame[i] = (normalized * f32::from(u16::MAX)) as u16;
                            }
                        }
                    }
                }
            },
            err_fn,
        )?,
        _ => return Err("Unsupported sample format".into()),
    };

    Ok(stream)
}

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

// Test function for simple passthrough
fn test_passthrough() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing simple passthrough mode...");
    
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
    let input_config = input_device.default_input_config()?;
    let output_config = output_device.default_output_config()?;

    println!("Input config: {:?}", input_config);
    println!("Output config: {:?}", output_config);
    
    // Print detailed device configuration
    print_device_config(&input_device, &output_device, &input_config, &output_config)?;

    // Create ring buffers for audio data
    let ring_buffer = RingBuffer::<f32>::new(8192);
    let (producer, consumer) = ring_buffer.split();

    // Flag to control the audio processing
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Spawn a thread to handle user input
    thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut buffer = String::new();
        
        println!("\nPassthrough Test Mode:");
        println!("  Press Enter to stop");
        
        while running_clone.load(Ordering::Relaxed) {
            buffer.clear();
            if stdin.read_line(&mut buffer).is_ok() {
                if buffer.trim().is_empty() {
                    running_clone.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
    });

    // Build the input stream
    let input_stream = build_input_stream(
        input_device,
        input_config,
        producer,
        running.clone(),
    )?;

    // Build the output stream with simple passthrough
    let output_stream = build_passthrough_output_stream(
        output_device,
        output_config,
        consumer,
        running.clone(),
    )?;

    // Play the streams
    input_stream.play()?;
    output_stream.play()?;

    println!("🎤 Passthrough test started. Speak into your microphone...");
    println!("Press Enter to stop");

    // Wait for the user to stop the program
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }

    println!("\nPassthrough test completed.");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Mac Pedals - Real-time Audio Reverb");
    println!("Choose mode:");
    println!("  1: Passthrough test (no effects)");
    println!("  2: Reverb mode");
    println!("Enter choice (1 or 2): ");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => test_passthrough(),
        "2" => run_reverb_mode(),
        _ => {
            println!("Invalid choice. Running passthrough test...");
            test_passthrough()
        }
    }
}

fn run_reverb_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Starting reverb mode...");

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
    
    // Configure reverb settings - start with mostly dry signal
    {
        let mut reverb_guard = reverb.lock().unwrap();
        reverb_guard.set_wet(0.1);      // 10% wet signal (start conservative)
        reverb_guard.set_dry(0.9);      // 90% dry signal
        reverb_guard.set_room_size(0.5); // Medium room
        reverb_guard.set_dampening(0.5); // Moderate dampening
        reverb_guard.set_width(0.5);    // Stereo width
    }

    // Flag to control the audio processing
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Spawn a thread to handle user input for real-time parameter adjustment
    let reverb_clone = reverb.clone();
    thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut buffer = String::new();
        
        println!("\nControls:");
        println!("  w <0-1> - Set wet level (e.g., w 0.5)");
        println!("  d <0-1> - Set dry level (e.g., d 0.5)");
        println!("  r <0-1> - Set room size (e.g., r 0.8)");
        println!("  p <0-1> - Set dampening (e.g., p 0.4)");
        println!("  x <0-1> - Set stereo width (e.g., x 0.5)");
        println!("  dry - Set to dry only (no reverb)");
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
                        let mut reverb_guard = reverb_clone.lock().unwrap();
                        
                        match parts[0] {
                            "w" => {
                                reverb_guard.set_wet(val);
                                println!("Wet level set to {:.2}", val);
                            }
                            "d" => {
                                reverb_guard.set_dry(val);
                                println!("Dry level set to {:.2}", val);
                            }
                            "r" => {
                                reverb_guard.set_room_size(val);
                                println!("Room size set to {:.2}", val);
                            }
                            "p" => {
                                reverb_guard.set_dampening(val);
                                println!("Dampening set to {:.2}", val);
                            }
                            "x" => {
                                reverb_guard.set_width(val);
                                println!("Stereo width set to {:.2}", val);
                            }
                            _ => {}
                        }
                    }
                } else if parts.len() == 1 {
                    match parts[0] {
                        "dry" => {
                            let mut reverb_guard = reverb_clone.lock().unwrap();
                            reverb_guard.set_wet(0.0);
                            reverb_guard.set_dry(1.0);
                            println!("Set to dry only (no reverb)");
                        }
                        "q" => {
                            running_clone.store(false, Ordering::Relaxed);
                            break;
                        }
                        _ => {}
                    }
                } else if parts.len() == 1 && parts[0] == "q" {
                    running_clone.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
    });

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

fn build_passthrough_output_stream(
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    mut consumer: Consumer<f32>,
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
                    
                    // Fill output frame based on channel configuration
                    match output_channels {
                        1 => {
                            // Mono output
                            frame[0] = input_sample;
                        }
                        2 => {
                            // Stereo output - duplicate mono to both channels
                            frame[0] = input_sample;
                            frame[1] = input_sample;
                        }
                        _ => {
                            // Multi-channel output - duplicate to all channels
                            for sample in frame.iter_mut() {
                                *sample = input_sample;
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
                    let i16_sample = (input_sample * f32::from(i16::MAX)) as i16;
                    
                    // Fill output frame based on channel configuration
                    match output_channels {
                        1 => {
                            frame[0] = i16_sample;
                        }
                        2 => {
                            frame[0] = i16_sample;
                            frame[1] = i16_sample;
                        }
                        _ => {
                            for sample in frame.iter_mut() {
                                *sample = i16_sample;
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
                    let normalized = (input_sample + 1.0) * 0.5;
                    let u16_sample = (normalized * f32::from(u16::MAX)) as u16;
                    
                    // Fill output frame based on channel configuration
                    match output_channels {
                        1 => {
                            frame[0] = u16_sample;
                        }
                        2 => {
                            frame[0] = u16_sample;
                            frame[1] = u16_sample;
                        }
                        _ => {
                            for sample in frame.iter_mut() {
                                *sample = u16_sample;
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

fn build_output_stream(
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    mut consumer: Consumer<f32>,
    reverb: Arc<Mutex<Freeverb>>,
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

                let mut reverb_guard = reverb.lock().unwrap();
                
                for frame in data.chunks_mut(output_channels) {
                    // Get input sample from ring buffer
                    let input_sample = consumer.pop().unwrap_or(0.0);
                    
                    // Apply reverb effect
                    let (left, right) = reverb_guard.tick((input_sample as f64, input_sample as f64));
                    
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

                let mut reverb_guard = reverb.lock().unwrap();
                
                for frame in data.chunks_mut(output_channels) {
                    let input_sample = consumer.pop().unwrap_or(0.0);
                    let (left, right) = reverb_guard.tick((input_sample as f64, input_sample as f64));
                    
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

                let mut reverb_guard = reverb.lock().unwrap();
                
                for frame in data.chunks_mut(output_channels) {
                    let input_sample = consumer.pop().unwrap_or(0.0);
                    let (left, right) = reverb_guard.tick((input_sample as f64, input_sample as f64));
                    
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

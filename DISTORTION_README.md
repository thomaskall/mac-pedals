# Distortion Module for Mac Pedals

A guitar distortion effect module for the Mac Pedals audio processing application, built in Rust. This module provides various distortion algorithms commonly used in guitar effects pedals, following the same pattern as the freeverb library.

## Features

### Distortion Types

1. **Soft Clipping** - Tube-like distortion using hyperbolic tangent function
2. **Hard Clipping** - Digital clipping with adjustable threshold
3. **Bit Crusher** - Lo-fi digital distortion with sample rate and bit depth reduction
4. **Wavefolder** - Complex harmonic distortion using wave folding
5. **Overdrive** - Asymmetric clipping for warm, musical distortion

### Controls

- **Drive** (0.0 - 1.0) - Controls the amount of distortion/gain
- **Level** (0.0 - 1.0) - Output volume control
- **Tone** (0.0 - 1.0) - High-frequency filter control
- **Bit Crusher Parameters** - Rate and depth for bit crusher effect

### Technical Features

- **Stereo Processing** - Processes left and right channels independently
- **Real-time Parameter Adjustment** - Thread-safe parameter changes
- **DC Blocking Filter** - Removes DC offset from the signal
- **Tone Filter** - Adjustable high-pass filter for tone shaping
- **Sample Rate Independent** - Works at any sample rate

## Usage

### Basic Usage

```rust
use distortion::{Distortion, DistortionType};

// Create a new distortion processor
let mut distortion = Distortion::new(44100);

// Configure the effect
distortion.set_distortion_type(DistortionType::Soft);
distortion.set_drive(0.6);
distortion.set_level(0.8);
distortion.set_tone(0.5);

// Process audio samples
let (left_out, right_out) = distortion.tick((input_left, input_right));
```

### Integration with Main Application

The distortion module follows the same pattern as the freeverb library used in the main application:

```rust
// Create and configure (similar to reverb in main.rs)
let distortion = Arc::new(Mutex::new(Distortion::new(sample_rate)));
{
    let mut distortion_guard = distortion.lock().unwrap();
    distortion_guard.set_distortion_type(DistortionType::Soft);
    distortion_guard.set_drive(0.5);
    distortion_guard.set_level(0.7);
}

// In audio callback (similar to reverb.tick() in main.rs)
let mut distortion_guard = distortion.lock().unwrap();
let (left, right) = distortion_guard.tick((input_sample as f64, input_sample as f64));
```

### Real-time Parameter Control

```rust
// Thread-safe parameter adjustment
let mut distortion_guard = distortion.lock().unwrap();
distortion_guard.set_drive(0.8);
distortion_guard.set_distortion_type(DistortionType::Overdrive);
distortion_guard.set_bit_crusher_params(0.3, 0.4);
```

## Distortion Types Explained

### 1. Soft Clipping (Tube-like)
- **Algorithm**: Hyperbolic tangent function (`tanh`)
- **Characteristic**: Smooth, musical clipping similar to vacuum tube amplifiers
- **Best for**: Warm, natural distortion
- **Drive Range**: 0.0 (clean) to 1.0 (heavy distortion)

### 2. Hard Clipping
- **Algorithm**: Threshold-based clipping with adjustable limits
- **Characteristic**: Sharp, digital clipping with defined limits
- **Best for**: Aggressive, high-gain sounds
- **Drive Range**: 0.0 (clean) to 1.0 (maximum clipping)

### 3. Bit Crusher
- **Algorithm**: Sample rate and bit depth reduction
- **Characteristic**: Lo-fi, digital degradation effect
- **Best for**: Retro, 8-bit style sounds
- **Parameters**: 
  - Rate (0.01 - 1.0): Sample rate reduction factor
  - Depth (0.1 - 1.0): Bit depth reduction factor

### 4. Wavefolder
- **Algorithm**: Sine wave folding with adjustable fold amount
- **Characteristic**: Complex harmonic generation
- **Best for**: Experimental, synth-like sounds
- **Drive Range**: 0.0 (clean) to 1.0 (maximum folding)

### 5. Overdrive
- **Algorithm**: Asymmetric clipping with different positive/negative thresholds
- **Characteristic**: Warm, musical distortion with harmonic richness
- **Best for**: Classic rock and blues tones
- **Drive Range**: 0.0 (clean) to 1.0 (maximum overdrive)

## Examples

### Basic Example
```rust
let mut distortion = Distortion::new(44100);
distortion.set_distortion_type(DistortionType::Soft);
distortion.set_drive(0.7);
distortion.set_level(0.8);

// Process a test signal
let test_input = 0.5;
let (left, right) = distortion.tick((test_input, test_input));
println!("Input: {}, Output: ({}, {})", test_input, left, right);
```

### Effect Chain Example
```rust
// Create multiple effects
let mut distortion = Distortion::new(44100);
let mut reverb = Freeverb::new(44100);

// Configure effects
distortion.set_distortion_type(DistortionType::Overdrive);
distortion.set_drive(0.6);
reverb.set_wet(0.3);
reverb.set_dry(0.7);

// Process through chain
let input = 0.5;
let (dist_left, dist_right) = distortion.tick((input, input));
let (reverb_left, reverb_right) = reverb.tick((dist_left, dist_right));
```

### Real-time Control Example
```rust
use std::sync::{Arc, Mutex};
use std::thread;

let distortion = Arc::new(Mutex::new(Distortion::new(44100)));
let distortion_clone = distortion.clone();

// Spawn control thread
thread::spawn(move || {
    let stdin = std::io::stdin();
    let mut buffer = String::new();
    
    loop {
        buffer.clear();
        if stdin.read_line(&mut buffer).is_ok() {
            let input = buffer.trim();
            if input.starts_with("drive ") {
                if let Ok(drive) = input[6..].parse::<f64>() {
                    let mut guard = distortion_clone.lock().unwrap();
                    guard.set_drive(drive);
                    println!("Drive set to {}", drive);
                }
            }
        }
    }
});
```

## Performance Considerations

- **CPU Usage**: Minimal overhead, suitable for real-time processing
- **Latency**: Zero additional latency beyond the processing time
- **Memory**: Small memory footprint (~100 bytes per instance)
- **Thread Safety**: Uses mutex for thread-safe parameter adjustment

## Integration with Main Application

To integrate the distortion module into the main Mac Pedals application:

1. **Replace Reverb**: Use distortion instead of reverb in the audio callback
2. **Add Effect Selection**: Allow switching between reverb and distortion
3. **Effect Chain**: Use both effects in series (distortion → reverb)
4. **Parallel Processing**: Process different frequency bands with different effects

### Example Integration in build_output_stream

```rust
// In build_output_stream function, replace reverb processing with:
let mut distortion_guard = distortion.lock().unwrap();
let (left, right) = distortion_guard.tick((input_sample as f64, input_sample as f64));

// Fill output frame
frame[0] = left as f32;
frame[1] = right as f32;
```

## Testing

Run the included tests:

```bash
# Run unit tests
cargo test

# Run the basic example
cd examples
rustc distortion_example.rs --extern distortion=../src/distortion.rs
./distortion_example

# Run the integration example
rustc distortion_integration.rs --extern distortion=../src/distortion.rs
./distortion_integration
```

## Future Enhancements

- **More Distortion Types**: Fuzz, octave fuzz, ring modulation
- **Advanced Filtering**: Multi-band processing, EQ controls
- **Preset System**: Save and load distortion settings
- **MIDI Control**: Real-time parameter control via MIDI
- **GUI Integration**: Visual parameter controls

## License

This module is part of the Mac Pedals project and follows the same license terms. 
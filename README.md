# Mac Pedals - Real-time Audio Processing

A Rust-based real-time audio processing application that captures input audio from your microphone and applies various effects before outputting to your speakers.

## Features

- **Real-time audio passthrough** - Capture and output audio with minimal latency
- **Interactive effects control** - Change audio effects at runtime via keyboard input
- **Multiple audio effects**:
  - Passthrough (no effect)
  - Gain (amplification)
  - Distortion (soft clipping)
  - Low-pass filter
  - High-pass filter
- **Cross-platform audio I/O** using CPAL
- **Lock-free ring buffer** for efficient sample transfer

## Prerequisites

- Rust (latest stable version)
- Audio input device (microphone)
- Audio output device (speakers/headphones)

## Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd mac-pedals
```

2. Build the project:
```bash
cargo build --release
```

## Usage

1. Run the application:
```bash
cargo run --release
```

2. The application will start and display your audio configuration:
```
Audio format: F32
Sample rate: 48000
Channels: 2
Starting audio passthrough...
```

3. Use the interactive menu to select audio effects:
```
=== Audio Effects Control ===
1: Passthrough (no effect)
2: Gain (amplify)
3: Distortion
4: Low-pass filter
5: High-pass filter
q: Quit
============================

Select effect (1-5, q):
```

4. Type a number (1-5) to change effects, or 'q' to quit.

## Audio Effects Explained

### 1. Passthrough
- No audio processing applied
- Useful for testing audio routing

### 2. Gain
- Amplifies the audio signal by 2x
- Be careful with high input levels to avoid clipping

### 3. Distortion
- Applies soft clipping distortion
- Creates harmonic overtones for guitar-like effects

### 4. Low-pass Filter
- Removes high frequencies above 1kHz
- Creates a "muffled" or "warm" sound

### 5. High-pass Filter
- Removes low frequencies below 1kHz
- Useful for removing rumble or focusing on high frequencies

## Architecture

The application uses a producer-consumer pattern with a ring buffer:

1. **Input Stream**: Captures audio from microphone
2. **Audio Processor**: Applies selected effects to samples
3. **Ring Buffer**: Transfers processed samples between threads
4. **Output Stream**: Plays processed audio to speakers

## Extending the Project

### Adding New Effects

To add a new audio effect:

1. Add a new variant to the `AudioEffect` enum:
```rust
enum AudioEffect {
    // ... existing effects
    Reverb(f32),  // New effect
}
```

2. Implement the processing logic in the `process` method:
```rust
AudioEffect::Reverb(delay) => {
    // Implement reverb algorithm
    sample + self.last_sample * delay
}
```

3. Add the effect to the interactive menu in `interactive_control()`.

### Advanced DSP

For more sophisticated audio processing, consider using the `dasp` crate which is already included as a dependency. It provides:

- Signal generators
- Filters
- Envelopes
- Interpolation
- And more

## Troubleshooting

### No Audio Input/Output
- Check your system's audio permissions
- Ensure microphone and speakers are properly connected
- Try different audio devices in your system settings

### High Latency
- Use `--release` build for better performance
- Reduce the ring buffer size (currently 32,768 samples)
- Check for other audio applications that might be using the audio device

### Audio Artifacts
- Ensure your input levels aren't too high
- Check for sample rate mismatches
- Verify your audio device supports the selected format

## Dependencies

- `cpal` - Cross-platform audio I/O
- `ringbuf` - Lock-free ring buffer
- `dasp` - Digital audio signal processing
- `anyhow` - Error handling

## License

[Add your license here]

## Contributing

[Add contribution guidelines here] 
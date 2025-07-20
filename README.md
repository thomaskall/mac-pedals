# Mac Pedals - Real-time Audio Reverb

A real-time audio passthrough application with reverb effects for macOS, built in Rust using the freeverb algorithm and cpal for cross-platform audio I/O.

## Features

- **Real-time audio processing**: Live audio input from your Mac's microphone or audio interface
- **Reverb effects**: High-quality reverb using the freeverb algorithm
- **Interactive controls**: Real-time parameter adjustment via command line
- **Cross-platform audio**: Uses cpal for robust audio I/O
- **Low latency**: Optimized for real-time performance

## Requirements

- macOS (tested on macOS 14.3.0)
- Rust toolchain (install via [rustup](https://rustup.rs/))
- Audio input device (microphone, audio interface, etc.)
- Audio output device (speakers, headphones, etc.)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd mac-pedals
```

2. Build the application:
```bash
cargo build --release
```

## Usage

1. Run the application:
```bash
./target/release/mac-pedals
```

2. The application will automatically detect your default audio input and output devices and start processing audio.

3. Use the interactive controls to adjust reverb parameters in real-time:

### Controls

- `w <0-1>` - Set wet level (reverb amount, e.g., `w 0.5`)
- `d <0-1>` - Set dry level (original signal amount, e.g., `d 0.5`)
- `r <0-1>` - Set room size (reverb space size, e.g., `r 0.8`)
- `p <0-1>` - Set dampening (high-frequency decay, e.g., `p 0.4`)
- `x <0-1>` - Set stereo width (stereo spread, e.g., `x 0.5`)
- `q` - Quit the application

### Example Usage

```bash
# Start with default settings
./target/release/mac-pedals

# In the interactive console:
w 0.3    # Set wet level to 30%
d 0.7    # Set dry level to 70%
r 0.9    # Set room size to 90% (large room)
p 0.2    # Set dampening to 20% (bright reverb)
x 0.8    # Set stereo width to 80% (wide stereo)
q        # Quit
```

## Default Settings

The application starts with these default reverb settings:
- **Wet Level**: 30% (reverb signal)
- **Dry Level**: 70% (original signal)
- **Room Size**: 80% (large room)
- **Dampening**: 40% (moderate high-frequency decay)
- **Stereo Width**: 50% (balanced stereo spread)

## Technical Details

### Architecture

- **Audio I/O**: Uses cpal for cross-platform audio handling
- **Reverb Algorithm**: Implements the freeverb algorithm with 8 comb filters and 4 all-pass filters
- **Buffer Management**: Uses ring buffers for efficient audio data transfer
- **Real-time Processing**: Thread-safe parameter adjustment with mutex-protected reverb instance

### Audio Processing Pipeline

1. **Input**: Audio captured from default input device
2. **Format Conversion**: Automatic conversion between different sample formats (F32, I16, U16)
3. **Stereo to Mono**: Converts stereo input to mono for processing
4. **Reverb Processing**: Applies freeverb algorithm with current parameters
5. **Output**: Sends processed stereo audio to default output device

### Performance

- **Latency**: Optimized for low-latency real-time processing
- **CPU Usage**: Efficient implementation with minimal CPU overhead
- **Buffer Size**: 8192 samples ring buffer for smooth audio flow

## Troubleshooting

### No Audio Input/Output
- Check that your audio devices are properly connected and set as default
- Ensure microphone permissions are granted to the terminal application
- Try running with different audio devices if available

### High Latency
- Close other audio applications that might be using the audio devices
- Check system audio settings for buffer size and sample rate
- Ensure no other applications are processing audio in real-time

### Audio Distortion
- Reduce the wet level if the reverb is too strong
- Check input levels to ensure they're not clipping
- Adjust room size and dampening for better sound quality

## Development

### Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

### Dependencies

- `cpal`: Cross-platform audio I/O
- `ringbuf`: Lock-free ring buffer for audio data
- `dasp`: Digital audio signal processing utilities
- `anyhow`: Error handling
- `freeverb`: Custom reverb implementation

### Project Structure

```
mac-pedals/
├── src/
│   └── main.rs          # Main application
├── freeverb/            # Reverb algorithm implementation
│   ├── src/
│   │   ├── lib.rs
│   │   ├── freeverb.rs
│   │   ├── comb.rs
│   │   ├── all_pass.rs
│   │   └── delay_line.rs
│   └── Cargo.toml
├── Cargo.toml           # Project dependencies
└── README.md           # This file
```

## License

This project is open source. See the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests. 
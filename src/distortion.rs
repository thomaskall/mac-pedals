/// Guitar distortion effect module
/// 
/// This module provides various distortion algorithms commonly used in guitar effects pedals.
/// It follows the same pattern as the freeverb library with a tick() function for processing.
/// 
/// IMPORTANT: Only ONE distortion effect is applied at a time. Use set_distortion_type()
/// to choose which effect to apply. The tick() function will apply the selected effect
/// to the input signal.

use std::f64::consts::PI;

/// Distortion types available
#[derive(Debug, Clone, Copy)]
pub enum DistortionType {
    /// Soft clipping using tanh function (tube-like)
    Soft,
    /// Hard clipping with adjustable threshold
    Hard,
    /// Bit crusher effect
    BitCrusher,
    /// Wavefolder distortion
    Wavefolder,
    /// Overdrive with asymmetric clipping
    Overdrive,
}

/// Main distortion processor
pub struct Distortion {
    /// Type of distortion to apply
    distortion_type: DistortionType,
    /// Drive amount (0.0 to 1.0)
    drive: f64,
    /// Output level (0.0 to 1.0)
    level: f64,
    /// Tone control (0.0 to 1.0, affects high frequency content)
    tone: f64,
    /// Sample rate for internal processing
    sample_rate: f64,
    /// DC blocking filter state
    dc_blocker: [f64; 2],
    /// Tone filter state
    tone_filter: [f64; 2],
    /// Bit crusher sample rate divider
    bit_crusher_counter: f64,
    /// Bit crusher sample rate
    bit_crusher_rate: f64,
    /// Bit crusher bit depth
    bit_crusher_depth: f64,
    /// Last sample for bit crusher
    last_sample: f64,
}

impl Distortion {
    /// Create a new distortion processor
    pub fn new(sample_rate: usize) -> Self {
        Self {
            distortion_type: DistortionType::Soft,
            drive: 0.5, // Drive parameter (0.0 to 1.0)
            level: 0.7,
            tone: 0.5,
            sample_rate: sample_rate as f64,
            dc_blocker: [0.0; 2],
            tone_filter: [0.0; 2],
            bit_crusher_counter: 0.0,
            bit_crusher_rate: 0.1,
            bit_crusher_depth: 0.5,
            last_sample: 0.0,
        }
    }

    /// Process a stereo input sample and return stereo output
    /// 
    /// This function applies the currently selected distortion effect (set via set_distortion_type())
    /// to the input signal. Only ONE effect is applied at a time.
    /// 
    /// # Arguments
    /// * `input` - Tuple of (left, right) input samples as f64
    /// 
    /// # Returns
    /// * Tuple of (left, right) output samples as f64
    pub fn tick(&mut self, input: (f64, f64)) -> (f64, f64) {
        let (left_in, right_in) = input;
        
        // Apply drive gain (convert drive parameter to actual gain)
        let drive_gain = self.calculate_drive_gain();
        let left_driven = left_in * drive_gain;
        let right_driven = right_in * drive_gain;
        
        // Apply distortion based on type (only ONE effect at a time)
        let left_distorted = self.apply_distortion(left_driven);
        let right_distorted = self.apply_distortion(right_driven);
        
        // Apply tone filter
        let left_toned = self.apply_tone_filter(left_distorted);
        let right_toned = self.apply_tone_filter(right_distorted);
        
        // Apply DC blocking filter
        let left_dc_blocked = self.apply_dc_blocker(left_toned);
        let right_dc_blocked = self.apply_dc_blocker(right_toned);
        
        // Apply output level
        let left_out = left_dc_blocked * self.level;
        let right_out = right_dc_blocked * self.level;
        
        (left_out, right_out)
    }

    /// Set the distortion type
    pub fn set_distortion_type(&mut self, distortion_type: DistortionType) {
        self.distortion_type = distortion_type;
    }

    /// Set the drive amount (0.0 to 1.0)
    pub fn set_drive(&mut self, drive: f64) {
        self.drive = drive.clamp(0.0, 1.0);
    }

    /// Set the output level (0.0 to 1.0)
    pub fn set_level(&mut self, level: f64) {
        self.level = level.clamp(0.0, 1.0);
    }

    /// Set the tone control (0.0 to 1.0)
    pub fn set_tone(&mut self, tone: f64) {
        self.tone = tone.clamp(0.0, 1.0);
    }

    /// Set bit crusher parameters
    pub fn set_bit_crusher_params(&mut self, rate: f64, depth: f64) {
        self.bit_crusher_rate = rate.clamp(0.01, 1.0);
        self.bit_crusher_depth = depth.clamp(0.1, 1.0);
    }

    /// Calculate drive gain based on drive setting
    fn calculate_drive_gain(&self) -> f64 {
        // Drive ranges from 1.0 (no drive) to 20.0 (high drive)
        1.0 + (self.drive * 19.0)
    }

    /// Apply the selected distortion algorithm
    fn apply_distortion(&mut self, input: f64) -> f64 {
        match self.distortion_type {
            DistortionType::Soft => self.soft_clip(input),
            DistortionType::Hard => self.hard_clip(input),
            DistortionType::BitCrusher => self.bit_crush(input),
            DistortionType::Wavefolder => self.wavefold(input),
            DistortionType::Overdrive => self.overdrive(input),
        }
    }

    /// Soft clipping using hyperbolic tangent (tube-like)
    fn soft_clip(&self, input: f64) -> f64 {
        input.tanh()
    }

    /// Hard clipping with adjustable threshold
    fn hard_clip(&self, input: f64) -> f64 {
        let threshold = 0.5 + (self.drive * 0.5); // 0.5 to 1.0
        if input > threshold {
            threshold
        } else if input < -threshold {
            -threshold
        } else {
            input
        }
    }

    /// Bit crusher effect
    fn bit_crush(&mut self, input: f64) -> f64 {
        self.bit_crusher_counter += self.bit_crusher_rate;
        
        if self.bit_crusher_counter >= 1.0 {
            self.bit_crusher_counter -= 1.0;
            self.last_sample = input;
        }
        
        // Quantize the sample
        let levels = (2.0_f64.powf(self.bit_crusher_depth * 16.0)) as f64;
        let quantized = (self.last_sample * levels).round() / levels;
        
        quantized
    }

    /// Wavefolder distortion
    fn wavefold(&self, input: f64) -> f64 {
        let fold_amount = 0.5 + (self.drive * 2.0); // 0.5 to 2.5
        let folded = (input * fold_amount).sin();
        folded / fold_amount
    }

    /// Overdrive with asymmetric clipping
    fn overdrive(&self, input: f64) -> f64 {
        let positive_threshold = 0.3 + (self.drive * 0.4); // 0.3 to 0.7
        let negative_threshold = 0.2 + (self.drive * 0.3); // 0.2 to 0.5
        
        if input > positive_threshold {
            positive_threshold + (input - positive_threshold) * 0.3
        } else if input < -negative_threshold {
            -negative_threshold + (input + negative_threshold) * 0.3
        } else {
            input
        }
    }

    /// Apply tone filter (simple high-pass filter)
    fn apply_tone_filter(&mut self, input: f64) -> f64 {
        // Simple first-order high-pass filter
        let cutoff = 100.0 + (self.tone * 2000.0); // 100Hz to 2.1kHz
        let rc = 1.0 / (2.0 * PI * cutoff);
        let dt = 1.0 / self.sample_rate;
        let alpha = rc / (rc + dt);
        
        let output = alpha * (self.tone_filter[0] + input - self.tone_filter[1]);
        self.tone_filter[1] = self.tone_filter[0];
        self.tone_filter[0] = input;
        
        // Mix between filtered and unfiltered signal
        let filtered = output;
        let unfiltered = input;
        
        filtered * self.tone + unfiltered * (1.0 - self.tone)
    }

    /// Apply DC blocking filter
    fn apply_dc_blocker(&mut self, input: f64) -> f64 {
        // Simple DC blocking filter
        let alpha = 0.995;
        let output = input - self.dc_blocker[0] + alpha * self.dc_blocker[1];
        self.dc_blocker[0] = input;
        self.dc_blocker[1] = output;
        output
    }

    /// Reset all internal state
    pub fn reset(&mut self) {
        self.dc_blocker = [0.0; 2];
        self.tone_filter = [0.0; 2];
        self.bit_crusher_counter = 0.0;
        self.last_sample = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distortion_creation() {
        let distortion = Distortion::new(44100);
        assert_eq!(distortion.sample_rate, 44100.0);
        assert_eq!(distortion.drive, 0.5);
        assert_eq!(distortion.level, 0.7);
    }

    #[test]
    fn test_soft_clipping() {
        let mut distortion = Distortion::new(44100);
        distortion.set_distortion_type(DistortionType::Soft);
        distortion.set_drive(1.0); // Maximum drive
        
        let (left, right) = distortion.tick((1.0, 1.0));
        assert!(left < 1.0); // Should be clipped
        assert!(right < 1.0);
        
        let (left, right) = distortion.tick((-1.0, -1.0));
        // With DC blocker and tone filter, output might be slightly different
        // but should still be reasonable (not extreme values)
        assert!(left > -2.0); // Should not be extremely negative
        assert!(right > -2.0);
    }

    #[test]
    fn test_hard_clipping() {
        let mut distortion = Distortion::new(44100);
        distortion.set_distortion_type(DistortionType::Hard);
        distortion.set_drive(1.0); // Maximum drive
        
        let (left, right) = distortion.tick((2.0, 2.0));
        assert!(left <= 1.0); // Should be hard clipped
        assert!(right <= 1.0);
    }

    #[test]
    fn test_parameter_bounds() {
        let mut distortion = Distortion::new(44100);
        
        // Test drive bounds
        distortion.set_drive(-1.0);
        assert_eq!(distortion.drive, 0.0);
        distortion.set_drive(2.0);
        assert_eq!(distortion.drive, 1.0);
        
        // Test level bounds
        distortion.set_level(-1.0);
        assert_eq!(distortion.level, 0.0);
        distortion.set_level(2.0);
        assert_eq!(distortion.level, 1.0);
        
        // Test tone bounds
        distortion.set_tone(-1.0);
        assert_eq!(distortion.tone, 0.0);
        distortion.set_tone(2.0);
        assert_eq!(distortion.tone, 1.0);
    }

    #[test]
    fn test_reset() {
        let mut distortion = Distortion::new(44100);
        
        // Process some audio to change internal state
        distortion.tick((0.5, 0.5));
        
        // Reset
        distortion.reset();
        
        // Internal state should be reset
        assert_eq!(distortion.dc_blocker, [0.0; 2]);
        assert_eq!(distortion.tone_filter, [0.0; 2]);
        assert_eq!(distortion.bit_crusher_counter, 0.0);
        assert_eq!(distortion.last_sample, 0.0);
    }
} 
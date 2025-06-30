//! Audio I/O module for SAPF
//!
//! This module provides real-time audio playback and recording capabilities
//! using the cpal audio library. It includes:
//!
//! - Audio input/output streaming  
//! - Integration with SAPF UGens and value types
//! - Multi-channel audio support
//! - Real-time buffer management

pub mod input;
pub mod output;
pub mod audio_ops;

pub use input::*;
pub use output::*;
pub use audio_ops::*;

use crate::core::{SapfError, Result};
use crate::dsp::ugen::Sample;

/// Maximum number of audio channels supported
pub const MAX_CHANNELS: usize = 32;

/// Default buffer size for audio processing
pub const DEFAULT_BUFFER_SIZE: usize = 512;

/// Audio sample rate type
pub type SampleRate = f64;

/// Audio configuration settings
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: SampleRate,
    pub buffer_size: usize,
    pub num_channels: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 96000.0,
            buffer_size: DEFAULT_BUFFER_SIZE,
            num_channels: 2,
        }
    }
}

impl AudioConfig {
    /// Create new audio configuration
    pub fn new(sample_rate: SampleRate, buffer_size: usize, num_channels: usize) -> Result<Self> {
        if num_channels == 0 {
            return Err(SapfError::InvalidArgument("num_channels must be > 0".to_string()));
        }
        if num_channels > MAX_CHANNELS {
            return Err(SapfError::InvalidArgument(
                format!("num_channels {} exceeds maximum {}", num_channels, MAX_CHANNELS)
            ));
        }
        if buffer_size == 0 {
            return Err(SapfError::InvalidArgument("buffer_size must be > 0".to_string()));
        }

        Ok(Self {
            sample_rate,
            buffer_size,
            num_channels,
        })
    }

    /// Get the nyquist frequency
    pub fn nyquist(&self) -> Sample {
        self.sample_rate / 2.0
    }

    /// Get samples per millisecond
    pub fn samples_per_ms(&self) -> Sample {
        self.sample_rate / 1000.0
    }
}
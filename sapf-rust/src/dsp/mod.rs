//! DSP (Digital Signal Processing) module
//!
//! This module implements the audio processing infrastructure for SAPF.
//! It includes unit generators (UGens), oscillators, filters, and other
//! audio processing components.

pub mod ugen;
pub mod oscillators;

pub use ugen::*;
pub use oscillators::*;
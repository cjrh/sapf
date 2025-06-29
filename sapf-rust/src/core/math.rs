//! Mathematical functions and constants for SAPF
//!
//! This module provides various mathematical functions and constants used throughout
//! the SAPF interpreter, including lookup tables for optimized trigonometric functions
//! and other mathematical operations.

use std::sync::Once;

/// Mathematical constants
pub mod constants {
    use std::f64::consts::PI;
    
    pub const TWO_PI: f64 = 2.0 * PI;
    pub const PI_2: f64 = PI / 2.0;
    pub const DEG_TO_RAD: f64 = PI / 180.0;
    pub const RAD_TO_DEG: f64 = 180.0 / PI;
    pub const MIN_TO_SECS: f64 = 60.0; // beats per minute to beats per second
    pub const SECS_TO_MIN: f64 = 1.0 / 60.0; // beats per second to beats per minute
    
    pub const LOG_001: f64 = -6.907755278982137; // ln(0.001)
    pub const LOG_01: f64 = -4.605170185988092;  // ln(0.01)
    pub const LOG_1: f64 = -2.302585092994046;   // ln(0.1)
    pub const LOG2_OVER_2: f64 = 0.3465735902799726; // ln(2)/2
    pub const RLOG2: f64 = 1.4426950408889634;  // 1/ln(2)
    pub const SQRT2: f64 = 1.4142135623730951;
    pub const RSQRT2: f64 = 0.7071067811865476; // 1/sqrt(2)
    
    // Used to truncate precision
    pub const TRUNC_FLOAT: f32 = 3.0 * (1u32 << 22) as f32;
    pub const TRUNC_DOUBLE: f64 = 3.0 * (1u64 << 51) as f64;
}

/// Sine table configuration
pub mod sine_table {
    use super::constants::TWO_PI;
    
    pub const SIZE: usize = 16384; // 144 dB SNR with linear interpolation
    pub const SIZE_4: usize = SIZE >> 2;
    pub const MASK: usize = SIZE - 1;
    pub const INV_SIZE: f64 = 1.0 / SIZE as f64;
    pub const OMEGA: f64 = TWO_PI * INV_SIZE;
    pub const INV_OMEGA: f64 = 1.0 / OMEGA;
}

/// DB to amplitude table configuration
pub mod dbamp_table {
    pub const SIZE: usize = 1500;
    pub const SCALE: f64 = 5.0;
    pub const INV_SCALE: f64 = 0.2;
    pub const OFFSET: f64 = 150.0;
}

/// Decay table configuration
pub mod decay_table {
    pub const SIZE: usize = 2000;
    pub const SCALE: f64 = 1000.0;
}

/// First order coefficient table configuration
pub mod first_order_table {
    pub const SIZE: usize = 1000;
    pub const INV_SIZE: f64 = 1.0 / SIZE as f64;
    pub const SCALE: f64 = 1000.0;
}

/// Lookup tables for optimized mathematical operations
pub struct LookupTables {
    sine_table: Vec<f64>,
    dbamp_table: Vec<f64>,
    decay_table: Vec<f64>,
    first_order_coeff_table: Vec<f64>,
}

static mut LOOKUP_TABLES: Option<LookupTables> = None;
static INIT_TABLES: Once = Once::new();

impl LookupTables {
    fn new() -> Self {
        let mut tables = LookupTables {
            sine_table: vec![0.0; sine_table::SIZE + 1],
            dbamp_table: vec![0.0; dbamp_table::SIZE + 2],
            decay_table: vec![0.0; decay_table::SIZE + 1],
            first_order_coeff_table: vec![0.0; first_order_table::SIZE + 1],
        };
        
        tables.fill_sine_table();
        tables.fill_dbamp_table();
        tables.fill_decay_table();
        tables.fill_first_order_coeff_table();
        
        tables
    }
    
    fn fill_sine_table(&mut self) {
        use sine_table::*;
        for i in 0..=SIZE {
            let phase = (i as f64) * OMEGA;
            self.sine_table[i] = phase.sin();
        }
    }
    
    fn fill_dbamp_table(&mut self) {
        use dbamp_table::*;
        for i in 0..=(SIZE + 1) {
            let db = (i as f64) * INV_SCALE - OFFSET;
            self.dbamp_table[i] = (db * 0.05 * std::f64::consts::LN_10).exp();
        }
    }
    
    fn fill_decay_table(&mut self) {
        use decay_table::*;
        for i in 0..=SIZE {
            let ratio = (i as f64) / SCALE;
            // e^(-ln(1001) * ratio) where 1001 ≈ 60dB decay
            self.decay_table[i] = (-6.908755 * ratio).exp();
        }
    }
    
    fn fill_first_order_coeff_table(&mut self) {
        use first_order_table::*;
        for i in 0..=SIZE {
            let freq = i as f64;
            // First order filter coefficient: 1 - e^(-2π * freq / sample_rate)
            // Assuming sample rate of 44100 for now
            let coeff = 1.0 - (-constants::TWO_PI * freq / 44100.0).exp();
            self.first_order_coeff_table[i] = coeff.clamp(0.0, 1.0);
        }
    }
}

/// Initialize lookup tables (called once)
pub fn init_tables() {
    INIT_TABLES.call_once(|| {
        let tables = LookupTables::new();
        unsafe {
            LOOKUP_TABLES = Some(tables);
        }
    });
}

/// Get reference to lookup tables
fn get_tables() -> &'static LookupTables {
    init_tables();
    unsafe { LOOKUP_TABLES.as_ref().unwrap() }
}

/// Linear interpolation lookup
#[inline]
pub fn lut(table: &[f64], index: usize, frac: f64) -> f64 {
    let a = table[index];
    let b = table[index + 1];
    a + frac * (b - a)
}

/// Cubic interpolated oscillator lookup
#[inline]
pub fn oscil_lut(table: &[f64], index: usize, mask: usize, x: f64) -> f64 {
    let y0 = table[(index.wrapping_sub(1)) & mask];
    let y1 = table[index & mask];
    let y2 = table[(index + 1) & mask];
    let y3 = table[(index + 2) & mask];
    
    let c0 = y1;
    let c1 = 0.5 * (y2 - y0);
    let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
    let c3 = 1.5 * (y1 - y2) + 0.5 * (y3 - y0);
    
    ((c3 * x + c2) * x + c1) * x + c0
}

/// Calculate decay factor from ratio
pub fn calc_decay(ratio: f64) -> f64 {
    let tables = get_tables();
    let abs_ratio = ratio.abs().clamp(0.0, 2.0);
    
    let findex = decay_table::SCALE * abs_ratio;
    let iindex = findex.floor();
    let result = lut(&tables.decay_table, iindex as usize, findex - iindex);
    
    if ratio < 0.0 { -result } else { result }
}

/// Convert dB gain to amplitude using lookup table
pub fn t_dbamp(dbgain: f64) -> f64 {
    let tables = get_tables();
    let clamped_gain = dbgain.clamp(-150.0, 150.0);
    let findex = dbamp_table::SCALE * (clamped_gain + dbamp_table::OFFSET);
    let iindex = findex.floor();
    lut(&tables.dbamp_table, iindex as usize, findex - iindex)
}

/// Calculate first-order filter coefficient
pub fn t_first_order_coeff(freq: f64) -> f64 {
    let tables = get_tables();
    let clamped_freq = freq.clamp(0.0, first_order_table::SCALE);
    let findex = clamped_freq;
    let iindex = findex.floor();
    lut(&tables.first_order_coeff_table, iindex as usize, findex - iindex)
}

/// Table-based sine function
pub fn tsin(x: f64) -> f64 {
    let tables = get_tables();
    let findex = sine_table::INV_OMEGA * x;
    let iindex = findex.floor();
    let index = (iindex as usize) & sine_table::MASK;
    lut(&tables.sine_table, index, findex - iindex)
}

/// Table-based cosine function
pub fn tcos(x: f64) -> f64 {
    tsin(x + std::f64::consts::FRAC_PI_2)
}

/// Table-based sine function with period of 1.0 instead of 2π
pub fn tsin1(x: f64) -> f64 {
    let tables = get_tables();
    let findex = sine_table::INV_SIZE * x;
    let iindex = findex.floor();
    let index = (iindex as usize) & sine_table::MASK;
    lut(&tables.sine_table, index, findex - iindex)
}

/// Table-based cosine function with period of 1.0 instead of 2π
pub fn tcos1(x: f64) -> f64 {
    tsin1(x + 0.25)
}

/// Fast table-based sine with integer indexing
pub fn tsinx(x: f64) -> f64 {
    let tables = get_tables();
    let iindex = x.floor();
    let index = (iindex as usize) & sine_table::MASK;
    lut(&tables.sine_table, index, x - iindex)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    
    #[test]
    fn test_constants() {
        assert_abs_diff_eq!(constants::TWO_PI, 2.0 * std::f64::consts::PI, epsilon = 1e-15);
        assert_abs_diff_eq!(constants::DEG_TO_RAD, std::f64::consts::PI / 180.0, epsilon = 1e-15);
        assert_abs_diff_eq!(constants::SQRT2, std::f64::consts::SQRT_2, epsilon = 1e-15);
    }
    
    #[test]
    fn test_lut() {
        let table = vec![0.0, 1.0, 4.0, 9.0];
        
        // Test exact indices
        assert_eq!(lut(&table, 0, 0.0), 0.0);
        assert_eq!(lut(&table, 1, 0.0), 1.0);
        
        // Test interpolation
        assert_eq!(lut(&table, 0, 0.5), 0.5);
        assert_eq!(lut(&table, 1, 0.5), 2.5);
    }
    
    #[test]
    fn test_tsin_accuracy() {
        init_tables();
        
        // Test known values
        assert_abs_diff_eq!(tsin(0.0), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(tsin(std::f64::consts::FRAC_PI_2), 1.0, epsilon = 1e-4);
        assert_abs_diff_eq!(tsin(std::f64::consts::PI), 0.0, epsilon = 1e-4);
        
        // Test against standard library with smaller range for lookup table accuracy
        for i in 0..20 {
            let x = (i as f64) * 0.1;
            assert_abs_diff_eq!(tsin(x), x.sin(), epsilon = 1e-3);
        }
    }
    
    #[test]
    fn test_tcos_accuracy() {
        init_tables();
        
        // Test known values
        assert_abs_diff_eq!(tcos(0.0), 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(tcos(std::f64::consts::FRAC_PI_2), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(tcos(std::f64::consts::PI), -1.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_dbamp() {
        init_tables();
        
        // Test 0 dB = 1.0 amplitude
        assert_abs_diff_eq!(t_dbamp(0.0), 1.0, epsilon = 1e-5);
        
        // Test -6 dB ≈ 0.5 amplitude
        assert_abs_diff_eq!(t_dbamp(-6.0), 0.5, epsilon = 1e-2);
        
        // Test -20 dB ≈ 0.1 amplitude
        assert_abs_diff_eq!(t_dbamp(-20.0), 0.1, epsilon = 1e-2);
    }
    
    #[test]
    fn test_decay() {
        init_tables();
        
        // Test decay at ratio 0 should be 1.0
        assert_abs_diff_eq!(calc_decay(0.0), 1.0, epsilon = 1e-5);
        
        // Test negative ratios
        assert_eq!(calc_decay(-0.5), -calc_decay(0.5));
    }
    
    #[test]
    fn test_tsin1_period() {
        init_tables();
        
        // Test that tsin1 has period of 1.0
        let x = 0.3;
        assert_abs_diff_eq!(tsin1(x), tsin1(x + 1.0), epsilon = 1e-4);
        assert_abs_diff_eq!(tsin1(x), tsin1(x + 2.0), epsilon = 1e-4);
    }
}
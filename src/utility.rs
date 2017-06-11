//! Provides data types and functions that are useful but not directly related
//! To synthesis
use std::f32;
use num::clamp;

/// Calculates constant power panning
/// This function uses `sin` and `cos` so use only when needed
///
/// * `pan` - the amount between -1 and 1 to pan
///
/// Returns:
/// * `(f32, f32)` - a tuple containing the raw amplitute modifier values.
/// We can directly multiply channels by these values and achieve a panning effect.
pub fn constant_power_pan(pan: f32) -> (f32, f32) {
		// clamp and convert into degrees between 0 - 90
        let pan_radians = ((clamp(pan, -1f32, 1f32) + 1f32) * 45f32).to_radians();
        let left_amp = clamp(pan_radians.cos(), 0f32, 1f32);
        let right_amp = clamp(pan_radians.sin(), 0f32, 1f32);
        
        // calculate gain for panning
        (left_amp, right_amp)
}
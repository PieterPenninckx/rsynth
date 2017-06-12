//! Provides data types and functions that are useful but not directly related
//! To synthesis
pub mod note;

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

/// Human readable names for MIDI note numbers
/// In this implementation, middle C is C5 (as opposed to C3).
/// Names are from the most common usages.  For instance,
/// We use `EFlat` instead of `DSharp`, although they are the same.
/// Because we are defining notes based on MIDI, this means that there
/// is `u8` max possible values, or 127 including 0, as opposed to a regular
/// 88 on a common piano.
/// 
/// List of note names and their equivalents:
///
/// * `C`
/// * `CSharp` - the same as D flat
/// * `D`
/// * `EFlat` - the same as D sharp
/// * `F`
/// * `FSharp` - the same as G flat
/// * `G`
/// * `AFlat` - the same as G sharp
/// * `A`
/// * `BFlat` - the same as A sharp
/// * `B`
pub enum MIDINote {

}
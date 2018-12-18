use dsp::pan;

pub struct SynthData {
    /// The sample rate the Synthesizer and voices should use
    pub sample_rate: f64,
    /// The balance of the instrument represented as a float between -1 and 1,
    /// where 0 is center and 1 is to the right.
    pan: f32,
    /// The raw amp values for panning
    /// This can be used in tandem with a state object to set the global
    /// panning values every block render, without having to perform
    /// an expensive panning formula every time.  For instance, we can
    /// calculate `constant_power_pan` in a callback every time the pan knob is moved
    /// and assign that value to a tuple.
    /// Then, before calling the `render_next` method on our synth, we can set the
    /// `pan_raw` field to our aforementioned tuple.
    /// Note that although the framework supports any number of outputs,
    /// panning is currently only supported with stereo.
    pub pan_raw: (f32, f32),
    /// The number of samples passed since the plugin started.  Can represent 24372 centuries of
    /// samples at 48kHz, so wrapping shouldn't be a problem.
    pub sample_counter: f64,
    // Probably some other fields to be added
}

impl Default for SynthData {
    fn default() -> Self {
        let pan = 0.0;
        SynthData {
            sample_rate: 48_000.0,
            pan: pan,
            pan_raw: pan::constant_power(pan),
            sample_counter: 0.0,
        }
    }
}

impl SynthData {
    /// Set the panning for the entire instrument
    /// This is done via a function instead of directly setting the field
    /// as the formula is potentially costly and should only be calculated
    /// when needed.  For instance, do not use this function in a loop for
    /// every sample.  Instead, update the value only when parameters change.
    /// If you need to set the panning every block render, consider accessing
    /// the `pan_raw` field directly.
    ///
    /// * `amount` - a float value between -1 and 1 where 0 is center and 1 is to the right.
    /// Values not within this range will be
    pub fn set_pan(&mut self, amount: f32) {
        self.pan = amount;
        let (pan_left_amp, pan_right_amp) = pan::constant_power(self.pan);
        self.pan_raw = (pan_left_amp, pan_right_amp);
    }
}

use point::Point;

/// General use envelope with any number of points.
#[derive(Clone)]
pub struct Envelope {
    pub points: Vec<Point>,
}

impl Envelope {
    /// Finds the amplitude at a certain value on the `x` axis.  Note that the envelope ends at `x = 1`
    /// and not the last `x` value specified.
    #[allow(unused)]
    pub fn interpolate(&self, x: f64) -> f64 {
        // TODO
        1f64
    }

    /// Finds the amplitude at a certain time.
    ///
    /// - `time` - the time, in milliseconds, that the envelope should calculate from
    /// - `total_length` - the total length, in milliseconds, that the envelope lasts.  Note that
    /// the envelope ends at the last `x` value specified.  This is to make time scaling / adding
    /// additional values after the last point easier.
    #[allow(unused)]
    pub fn interpolate_at_time(&self, time: f64, total_length: f64) -> f64 {
        // TODO
        1f64
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Envelope {
            points: vec![Point { x: 0f64, y: 1f64 }, Point { x: 1f64, y: 1f64 }],
        }
    }
}

/// Factory for `Envelope`
pub struct EnvelopeBuilder {
    pub points: Vec<Point>,
}

impl EnvelopeBuilder {
    /// Create a new `EnvelopeBuilder`
    pub fn new() -> Self {
        EnvelopeBuilder { points: vec![] }
    }

    /// Add a point to the envelope.
    pub fn add_point(mut self, point: Point) -> Self {
        self.points.push(point);
        self
    }

    /// Sorts points in the envelope and returns a `GenericEnvelope`
    pub fn finalize(mut self) -> Envelope {
        // sort the points
        self.points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        // return our `Envelope`
        Envelope {
            points: self.points,
        }
    }
}

//TODO: Specialized envelope with a vector for each ADSR stage

/// A struct that contains a variety of envelopes that our voice may need
#[derive(Clone)]
pub struct EnvelopeContainer {
    amplitude: Envelope,
}

impl Default for EnvelopeContainer {
    fn default() -> Self {
        EnvelopeContainer {
            amplitude: Envelope::default(),
        }
    }
}

/// Specifies a generic trait to be used by different types of points.  X and Y values can be anywhere from 0 to 1.
#[derive(Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

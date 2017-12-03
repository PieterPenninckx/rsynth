/// Get the time, in milliseconds, that has passed from a sample count
#[inline]
pub fn sample_count_to_ms(sample_count: f64, sample_rate: f64) -> f64 {
	(sample_count_to_seconds(sample_count, sample_rate) / 1000f64)
}

/// Get the time, in seconds, that has passed from a sample count
#[inline]
pub fn sample_count_to_seconds(sample_count: f64, sample_rate: f64) -> f64 {
	(sample_count / sample_rate)
}
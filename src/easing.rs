

pub fn exp_in_out(t: f64) -> f64 {
    if t < 0.5 {
        2.0f64.powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - 2.0f64.powf(-20.0 * t + 10.0)) / 2.0
    }
}
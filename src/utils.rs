pub mod math{
    pub fn round(value: f64, decimal_places: u32) -> f64 {
        let factor = 10f64.powi(decimal_places as i32);
        (value * factor).round() / factor
    }
}

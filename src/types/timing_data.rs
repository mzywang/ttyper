#[derive(Clone, Debug, PartialEq)]
pub struct TimingData {
    // Instead of storing WPM, we store CPS (clicks per second)
    pub overall_cps: f64,
    pub per_event: Vec<f64>,
    pub per_key: HashMap<KeyEvent, f64>,
}

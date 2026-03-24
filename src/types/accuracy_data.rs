#[derive(Clone, Debug, PartialEq)]
pub struct AccuracyData {
    pub overall: Fraction,
    pub per_key: HashMap<KeyEvent, Fraction>,
}

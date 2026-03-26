use crate::types::Fraction;
use std::collections::HashMap;
use tuirealm::ratatui::crossterm::event::KeyEvent;

#[derive(Clone, Debug, PartialEq)]
pub struct AccuracyData {
    pub overall: Fraction,
    pub per_key: HashMap<KeyEvent, Fraction>,
}

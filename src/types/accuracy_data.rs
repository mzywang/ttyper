use std::collections::HashMap;
use tuirealm::ratatui::crossterm::event::KeyEvent;
use crate::types::Fraction;

#[derive(Clone, Debug, PartialEq)]
pub struct AccuracyData {
    pub overall: Fraction,
    pub per_key: HashMap<KeyEvent, Fraction>,
}

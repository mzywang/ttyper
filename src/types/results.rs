use crate::calculate;
use crate::types::{AccuracyData, Test, TestEvent, TimingData};

#[derive(Clone, Debug, PartialEq)]
pub struct Results {
    pub timing: TimingData,
    pub accuracy: AccuracyData,
    pub missed_words: Vec<String>,
}

impl From<&Test> for Results {
    fn from(test: &Test) -> Self {
        let events: Vec<&TestEvent> = test.words.iter().flat_map(|w| w.events.iter()).collect();

        Self {
            timing: calculate::timing(&events),
            accuracy: calculate::accuracy(&events),
            missed_words: calculate::missed_words(test),
        }
    }
}

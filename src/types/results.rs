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
            timing: calc_timing(&events),
            accuracy: calc_accuracy(&events),
            missed_words: calc_missed_words(test),
        }
    }
}

use std::collections::HashMap;
use std::time::Instant;
use std::{cmp, fmt};
use tuirealm::ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub type StartTestWords = Vec<String>;

pub struct TestEvent {
    pub time: Instant,
    pub key: KeyEvent,
    pub correct: Option<bool>,
}

pub fn is_missed_word_event(event: &TestEvent) -> bool {
    event.correct != Some(true)
}

impl fmt::Debug for TestEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestEvent")
            .field("time", &String::from("Instant { ... }"))
            .field("key", &self.key)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestWord {
    pub text: String,
    pub progress: String,
    pub events: Vec<TestEvent>,
}

impl Clone for TestEvent {
    fn clone(&self) -> Self {
        Self {
            time: self.time,
            key: self.key,
            correct: self.correct,
        }
    }
}

impl PartialEq for TestEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.key == other.key && self.correct == other.correct
    }
}

impl From<String> for TestWord {
    fn from(string: String) -> Self {
        TestWord {
            text: string,
            progress: String::new(),
            events: Vec::new(),
        }
    }
}

impl From<&str> for TestWord {
    fn from(string: &str) -> Self {
        Self::from(string.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    pub words: Vec<TestWord>,
    pub current_word: usize,
    pub complete: bool,
    pub backtracking_enabled: bool,
    pub sudden_death_enabled: bool,
    pub backspace_enabled: bool,
}

impl Test {
    pub fn new(
        words: Vec<String>,
        backtracking_enabled: bool,
        sudden_death_enabled: bool,
        backspace_enabled: bool,
    ) -> Self {
        Self {
            words: words.into_iter().map(TestWord::from).collect(),
            current_word: 0,
            complete: false,
            backtracking_enabled,
            sudden_death_enabled,
            backspace_enabled,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        let word = &mut self.words[self.current_word];
        match key.code {
            KeyCode::Char(' ') | KeyCode::Enter => {
                if word.text.chars().nth(word.progress.len()) == Some(' ') {
                    word.progress.push(' ');
                    word.events.push(TestEvent {
                        time: Instant::now(),
                        correct: Some(true),
                        key,
                    })
                } else if !word.progress.is_empty() || word.text.is_empty() {
                    let correct = word.text == word.progress;
                    if self.sudden_death_enabled && !correct {
                        self.reset();
                    } else {
                        word.events.push(TestEvent {
                            time: Instant::now(),
                            correct: Some(correct),
                            key,
                        });
                        self.next_word();
                    }
                }
            }
            KeyCode::Backspace => {
                if word.progress.is_empty() && self.backtracking_enabled && self.backspace_enabled {
                    self.last_word();
                } else if self.backspace_enabled {
                    word.events.push(TestEvent {
                        time: Instant::now(),
                        correct: Some(!word.text.starts_with(&word.progress[..])),
                        key,
                    });
                    word.progress.pop();
                }
            }
            // CTRL-BackSpace and CTRL-W
            KeyCode::Char('h') | KeyCode::Char('w')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                if self.words[self.current_word].progress.is_empty() {
                    self.last_word();
                }

                let word = &mut self.words[self.current_word];

                word.events.push(TestEvent {
                    time: Instant::now(),
                    correct: None,
                    key,
                });
                word.progress.clear();
            }
            KeyCode::Char(c) => {
                word.progress.push(c);
                let correct = word.text.starts_with(&word.progress[..]);
                if self.sudden_death_enabled && !correct {
                    self.reset();
                } else {
                    word.events.push(TestEvent {
                        time: Instant::now(),
                        correct: Some(correct),
                        key,
                    });
                    if word.progress == word.text && self.current_word == self.words.len() - 1 {
                        self.complete = true;
                        self.current_word = 0;
                    }
                }
            }
            _ => {}
        };
    }

    fn last_word(&mut self) {
        if self.current_word != 0 {
            self.current_word -= 1;
        }
    }

    fn next_word(&mut self) {
        if self.current_word == self.words.len() - 1 {
            self.complete = true;
            self.current_word = 0;
        } else {
            self.current_word += 1;
        }
    }

    fn reset(&mut self) {
        self.words.iter_mut().for_each(|word: &mut TestWord| {
            word.progress.clear();
            word.events.clear();
        });
        self.current_word = 0;
        self.complete = false;
    }
}

// Result types

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fraction {
    pub numerator: usize,
    pub denominator: usize,
}

impl Fraction {
    pub const fn new(numerator: usize, denominator: usize) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl From<Fraction> for f64 {
    fn from(f: Fraction) -> Self {
        f.numerator as f64 / f.denominator as f64
    }
}

impl cmp::Ord for Fraction {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        f64::from(*self).partial_cmp(&f64::from(*other)).unwrap()
    }
}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimingData {
    // Instead of storing WPM, we store CPS (clicks per second)
    pub overall_cps: f64,
    pub per_event: Vec<f64>,
    pub per_key: HashMap<KeyEvent, f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccuracyData {
    pub overall: Fraction,
    pub per_key: HashMap<KeyEvent, Fraction>,
}

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

fn calc_timing(events: &[&TestEvent]) -> TimingData {
    let mut timing = TimingData {
        overall_cps: -1.0,
        per_event: Vec::new(),
        per_key: HashMap::new(),
    };

    // map of keys to a two-tuple (total time, clicks) for counting average
    let mut keys: HashMap<KeyEvent, (f64, usize)> = HashMap::new();

    for win in events.windows(2) {
        let event_dur = win[1]
            .time
            .checked_duration_since(win[0].time)
            .map(|d| d.as_secs_f64());

        if let Some(event_dur) = event_dur {
            timing.per_event.push(event_dur);

            let key = keys.entry(win[1].key).or_insert((0.0, 0));
            key.0 += event_dur;
            key.1 += 1;
        }
    }

    timing.per_key = keys
        .into_iter()
        .map(|(key, (total, count))| (key, total / count as f64))
        .collect();

    timing.overall_cps = if timing.per_event.is_empty() {
        0.0
    } else {
        timing.per_event.len() as f64 / timing.per_event.iter().sum::<f64>()
    };

    timing
}

fn calc_accuracy(events: &[&TestEvent]) -> AccuracyData {
    let mut acc = AccuracyData {
        overall: Fraction::new(0, 0),
        per_key: HashMap::new(),
    };

    events
        .iter()
        .filter(|event| event.correct.is_some())
        .for_each(|event| {
            let key = acc
                .per_key
                .entry(event.key)
                .or_insert_with(|| Fraction::new(0, 0));

            acc.overall.denominator += 1;
            key.denominator += 1;

            if event.correct.unwrap() {
                acc.overall.numerator += 1;
                key.numerator += 1;
            }
        });

    acc
}

fn calc_missed_words(test: &Test) -> Vec<String> {
    test.words
        .iter()
        .filter(|word| word.events.iter().any(is_missed_word_event))
        .map(|word| word.text.clone())
        .collect()
}

use rand::{seq::SliceRandom, thread_rng};
use tuirealm::ratatui::crossterm::event::KeyCode as CrosstermKeyCode;
use tuirealm::ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    symbols::Marker,
    text::{Line, Span, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Widget},
};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Event, Key, KeyModifiers, NoUserEvent},
    AttrValue, Attribute, Component, Frame, MockComponent, State,
};

use crate::config::Theme;
use crate::messages::Msg;
use crate::types::{Fraction, Results};

// Convert CPS to WPM (clicks per second)
const WPM_PER_CPS: f64 = 12.0;

// Width of the moving average window for the WPM chart
const WPM_SMA_WIDTH: usize = 10;

pub struct ResultsComponent {
    pub results: Results,
    pub theme: Theme,
}

impl MockComponent for ResultsComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let buf = frame.buffer_mut();

        buf.set_style(area, self.theme.default);

        // Chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        let res_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1) // Graph looks tremendously better with just a little margin
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(chunks[0]);
        let info_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(res_chunks[0]);

        let msg = if self.results.missed_words.is_empty() {
            "Press 'q' to quit or 'r' for another test"
        } else {
            "Press 'q' to quit, 'r' for another test or 'p' to practice missed words"
        };

        let exit = Span::styled(msg, self.theme.results_restart_prompt);
        buf.set_span(chunks[1].x, chunks[1].y, &exit, chunks[1].width);

        // Sections
        let mut overview_text = Text::styled("", self.theme.results_overview);
        overview_text.extend([
            Line::from(format!(
                "Adjusted WPM: {:.1}",
                self.results.timing.overall_cps
                    * WPM_PER_CPS
                    * f64::from(self.results.accuracy.overall)
            )),
            Line::from(format!(
                "Accuracy: {:.1}%",
                f64::from(self.results.accuracy.overall) * 100f64
            )),
            Line::from(format!(
                "Raw WPM: {:.1}",
                self.results.timing.overall_cps * WPM_PER_CPS
            )),
            Line::from(format!(
                "Correct Keypresses: {}",
                self.results.accuracy.overall
            )),
        ]);
        let overview = Paragraph::new(overview_text).block(
            Block::default()
                .title(Span::styled("Overview", self.theme.title))
                .borders(Borders::ALL)
                .border_type(self.theme.border_type)
                .border_style(self.theme.results_overview_border),
        );
        overview.render(info_chunks[0], buf);

        let mut worst_keys: Vec<(&tuirealm::ratatui::crossterm::event::KeyEvent, &Fraction)> = self
            .results
            .accuracy
            .per_key
            .iter()
            .filter(|(key, _)| matches!(key.code, CrosstermKeyCode::Char(_)))
            .collect();
        worst_keys.sort_unstable_by_key(|x| x.1);

        let mut worst_text = Text::styled("", self.theme.results_worst_keys);
        worst_text.extend(
            worst_keys
                .iter()
                .filter_map(|(key, acc)| {
                    if let CrosstermKeyCode::Char(character) = key.code {
                        let key_accuracy = f64::from(**acc) * 100.0;
                        if key_accuracy != 100.0 {
                            Some(format!("- {} at {:.1}% accuracy", character, key_accuracy))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .take(5)
                .map(Line::from),
        );
        let worst = Paragraph::new(worst_text).block(
            Block::default()
                .title(Span::styled("Worst Keys", self.theme.title))
                .borders(Borders::ALL)
                .border_type(self.theme.border_type)
                .border_style(self.theme.results_worst_keys_border),
        );
        worst.render(info_chunks[1], buf);

        let wpm_sma: Vec<(f64, f64)> = self
            .results
            .timing
            .per_event
            .windows(WPM_SMA_WIDTH)
            .enumerate()
            .map(|(i, window): (usize, &[f64])| {
                (
                    (i + WPM_SMA_WIDTH) as f64,
                    window.len() as f64 / window.iter().copied().sum::<f64>() * WPM_PER_CPS,
                )
            })
            .collect();

        // Render the chart if possible
        if !wpm_sma.is_empty() {
            let wpm_sma_min = wpm_sma
                .iter()
                .map(|(_, x)| x)
                .fold(f64::INFINITY, |a: f64, &b: &f64| a.min(b));
            let wpm_sma_max = wpm_sma
                .iter()
                .map(|(_, x)| x)
                .fold(f64::NEG_INFINITY, |a: f64, &b: &f64| a.max(b));

            let wpm_datasets = vec![Dataset::default()
                .name("WPM")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(self.theme.results_chart)
                .data(&wpm_sma)];

            let y_label_min = wpm_sma_min as u16;
            let y_label_max = (wpm_sma_max as u16).max(y_label_min + 6);

            let wpm_chart = Chart::new(wpm_datasets)
                .block(Block::default().title(vec![Span::styled("Chart", self.theme.title)]))
                .x_axis(
                    Axis::default()
                        .title(Span::styled("Keypresses", self.theme.results_chart_x))
                        .bounds([0.0, self.results.timing.per_event.len() as f64]),
                )
                .y_axis(
                    Axis::default()
                        .title(Span::styled(
                            "WPM (10-keypress rolling average)",
                            self.theme.results_chart_y,
                        ))
                        .bounds([wpm_sma_min, wpm_sma_max])
                        .labels(
                            (y_label_min..y_label_max)
                                .step_by(5)
                                .map(|n| Span::raw(format!("{}", n)))
                                .collect::<Vec<_>>(),
                        ),
                );
            wpm_chart.render(res_chunks[1], buf);
        }
    }

    fn query(&self, _attr: Attribute) -> Option<AttrValue> {
        None
    }

    fn attr(&mut self, _attr: Attribute, _value: AttrValue) {}

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for ResultsComponent {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key)
                if key.code == Key::Char('q') && key.modifiers == KeyModifiers::NONE =>
            {
                Some(Msg::AppClose)
            }
            Event::Keyboard(key)
                if key.code == Key::Char('r') && key.modifiers == KeyModifiers::NONE =>
            {
                Some(Msg::RestartTest)
            }
            Event::Keyboard(key)
                if key.code == Key::Char('p') && key.modifiers == KeyModifiers::NONE =>
            {
                if self.results.missed_words.is_empty() {
                    return None;
                }
                // repeat each missed word 5 times
                let mut practice_words: Vec<String> = (self.results.missed_words)
                    .iter()
                    .flat_map(|w: &String| vec![w.clone(); 5])
                    .collect();

                practice_words.shuffle(&mut thread_rng());

                Some(Msg::StartTest(practice_words))
            }
            _ => None,
        }
    }
}

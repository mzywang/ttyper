use tuirealm::ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Event, Key, KeyModifiers as TuiKeyModifiers, NoUserEvent},
    props::{Alignment, Borders as TuiBorders, Color, Style},
    AttrValue, Attribute, Component, Frame, MockComponent, State,
};

use crate::config::Theme;
use crate::messages::Msg;
use crate::types::{Results, Test, TestWord};

pub struct TestComponent {
    pub test: Test,
    pub theme: Theme,
}

impl MockComponent for TestComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let buf = frame.buffer_mut();

        buf.set_style(area, self.theme.default);

        // Chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(6)])
            .split(area);

        // Input Section
        let input_block = Block::default()
            .title(Line::from(vec![Span::styled("Input", self.theme.title)]))
            .borders(Borders::ALL)
            .border_type(self.theme.border_type)
            .border_style(self.theme.input_border);

        let input_inner_area = input_block.inner(chunks[0]);
        input_block.render(chunks[0], buf);

        let input_text = Line::from(self.test.words[self.test.current_word].progress.clone());
        buf.set_line(
            input_inner_area.x,
            input_inner_area.y,
            &input_text,
            input_inner_area.width,
        );

        // Target (Prompt) Section
        let target_lines: Vec<Line> = {
            let words = words_to_spans(&self.test.words, self.test.current_word, &self.theme);

            let mut lines: Vec<Line> = Vec::new();
            let mut current_line: Vec<Span> = Vec::new();
            let mut current_width = 0;
            for word in words {
                let word_width: usize = word.iter().map(|s| s.width()).sum();

                if current_width + word_width > chunks[1].width as usize - 2 {
                    lines.push(Line::from(current_line.clone()));
                    current_line.clear();
                    current_width = 0;
                }

                current_line.extend(word);
                current_width += word_width;
            }
            lines.push(Line::from(current_line));

            lines
        };

        let target = Paragraph::new(target_lines).block(
            Block::default()
                .title(Span::styled("Prompt", self.theme.title))
                .borders(Borders::ALL)
                .border_type(self.theme.border_type)
                .border_style(self.theme.prompt_border),
        );
        target.render(chunks[1], buf);

        // Cursor positioning
        let inner_x = chunks[0].x + 1;
        let inner_y = chunks[0].y + 1;
        let progress_width =
            Line::from(self.test.words[self.test.current_word].progress.as_str()).width() as u16;
        let max_cursor_x = chunks[0].right().saturating_sub(2);

        frame.set_cursor_position(((inner_x + progress_width).min(max_cursor_x), inner_y));
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

impl Component<Msg, NoUserEvent> for TestComponent {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key_event) => {
                // Convert tuirealm KeyEvent to crossterm KeyEvent for Test::handle_key
                let crossterm_key = convert_tuirealm_to_crossterm_key(key_event);

                self.test.handle_key(crossterm_key);
                if self.test.complete {
                    let results = Results::from(&self.test);
                    Some(Msg::ShowResults(results))
                } else {
                    Some(Msg::None)
                }
            }
            _ => None,
        }
    }
}

fn convert_tuirealm_to_crossterm_key(
    key: tuirealm::event::KeyEvent,
) -> tuirealm::ratatui::crossterm::event::KeyEvent {
    use tuirealm::event::Key;
    use tuirealm::ratatui::crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
    };

    let code = match key.code {
        Key::Backspace => KeyCode::Backspace,
        Key::Enter => KeyCode::Enter,
        Key::Left => KeyCode::Left,
        Key::Right => KeyCode::Right,
        Key::Up => KeyCode::Up,
        Key::Down => KeyCode::Down,
        Key::Home => KeyCode::Home,
        Key::End => KeyCode::End,
        Key::PageUp => KeyCode::PageUp,
        Key::PageDown => KeyCode::PageDown,
        Key::Tab => KeyCode::Tab,
        Key::BackTab => KeyCode::BackTab,
        Key::Delete => KeyCode::Delete,
        Key::Insert => KeyCode::Insert,
        Key::Function(n) => KeyCode::F(n),
        Key::Char(c) => KeyCode::Char(c),
        Key::Null => KeyCode::Null,
        Key::Esc => KeyCode::Esc,
        Key::CapsLock => KeyCode::CapsLock,
        Key::ScrollLock => KeyCode::ScrollLock,
        Key::NumLock => KeyCode::NumLock,
        Key::PrintScreen => KeyCode::PrintScreen,
        Key::Pause => KeyCode::Pause,
        Key::Menu => KeyCode::Menu,
        Key::KeypadBegin => KeyCode::KeypadBegin,
        _ => KeyCode::Null,
    };

    let mut modifiers = KeyModifiers::empty();
    if key.modifiers.contains(TuiKeyModifiers::SHIFT) {
        modifiers.insert(KeyModifiers::SHIFT);
    }
    if key.modifiers.contains(TuiKeyModifiers::CONTROL) {
        modifiers.insert(KeyModifiers::CONTROL);
    }
    if key.modifiers.contains(TuiKeyModifiers::ALT) {
        modifiers.insert(KeyModifiers::ALT);
    }

    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

// Helpers

fn words_to_spans<'a>(
    words: &'a [TestWord],
    current_word: usize,
    theme: &'a Theme,
) -> Vec<Vec<Span<'a>>> {
    let mut spans = Vec::new();

    for word in &words[..current_word] {
        let parts = split_typed_word(word);
        spans.push(word_parts_to_spans(parts, theme));
    }

    if current_word < words.len() {
        let parts_current = split_current_word(&words[current_word]);
        spans.push(word_parts_to_spans(parts_current, theme));

        for word in &words[current_word + 1..] {
            let parts = vec![(word.text.clone(), Status::Untyped)];
            spans.push(word_parts_to_spans(parts, theme));
        }
    }
    spans
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Status {
    Correct,
    Incorrect,
    CurrentUntyped,
    CurrentCorrect,
    CurrentIncorrect,
    Cursor,
    Untyped,
    Overtyped,
}

fn split_current_word(word: &TestWord) -> Vec<(String, Status)> {
    let mut parts = Vec::new();
    let mut cur_string = String::new();
    let mut cur_status = Status::Untyped;

    let mut progress = word.progress.chars();
    for tc in word.text.chars() {
        let p = progress.next();
        let status = match p {
            None => Status::CurrentUntyped,
            Some(c) => match c {
                c if c == tc => Status::CurrentCorrect,
                _ => Status::CurrentIncorrect,
            },
        };

        if status == cur_status {
            cur_string.push(tc);
        } else {
            if !cur_string.is_empty() {
                parts.push((cur_string, cur_status));
                cur_string = String::new();
            }
            cur_string.push(tc);
            cur_status = status;

            // first currentuntyped is cursor
            if status == Status::CurrentUntyped {
                parts.push((cur_string, Status::Cursor));
                cur_string = String::new();
            }
        }
    }
    if !cur_string.is_empty() {
        parts.push((cur_string, cur_status));
    }
    let overtyped = progress.collect::<String>();
    if !overtyped.is_empty() {
        parts.push((overtyped, Status::Overtyped));
    }
    parts
}

fn split_typed_word(word: &TestWord) -> Vec<(String, Status)> {
    let mut parts = Vec::new();
    let mut cur_string = String::new();
    let mut cur_status = Status::Untyped;

    let mut progress = word.progress.chars();
    for tc in word.text.chars() {
        let p = progress.next();
        let status = match p {
            None => Status::Untyped,
            Some(c) => match c {
                c if c == tc => Status::Correct,
                _ => Status::Incorrect,
            },
        };

        if status == cur_status {
            cur_string.push(tc);
        } else {
            if !cur_string.is_empty() {
                parts.push((cur_string, cur_status));
                cur_string = String::new();
            }
            cur_string.push(tc);
            cur_status = status;
        }
    }
    if !cur_string.is_empty() {
        parts.push((cur_string, cur_status));
    }

    let overtyped = progress.collect::<String>();
    if !overtyped.is_empty() {
        parts.push((overtyped, Status::Overtyped));
    }
    parts
}

fn word_parts_to_spans(parts: Vec<(String, Status)>, theme: &Theme) -> Vec<Span<'_>> {
    let mut spans = Vec::new();
    for (text, status) in parts {
        let style = match status {
            Status::Correct => theme.prompt_correct,
            Status::Incorrect => theme.prompt_incorrect,
            Status::Untyped => theme.prompt_untyped,
            Status::CurrentUntyped => theme.prompt_current_untyped,
            Status::CurrentCorrect => theme.prompt_current_correct,
            Status::CurrentIncorrect => theme.prompt_current_incorrect,
            Status::Cursor => theme.prompt_current_untyped.patch(theme.prompt_cursor),
            Status::Overtyped => theme.prompt_incorrect,
        };

        spans.push(Span::styled(text, style));
    }
    spans.push(Span::styled(" ", theme.prompt_untyped));
    spans
}

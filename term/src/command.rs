use std::collections::VecDeque;

use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span};
use ratatui::style::Color;
use ratatui::widgets::{Block, Paragraph, Widget};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

pub struct Command {
    input: Input,
    history: VecDeque<String>,
    history_position: Option<usize>,
    completions: &'static [&'static str],
    completion: Option<&'static str>,
}

pub enum Control {
    Command(String),
    Hide,
}

impl Command {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(32),
            input: Input::default(),
            history_position: None,
            completions: &[],
            completion: None,
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let mut line = vec![Span::from(":"), Span::from(self.input.value())];
        if let Some(completion) = self
            .completion
            .and_then(|c| c.strip_prefix(self.input.value()))
        {
            line.push(Span::from(completion).style(Color::DarkGray));
        }
        let input = Paragraph::new(Line::from(line))
            .scroll((0, scroll as u16))
            .style(Color::Yellow)
            .block(Block::bordered());
        input.render(area, frame.buffer_mut());

        // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
        // end of the input text and one line down from the border to the input line
        let x = self.input.visual_cursor().max(scroll) - scroll + 2;
        frame.set_cursor_position((area.x + x as u16, area.y + 1));
    }

    pub fn set_completions(&mut self, completions: &'static [&'static str]) {
        self.completions = completions;
        self.completion = None;
    }

    fn set_history(&mut self, position: usize) {
        self.history_position = Some(position);
        self.set_value(self.history[position].clone());
    }

    fn set_value(&mut self, value: String) {
        self.input = Input::new(value);
        self.refresh_completion();
    }

    fn refresh_completion(&mut self) {
        if self.input.value().is_empty() {
            self.completion = None;
        } else {
            self.completion = self
                .completions
                .iter()
                .find(|c| c.starts_with(self.input.value()) && c.len() != self.input.value().len())
                .cloned();
        }
    }

    fn value_and_reset(&mut self) -> String {
        let value = self.input.value_and_reset();
        self.history_position = None;
        self.completion = None;
        value
    }

    pub fn handle_event(&mut self, event: Event) -> Option<Control> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Esc => {
                    if key_event.is_press() {
                        self.value_and_reset();
                        return Some(Control::Hide);
                    }
                    return None;
                }
                KeyCode::Up => {
                    if key_event.is_press() && !self.history.is_empty() {
                        let position = self
                            .history_position
                            .unwrap_or(self.history.len())
                            .saturating_sub(1);
                        self.set_history(position);
                    }
                    return None;
                }
                KeyCode::Down => {
                    if !self.history.is_empty() && key_event.is_press() {
                        if let Some(position) = self.history_position {
                            let position = (position + 1).min(self.history.len() - 1);
                            self.set_history(position);
                        }
                    }
                    return None;
                }
                KeyCode::Right => {
                    if key_event.is_press() {
                        if let Some(completion) = self.completion {
                            self.set_value(completion.into());
                            return None;
                        }
                    }
                }
                KeyCode::Enter => {
                    if key_event.is_press() {
                        let value = self.value_and_reset();
                        return if value.trim().is_empty() {
                            Some(Control::Hide)
                        } else {
                            if self.history.len() == 32 {
                                self.history.pop_front();
                            }
                            if Some(&value) != self.history.back() {
                                self.history.push_back(value.clone());
                            }

                            Some(Control::Command(value))
                        };
                    }
                    return None;
                }
                _ => {}
            }
        }

        if let Some(changed) = self.input.handle_event(&event) {
            if changed.value {
                self.history_position = None;
                self.refresh_completion();
            }
        }
        None
    }
}

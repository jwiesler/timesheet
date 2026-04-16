use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListState, StatefulWidget};

use crate::term::style::{HIGHLIGHT, TIME};

pub struct ListSelect {
    state: ListState,
    codes: Vec<KeyCode>,
    list: List<'static>,
}

impl ListSelect {
    pub fn new(options: Vec<(&'static str, KeyCode)>) -> ListSelect {
        let codes = options.iter().map(|(_, key)| *key).collect();
        let list = List::new(options.into_iter().map(|(text, key)| {
            Line::from(vec![
                Span::from(format!("{: <2}", key.as_char().unwrap_or(' '))).style(TIME),
                Span::from(text),
            ])
        }))
        .highlight_style(HIGHLIGHT);
        let mut state = ListState::default();
        state.select(Some(0));
        ListSelect { state, codes, list }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        (&self.list).render(area, buf, &mut self.state);
    }

    pub fn handle_event(&mut self, e: &Event) -> Option<usize> {
        if let Event::Key(event) = e
            && event.is_press()
        {
            match event.code {
                KeyCode::Down => {
                    self.state.scroll_down_by(1);
                }
                KeyCode::Up => {
                    self.state.scroll_up_by(1);
                }
                KeyCode::Enter => {
                    return Some(self.state.selected().unwrap());
                }
                _ => {
                    if let Some(index) = self.codes.iter().position(|v| v == &event.code) {
                        return Some(index);
                    }
                }
            }
        }
        None
    }

    pub fn height(&self) -> u16 {
        self.list.len().try_into().unwrap()
    }
}

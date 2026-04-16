use std::borrow::Cow;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    List, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
};

use crate::term::style::{HIGHLIGHT, TIME};

pub struct ListSelect {
    state: ListState,
    codes: Vec<KeyCode>,
    list: List<'static>,
}

impl ListSelect {
    pub fn new(options: Vec<(Cow<'static, str>, Option<KeyCode>)>, selected: usize) -> ListSelect {
        let codes = options.iter().filter_map(|(_, key)| *key).collect();
        let list = List::new(options.into_iter().map(|(text, key)| {
            Line::from(vec![
                Span::from(format!(
                    "{: <2}",
                    key.and_then(|k| k.as_char()).unwrap_or(' ')
                ))
                .style(TIME),
                Span::from(text),
            ])
        }))
        .highlight_style(HIGHLIGHT);
        let state = ListState::default().with_selected(Some(selected));
        ListSelect { state, codes, list }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        (&self.list).render(area, buf, &mut self.state);
        let scroll_elements = self.list.len().saturating_sub(usize::from(area.height));
        let offset = self.state.offset();
        let mut state = ScrollbarState::new(scroll_elements).position(offset);
        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(area, buf, &mut state);
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

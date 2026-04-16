use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span};
use ratatui::widgets::{Block, Clear, Widget};
use times::Date;

use crate::term::components::popup_rect;
use crate::term::components::select::ListSelect;
use crate::term::style::{BORDER, BORDERS};

pub struct Months(Box<ListSelect>);

pub enum Control {
    Hide,
    Month(usize),
}

impl Months {
    pub fn new(months: impl IntoIterator<Item = Date>, current: usize) -> Months {
        Self(Box::new(ListSelect::new(
            months
                .into_iter()
                .enumerate()
                .map(|(i, date)| {
                    (
                        format!("{:0>2}-{}", date.month(), date.year()).into(),
                        char::from_digit(i.try_into().unwrap(), 10).map(KeyCode::Char),
                    )
                })
                .collect(),
            current,
        )))
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let height = self.0.height().min(12) + 2;
        let area = popup_rect(area, height);
        Clear.render(area, buf);
        let title = Line::from(Span::from("Select month"));
        let block = Block::bordered()
            .title_top(Span::default())
            .title_top(title)
            .border_set(BORDERS)
            .border_style(BORDER);
        (&block).render(area, buf);
        let area = block.inner(area);
        self.0.render(area, buf);
    }

    pub fn handle_event(&mut self, e: &Event) -> Option<Control> {
        if let Event::Key(key) = e
            && e.is_key_press()
            && let KeyCode::Esc = key.code
        {
            return Some(Control::Hide);
        }
        self.0.handle_event(e).map(Control::Month)
    }

    pub fn controls() -> &'static [(&'static str, KeyCode)] {
        &[("Confirm", KeyCode::Enter), ("Cancel", KeyCode::Esc)]
    }
}

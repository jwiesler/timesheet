use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::widgets::{Block, Clear, Widget};
use times::convert::AccumulatedTime;
use times::{Date, Minutes};

use crate::term::app::output_time_delta;
use crate::term::components::popup_rect;
use crate::term::style::{BORDER, BORDERS};

pub struct Summary {
    date: Date,
    time: AccumulatedTime,
    expected_min_work: Minutes,
}

pub enum Control {
    Hide,
}

impl Summary {
    pub fn new(date: Date, time: AccumulatedTime, expected_min_work: Minutes) -> Summary {
        Self {
            date,
            time,
            expected_min_work,
        }
    }

    #[expect(clippy::unused_self)]
    pub fn handle_event(&mut self, event: &Event) -> Option<Control> {
        if let Event::Key(key) = event
            && event.is_key_press()
            && let KeyCode::Esc = key.code
        {
            return Some(Control::Hide);
        }

        None
    }

    pub fn controls() -> &'static [(&'static str, KeyCode)] {
        &[("Close", KeyCode::Esc)]
    }

    fn minutes_to_span(minutes: Minutes) -> Span<'static> {
        Span::from(format!("{}", minutes.into_duration()))
    }

    pub fn render(&mut self, area: Rect, frame: &mut Frame<'_>) {
        let stats = [
            ("Work time: ", Self::minutes_to_span(self.time.work_time())),
            (
                "Travel time: ",
                Self::minutes_to_span(self.time.travel_time()),
            ),
            (
                "Billable time: ",
                Self::minutes_to_span(self.time.billable_time()),
            ),
            (
                "Expected time: ",
                Self::minutes_to_span(self.expected_min_work),
            ),
            (
                "Difference: ",
                output_time_delta(self.time.billable_time(), self.expected_min_work),
            ),
        ];
        let height = stats.len() + 2;
        let area = popup_rect(area, height.try_into().unwrap());
        Clear.render(area, frame.buffer_mut());
        let title = Line::from(Span::from(format!("Summary of year {}", self.date.year())));
        let block = Block::bordered()
            .title_top(Span::default())
            .title_top(title)
            .border_set(BORDERS)
            .border_style(BORDER);
        (&block).render(area, frame.buffer_mut());
        let area = block.inner(area);
        let areas = area.layout_vec(&Layout::vertical(
            stats.clone().map(|_| Constraint::Length(1)),
        ));
        for ((heading, stat), area) in stats.into_iter().zip(areas) {
            let line = Line::from(vec![Span::from(heading), stat]);
            line.render(area, frame.buffer_mut());
        }
    }
}

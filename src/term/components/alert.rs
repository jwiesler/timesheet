use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::prelude::{Text, Widget};
use ratatui::widgets::{Block, Clear};

use crate::term::style::{BORDER, BORDERS, HIGHLIGHT};

pub struct Alert {
    text: String,
}

pub enum Control {
    Exit,
}

impl From<String> for Alert {
    fn from(value: String) -> Self {
        Self { text: value }
    }
}

impl Alert {
    pub fn controls() -> &'static [(&'static str, KeyCode)] {
        &[("Dismiss", KeyCode::Enter)]
    }

    #[expect(clippy::unused_self)]
    pub fn handle_event(&mut self, event: &Event) -> Option<Control> {
        if let Event::Key(event) = event
            && event.is_press()
            && event.code == KeyCode::Enter
        {
            return Some(Control::Exit);
        }
        None
    }

    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    pub fn draw(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Error ")
            .title_alignment(Alignment::Center)
            .border_set(BORDERS)
            .border_style(BORDER);
        let area = Self::popup_area(area, 60, 20);
        Clear.render(area, buf);
        (&block).render(area, buf);
        let area = block.inner(area);
        let [message_area, _, button_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .areas(area);
        let button = Text::from(" Dismiss ").style(HIGHLIGHT);
        let button_area = h_center(button_area, button.width());
        button.render(button_area, buf);
        let message = Text::from(self.text.as_str());
        let message_area = h_center(message_area, message.width());
        message.render(message_area, buf);
    }
}

pub fn h_center(area: Rect, width: usize) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width.try_into().unwrap())])
        .flex(Flex::Center)
        .areas(area);
    area
}

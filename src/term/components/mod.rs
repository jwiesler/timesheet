use ratatui::layout::{Constraint, Rect};

pub mod add;
pub mod alert;
pub mod command;
pub mod month;
pub mod months;
pub mod select;
pub mod summary;

fn popup_rect(area: Rect, height: u16) -> Rect {
    let min_width = 30;
    let width = if area.width > min_width {
        min_width + (area.width - min_width) / 4
    } else {
        area.width
    };

    area.centered(Constraint::Length(width), Constraint::Length(height))
}

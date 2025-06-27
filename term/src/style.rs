use ratatui::style::{Color, Style};

// pub const AQUA: Color = Color::Rgb(0, 0xFF, 0xFF);
pub const LIGHT_SKY_BLUE: Color = Color::Rgb(0x87, 0xCE, 0xFA);

pub const HIGHLIGHT: Style = Style::new().bg(Color::LightCyan);
pub const BORDER: Style = Style::new().fg(LIGHT_SKY_BLUE);
pub const PROJECT: Style = Style::new().fg(Color::LightGreen);
pub const TIME: Style = Style::new().fg(Color::LightMagenta);
pub const DATE: Style = Style::new().fg(Color::LightYellow);

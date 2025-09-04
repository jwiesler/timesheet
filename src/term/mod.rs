mod command;
mod data;
mod editor;
mod model;
mod month;
mod style;

use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Clear, Widget};
use times::generate::Template;
use times::{Date, Minutes, NaiveDate};

use crate::append_to_file;
use crate::term::command::Command;
use crate::term::data::Data;
use crate::term::editor::run_editor;
use crate::term::model::Model;
use crate::term::month::Month;
use crate::term::style::{BORDER, HIGHLIGHT};

pub fn run_term(path: &Path) -> std::io::Result<()> {
    let state = Data::from_dir(path.parent().unwrap())?;
    let mut terminal = ratatui::init();
    let today = Date::today();
    let month = {
        let (date, path) = state
            .months
            .iter()
            .find(|(_, p)| p.as_path() == path)
            .or(state.months.last())
            .unwrap()
            .clone();
        let month = Model::load(date, path)?;
        Month::new(month)
    };
    let result = App::new(state, today, month).run(&mut terminal);
    ratatui::restore();
    result
}

pub(crate) trait View {
    fn render(&mut self, area: Rect, buf: &mut Buffer);

    #[must_use]
    fn handle_event(&mut self, e: Event) -> Option<Control>;

    fn command(&mut self, command: &str, args: &[&str]) -> Result<(), UnknownCommand>;
}

#[derive(Eq, PartialEq)]
enum Focus {
    Input,
    View,
    Alert,
}

#[must_use]
pub(crate) enum Control {
    Quit,
    Month(Date, Rc<PathBuf>),
    Edit,
    Alert(String),
}

struct Error(String);

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error(s)
    }
}

impl From<std::io::Error> for Error {
    fn from(s: std::io::Error) -> Self {
        Error(s.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct App {
    focus: Focus,
    command: Command,
    data: Data,
    month: Month,
    today: Date,
    alert: Alert,
}

impl App {
    fn new(data: Data, today: Date, month: Month) -> Self {
        let mut command = Command::new();
        command.set_completions(&[
            "month",
            "month prev",
            "month next",
            "month last",
            "month first",
            "expand",
            "collapse",
            "add normal",
            "add tech-day",
            "add holiday",
            "add empty",
            "add ill",
            "add tng-weekly",
        ]);
        Self {
            month,
            data,
            focus: Focus::View,
            command,
            today,
            alert: Alert::new(),
        }
    }

    fn find_month(&self, month: u32, year: i32) -> Option<&(Date, Rc<PathBuf>)> {
        let needle = Date::new(NaiveDate::from_ymd_opt(year, month, 1)?);
        self.data
            .months
            .binary_search_by(|(date, _)| date.cmp(&needle))
            .ok()
            .and_then(|i| self.data.months.get(i))
    }

    fn current_month(&self) -> usize {
        self.data
            .months
            .iter()
            .enumerate()
            .find(|(_, (date, _))| date == &self.month.date())
            .unwrap()
            .0
    }

    fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            match self.handle_event(event::read()?) {
                None => {}
                Some(Control::Quit) => break,
                Some(Control::Month(date, path)) => {
                    let model = Model::load(date, path)?;
                    self.month = Month::new(model);
                }
                Some(Control::Edit) => {
                    run_editor(terminal, self.month.path(), self.month.line())?;
                    let model = Model::load(self.month.date(), self.month.path().clone())?;
                    self.month.reload(model);
                }
                Some(Control::Alert(message)) => {
                    self.alert = Alert::from(message);
                    self.focus = Focus::Alert;
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        let view_area = if let Focus::Input = self.focus {
            let [input_area, rest] =
                Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(frame.area());
            self.command.draw(input_area, frame);
            rest
        } else {
            frame.area()
        };

        self.month.render(view_area, frame.buffer_mut());

        if let Focus::Alert = self.focus {
            self.alert.draw(frame.area(), frame.buffer_mut());
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<Control> {
        if let Event::Key(event) = event
            && event.is_press()
            && event.code == KeyCode::Char('c')
            && event.modifiers == KeyModifiers::CONTROL
        {
            return Some(Control::Quit);
        }
        match self.focus {
            Focus::Input => match self.command.handle_event(&event)? {
                command::Control::Command(command) => {
                    self.focus = Focus::View;
                    let mut iter = command.split_whitespace();
                    if let Some(command) = iter.next() {
                        let args = iter.collect::<Vec<&str>>();
                        return self
                            .handle_command(command, &args)
                            .unwrap_or_else(|e| Some(Control::Alert(e.0)));
                    }
                }
                command::Control::Hide => {
                    self.focus = Focus::View;
                }
            },
            Focus::View => {
                if let Event::Key(event) = event
                    && event.code == KeyCode::Char(':')
                    && event.is_press()
                {
                    self.focus = Focus::Input;
                    return None;
                }
                return self.month.handle_event(event);
            }
            Focus::Alert => {
                if let Event::Key(event) = event
                    && event.is_press()
                    && event.code == KeyCode::Enter
                {
                    self.focus = Focus::View;
                }
            }
        }
        None
    }

    fn handle_command(&mut self, command: &str, args: &[&str]) -> Result<Option<Control>, Error> {
        match command {
            "q" => Ok(Some(Control::Quit)),
            "month" => {
                let date = match *args {
                    ["last"] => self.data.months.last(),
                    ["first"] => self.data.months.first(),
                    ["prev"] => {
                        let current = self.current_month();
                        self.data.months.get(current.saturating_sub(1))
                    }
                    ["next"] => {
                        let current = self.current_month();
                        self.data.months.get(current.saturating_add(1))
                    }
                    [month] => {
                        let month = month
                            .parse::<u32>()
                            .map_err(|err| format!("Failed to parse month: {err}"))?;
                        let year = self.today.year();
                        self.find_month(month, year)
                    }
                    [month, year] => {
                        let month = month
                            .parse::<u32>()
                            .map_err(|err| format!("Failed to parse year: {err}"))?;
                        let year = year
                            .parse::<i32>()
                            .map_err(|err| format!("Failed to parse year: {err}"))?;
                        self.find_month(month, year)
                    }
                    _ => return Err(format!("Unknown args to `month`: {args:?}").into()),
                };
                Ok(date.map(|(date, path)| Control::Month(*date, path.clone())))
            }
            "add" => {
                let [template_name, args @ ..] = args else {
                    return Err("Missing template name argument to `add`".to_owned().into());
                };

                let template = match *template_name {
                    "empty" => Template::Empty,
                    "tech-day" => Template::TechDay,
                    "holiday" => Template::Holiday,
                    "normal" => Template::Normal,
                    "ill" => Template::Ill,
                    "tng-weekly" => Template::TNGWeekly,
                    _ => {
                        return Err(
                            format!("Unknown template arg to `add`: {template_name}").into()
                        );
                    }
                };
                let date = self
                    .month
                    .days()
                    .last()
                    .and_then(|d| d.date.value.following_day_in_month())
                    .unwrap_or(self.month.date())
                    .next_weekday_in_month()
                    .expect("last day in the month");
                let rendered = template
                    .execute(date, args)
                    .map_err(|e| format!("Failed to run template {template_name}: {e}"))?;
                append_to_file(self.month.path(), &rendered)?;
                let model = Model::load(self.month.date(), self.month.path().clone())?;
                self.month.reload(model);
                self.month.select_last();
                Ok(None)
            }
            _ => self
                .month
                .command(command, args)
                .map(|()| None)
                .map_err(|_| format!("Unknown command: {command}").into()),
        }
    }
}

#[derive(Debug)]
pub struct UnknownCommand;

struct Alert {
    text: String,
}

impl From<String> for Alert {
    fn from(value: String) -> Self {
        Self { text: value }
    }
}

impl Alert {
    fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    fn draw(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Error ")
            .title_alignment(Alignment::Center)
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

fn output_time_delta(lhs: Minutes, rhs: Minutes) -> Span<'static> {
    if lhs < rhs {
        let delta = rhs - lhs;
        let duration = delta.into_duration();
        Span::from(format!("-{duration}")).style(Style::default().fg(Color::Red))
    } else {
        let delta = lhs - rhs;
        let duration = delta.into_duration();
        Span::from(format!("+{duration}")).style(Style::default().fg(Color::Green))
    }
}

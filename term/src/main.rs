mod command;
mod data;
mod editor;
mod model;
mod month;
mod style;

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use times::generate::Template;
use times::{Date, Minutes, NaiveDate};

use crate::command::Command;
use crate::data::Data;
use crate::editor::run_editor;
use crate::model::Model;
use crate::month::Month;

fn main() -> std::io::Result<()> {
    let arg = std::env::args_os().nth(1);
    let dir = arg
        .map(PathBuf::from)
        .unwrap_or_else(|| "timesheets".into());
    let state = Data::from_dir(dir)?;
    let mut terminal = ratatui::init();
    let today = Date::today();
    let month = {
        let (date, path) = state.months.last().unwrap().clone();
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

    fn command(&mut self, command: &str, args: &[&str]);
}

#[derive(Eq, PartialEq)]
enum Focus {
    Input,
    View,
}

#[must_use]
enum Control {
    Quit,
    Month(Date, Rc<PathBuf>),
    Edit,
}

struct App {
    focus: Focus,
    command: Command,
    data: Data,
    month: Month,
    today: Date,
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
            "add empty",
            "add tech-day",
            "add holiday",
            "add normal",
            "add ill",
            "add tng-weekly",
        ]);
        Self {
            month,
            data,
            focus: Focus::View,
            command,
            today,
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
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let view_area = if let Focus::Input = self.focus {
            let [input_area, rest] =
                Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(frame.area());
            self.command.draw(input_area, frame);
            rest
        } else {
            frame.area()
        };

        self.month.render(view_area, frame.buffer_mut());
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
            Focus::Input => match self.command.handle_event(event)? {
                command::Control::Command(command) => {
                    self.focus = Focus::View;
                    let mut iter = command.split_whitespace();
                    if let Some(command) = iter.next() {
                        let args = iter.collect::<Vec<&str>>();
                        return self.handle_command(command, &args);
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
        }
        None
    }

    fn handle_command(&mut self, command: &str, args: &[&str]) -> Option<Control> {
        match command {
            "q" => Some(Control::Quit),
            "month" => {
                let (date, path) = match *args {
                    ["last"] => self.data.months.last()?,
                    ["first"] => self.data.months.first()?,
                    ["prev"] => {
                        let current = self.current_month();
                        self.data.months.get(current.saturating_sub(1))?
                    }
                    ["next"] => {
                        let current = self.current_month();
                        self.data.months.get(current.saturating_add(1))?
                    }
                    [month] => {
                        let month = month.parse::<u32>().ok()?;
                        let year = self.today.year();
                        self.find_month(month, year)?
                    }
                    [month, year] => {
                        let month = month.parse::<u32>().ok()?;
                        let year = year.parse::<i32>().ok()?;
                        self.find_month(month, year)?
                    }
                    _ => return None,
                };
                Some(Control::Month(*date, path.clone()))
            }
            "add" => {
                if let [template, args @ ..] = args {
                    let template = match *template {
                        "empty" => Template::Empty,
                        "tech-day" => Template::TechDay,
                        "holiday" => Template::Holiday,
                        "normal" => Template::Normal,
                        "ill" => Template::Ill,
                        "tng-weekly" => Template::TNGWeekly,
                        _ => return None,
                    };
                    let date = self
                        .month
                        .days()
                        .last()
                        .and_then(|d| d.date.value.following_day_in_month())
                        .unwrap_or(self.month.date())
                        .next_weekday_in_month()
                        .expect("last day in the month");
                    let Ok(rendered) = template.execute(date, args) else {
                        return None;
                    };
                    append_to_file(self.month.path(), &rendered).unwrap();
                    let model = Model::load(self.month.date(), self.month.path().clone()).unwrap();
                    self.month.reload(model);
                }
                None
            }
            _ => {
                self.month.command(command, args);
                None
            }
        }
    }
}

fn append_to_file(path: &Path, text: &str) -> Result<(), std::io::Error> {
    let file = OpenOptions::new().append(true).open(path)?;
    BufWriter::new(file).write_all(text.as_bytes())
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

use std::fmt::Display;
use std::path::PathBuf;
use std::rc::Rc;

use chrono::NaiveDate;
use ratatui::Frame;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Color, Line, Span, Style, Widget};
use ratatui::widgets::{Block, Padding};
use times::generate::Template;
use times::{Date, Minutes};

use crate::append_to_file;
use crate::term::components::add::Add;
use crate::term::components::alert::Alert;
use crate::term::components::command::Command;
use crate::term::components::month::Month;
use crate::term::components::months::Months;
use crate::term::components::{add, alert, command, months};
use crate::term::data::Data;
use crate::term::editor::run_editor;
use crate::term::model::Model;
use crate::term::style::{BORDERS, border};

enum Focus {
    Input,
    View,
    Alert(Alert),
    Dialog(Add),
    Months(Months),
}

#[must_use]
pub(crate) enum Control {
    Quit,
    Month(Date, Rc<PathBuf>),
    Edit,
    Alert(String),
}

pub struct App {
    focus: Focus,
    command: Command,
    data: Data,
    month: Month,
    today: Date,
}

impl App {
    pub fn new(data: Data, today: Date, month: Month) -> Self {
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
            "add full",
            "add holiday",
            "add empty",
            "add ill",
            "add timeoff",
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

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            match self.handle_event(&event::read()?) {
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
                    self.focus = Focus::Alert(Alert::from(message));
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = if let Focus::Input = self.focus {
            let [input_area, rest] =
                Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(frame.area());
            self.command.draw(input_area, frame);
            rest
        } else {
            frame.area()
        };
        let [view_area, controls_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let border = border(matches!(self.focus, Focus::View));
        let block = Block::bordered()
            .border_style(border)
            .border_set(BORDERS)
            .padding(Padding::horizontal(1));

        let title = self
            .month
            .render(block.inner(view_area), frame.buffer_mut());
        block.title_top(title).render(view_area, frame.buffer_mut());

        let controls = match &mut self.focus {
            Focus::Input => &[("Confirm", KeyCode::Enter), ("Cancel", KeyCode::Esc)],
            Focus::View => Month::controls(),
            Focus::Alert(alert) => {
                alert.draw(view_area, frame.buffer_mut());
                Alert::controls()
            }
            Focus::Dialog(dialog) => {
                dialog.render(view_area, frame);
                Add::controls()
            }
            Focus::Months(months) => {
                months.render(view_area, frame.buffer_mut());
                Months::controls()
            }
        };
        let base_controls: &[_] = if matches!(self.focus, Focus::View) {
            &[("Add", KeyCode::Char('a')), ("Month", KeyCode::Char('m'))]
        } else {
            &[]
        };
        let controls = Self::format_controls(controls.iter().chain(base_controls));
        controls.render(controls_area, frame.buffer_mut());
    }

    fn format_controls<'a>(
        controls: impl Iterator<Item = &'a (&'a str, KeyCode)>,
    ) -> Line<'static> {
        let mut res = Vec::with_capacity(2 * controls.size_hint().0);
        for (i, (name, key)) in controls.into_iter().enumerate() {
            if i > 0 {
                res.push(Span::from(" | "));
            }
            res.push(Span::from(format!("{name}: <{key}>")));
        }
        Line::from(res).style(Style::default().fg(Color::Blue))
    }

    fn handle_event(&mut self, event: &Event) -> Option<Control> {
        if let Event::Key(event) = event
            && event.is_press()
            && event.code == KeyCode::Char('c')
            && event.modifiers == KeyModifiers::CONTROL
        {
            return Some(Control::Quit);
        }
        match &mut self.focus {
            Focus::Input => match self.command.handle_event(event)? {
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
                    && event.is_press()
                {
                    match event.code {
                        KeyCode::Char(':') => {
                            self.focus = Focus::Input;
                            return None;
                        }
                        KeyCode::Char('m') => {
                            let current = self.current_month();
                            self.focus = Focus::Months(Months::new(
                                self.data.months.iter().rev().map(|(a, _)| *a),
                                self.data.months.len() - 1 - current,
                            ));
                        }
                        KeyCode::Char('a') => {
                            self.focus = Focus::Dialog(Add::new());
                        }
                        _ => {}
                    }
                }
                return self.month.handle_event(event);
            }
            Focus::Alert(alert) => match alert.handle_event(event)? {
                alert::Control::Exit => {
                    self.focus = Focus::View;
                }
            },
            Focus::Dialog(dialog) => match dialog.handle_event(event)? {
                add::Control::Add { template, args } => {
                    self.focus = Focus::View;
                    return self
                        .execute_template(
                            template,
                            &args.iter().map(String::as_str).collect::<Vec<_>>(),
                        )
                        .map_or_else(|e| Some(Control::Alert(e.0)), |()| None);
                }
                add::Control::Hide => {
                    self.focus = Focus::View;
                }
            },
            Focus::Months(months) => match months.handle_event(event)? {
                months::Control::Hide => {
                    self.focus = Focus::View;
                }
                months::Control::Month(month) => {
                    let month = self.data.months.len() - 1 - month;
                    self.focus = Focus::View;
                    return self
                        .data
                        .months
                        .get(month)
                        .map(|(date, path)| Control::Month(*date, path.clone()));
                }
            },
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
                            .map_err(|err| format!("Failed to parse month {month:?}: {err}"))?;
                        let year = self.today.year();
                        self.find_month(month, year)
                    }
                    [month, year] => {
                        let month = month
                            .parse::<u32>()
                            .map_err(|err| format!("Failed to parse month {month:?}: {err}"))?;
                        let year = year
                            .parse::<i32>()
                            .map_err(|err| format!("Failed to parse year {year:?}: {err}"))?;
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
                    "holiday" => Template::Holiday,
                    "normal" => Template::Normal,
                    "ill" => Template::Ill,
                    "full" => Template::Full,
                    "timeoff" => Template::TimeOff,
                    _ => {
                        return Err(
                            format!("Unknown template arg to `add`: {template_name}").into()
                        );
                    }
                };
                self.execute_template(template, args)?;
                Ok(None)
            }
            _ => self
                .month
                .command(command, args)
                .map(|()| None)
                .map_err(|_| format!("Unknown command: {command}").into()),
        }
    }

    fn execute_template(&mut self, template: Template, args: &[&str]) -> Result<(), Error> {
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
            .map_err(|e| format!("Failed to run template {template:?}: {e}"))?;
        append_to_file(self.month.path(), &rendered)?;
        let model = Model::load(self.month.date(), self.month.path().clone())?;
        self.month.reload(model);
        self.month.select_last();
        Ok(())
    }
}

pub fn output_time_delta(lhs: Minutes, rhs: Minutes) -> Span<'static> {
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

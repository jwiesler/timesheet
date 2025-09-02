use std::collections::HashMap;
use std::ops::Not;
use std::path::PathBuf;
use std::rc::Rc;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::prelude::Line;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, List, ListItem, ListState, Padding, StatefulWidget};
use times::convert::Day;
use times::Date;

use crate::term::model::Model;
use crate::term::style::{BORDER, DATE, HIGHLIGHT, PROJECT, TIME};
use crate::term::{output_time_delta, Control, View};

pub struct Month {
    state: ListState,
    expanded: Vec<bool>,
    model: Model,
}

impl Month {
    pub fn new(model: Model) -> Self {
        let state =
            ListState::default().with_selected(model.month().days.is_empty().not().then_some(0));
        let days = model.month().days.len();
        Self {
            state,
            expanded: vec![false; days],
            model,
        }
    }

    pub(crate) fn reload(&mut self, model: Model) {
        let selected = self.state.selected().unwrap_or_default();
        let Some((day, start)) = self.day_index_from_index(selected) else {
            self.model = model;
            self.expanded.resize(self.days().len(), false);
            self.state.select(Some(0));
            return;
        };
        let offset = selected - start;
        let selected_date = self.days()[day].date.value;
        let expanded = self
            .days()
            .iter()
            .zip(self.expanded.iter())
            .map(|(day, expanded)| (day.date.value, *expanded))
            .collect::<HashMap<_, _>>();
        self.model = model;
        self.expanded.resize(self.days().len(), false);
        let mut list_offset = 0;
        for (day_index, day) in self.model.month().days.iter().enumerate() {
            if self.state.selected().is_none() && day.date.value >= selected_date {
                let offset = offset.min(day.entries.len() + 1);
                self.state.select(Some(list_offset + offset));
            }
            let expanded = expanded.get(&day.date.value).copied().unwrap_or_default();
            self.expanded[day_index] = expanded;
            list_offset += len_of_entry(day, expanded);
        }
    }

    pub(crate) fn path(&self) -> &Rc<PathBuf> {
        self.model.path()
    }

    pub(crate) fn line(&self) -> u32 {
        let selected = self.state.selected().unwrap_or_default();
        let Some((day, start)) = self.day_index_from_index(selected) else {
            return 0;
        };
        let day = &self.days()[day];
        if selected == start {
            day.date.line
        } else {
            day.entries[selected - start - 1].value.start.line
        }
        .try_into()
        .unwrap()
    }

    pub(crate) fn date(&self) -> Date {
        self.model.date()
    }

    pub(crate) fn days(&self) -> &[Day] {
        &self.model.month().days
    }

    fn render_day(day: &Day, expanded: bool) -> Vec<ListItem<'_>> {
        let expected = day.expected_time();
        let date = day.date.value.to_string();

        let arrow = Span::from(if expanded { "▼ " } else { "▶ " });
        let mut text = vec![
            arrow,
            Span::from(date).style(DATE),
            Span::from(" -> "),
            Span::from(day.times.billable_time().into_duration().to_string()),
        ];
        if day.times.billable_time() != expected {
            let delta = output_time_delta(day.times.billable_time(), expected);
            text.extend([Span::from(" ("), delta, Span::from(")")]);
        }
        let mut lines = vec![ListItem::new(Line::from(text))];
        if expanded {
            lines.extend(day.entries.iter().map(|entry| {
                let entry = &entry.value;
                let mut items = vec![
                    Span::from("   "),
                    Span::from(format!("{} - {}", entry.start.value, entry.end.value)).style(TIME),
                    Span::from(" "),
                    Span::from(entry.identifier.as_str()).style(PROJECT),
                ];
                if let Some(comment) = &entry.comment {
                    items.push(Span::from(" "));
                    items.push(Span::from(comment.as_str()));
                }
                ListItem::new(Line::from(items))
            }));
        }
        lines
    }

    fn day_index_from_index(&self, index: usize) -> Option<(usize, usize)> {
        let mut running_index = 0;
        let days = self.days();
        for ((i, day), expanded) in days.iter().enumerate().zip(&self.expanded) {
            let start = running_index;
            let end = running_index + len_of_entry(day, *expanded);
            running_index = end;
            if (start..end).contains(&index) {
                return Some((i, start));
            }
        }
        None
    }

    fn start_of_day(&self, index: usize) -> usize {
        self.days()[..index]
            .iter()
            .zip(&self.expanded)
            .map(|(d, expanded)| len_of_entry(d, *expanded))
            .sum::<usize>()
    }
}

fn len_of_entry(day: &Day, expanded: bool) -> usize {
    (if expanded { day.entries.len() } else { 0 }) + 1
}

impl View for Month {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let rows = self
            .model
            .month()
            .days
            .iter()
            .enumerate()
            .flat_map(|(i, day)| {
                let expanded = self.expanded[i];
                Self::render_day(day, expanded)
            });
        let billable_time = self.model.month().times.billable_time();
        let title = Line::from(vec![
            Span::from(format!(
                " Monat {:0>2}-{} ",
                self.date().month(),
                self.date().year()
            ))
            .style(Style::new().fg(Color::White)),
            Span::from(format!("-> {} (", billable_time.into_duration())).style(Style::reset()),
            output_time_delta(billable_time, self.model.month().expected_min_work),
            Span::from(") ").style(Style::reset()),
        ]);

        let block = Block::bordered()
            .title(title)
            .border_style(BORDER)
            .padding(Padding::horizontal(1));
        let list_height = block.inner(area).height;
        let list = List::new(rows).block(block).highlight_style(HIGHLIGHT);
        *self.state.offset_mut() = self
            .state
            .offset()
            .min(list.len().saturating_sub(usize::from(list_height)));
        list.render(area, buf, &mut self.state);
    }

    fn handle_event(&mut self, e: Event) -> Option<Control> {
        let Event::Key(e) = e else {
            return None;
        };
        if !e.is_press() {
            return None;
        }
        match e.code {
            KeyCode::Down => {
                self.state.scroll_down_by(1);
            }
            KeyCode::Up => {
                self.state.scroll_up_by(1);
            }
            KeyCode::Left => {
                if let Some(selected) = self.state.selected() {
                    let (day, start) = self.day_index_from_index(selected).unwrap_or_default();
                    self.expanded[day] = false;
                    self.state.select(Some(start));
                }
            }
            KeyCode::Right => {
                if let Some(selected) = self.state.selected() {
                    let (day, _) = self.day_index_from_index(selected).unwrap_or_default();
                    self.expanded[day] = true;
                }
            }
            KeyCode::Home | KeyCode::Char('g') => self.state.select_first(),
            KeyCode::End | KeyCode::Char('G') => self.state.select_last(),
            KeyCode::Char('n') => {
                if let Some(selected) = self.state.selected()
                    && let Some((day, start)) = self.day_index_from_index(selected)
                {
                    self.state.select(Some(
                        start + len_of_entry(&self.days()[day], self.expanded[day]),
                    ));
                }
            }
            KeyCode::Char('N') => {
                if let Some(selected) = self.state.selected() {
                    let (day, start) = self.day_index_from_index(selected).unwrap_or_default();
                    let index = if selected == start && day > 0 {
                        start - len_of_entry(&self.days()[day - 1], self.expanded[day - 1])
                    } else {
                        start
                    };
                    self.state.select(Some(index));
                }
            }
            KeyCode::Char('e') => {
                return Some(Control::Edit);
            }
            _ => {}
        }
        None
    }

    fn command(&mut self, command: &str, _: &[&str]) {
        if let Ok(day) = command.parse::<u32>() {
            let days = &self.model.month().days;
            if days.is_empty() {
                return;
            }
            let index = days
                .iter()
                .zip(&self.expanded)
                .take_while(|(d, _)| d.date.value.day() < day)
                .map(|(d, expanded)| len_of_entry(d, *expanded))
                .sum::<usize>();
            self.state.select(Some(index));
            *self.state.offset_mut() = index;
            return;
        }
        match command {
            "collapse" | "c" => {
                let selected = self.state.selected().unwrap();
                let (day, _) = self.day_index_from_index(selected).unwrap_or_default();
                self.expanded.fill(false);
                let new_day_start = self.start_of_day(day);
                self.state.select(Some(new_day_start));
                *self.state.offset_mut() = new_day_start;
            }
            "expand" | "e" => {
                let selected = self.state.selected().unwrap();
                let (day, start) = self.day_index_from_index(selected).unwrap_or_default();
                let day_offset = selected - start;
                self.expanded.fill(true);
                let new_day_start = self.start_of_day(day);
                self.state.select(Some(new_day_start + day_offset));
                *self.state.offset_mut() = new_day_start;
            }
            _ => {}
        }
    }
}

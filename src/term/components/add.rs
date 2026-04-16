use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Widget};
use times::generate::Template;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use crate::term::components::popup_rect;
use crate::term::components::select::ListSelect;
use crate::term::style::{BORDER, BORDERS};

pub enum Control {
    Add {
        template: Template,
        args: Vec<String>,
    },
    Hide,
}

enum State {
    Template(Box<ListSelect>),
    Parameters {
        template: Template,
        name: &'static str,
        input: Input,
    },
}

pub struct Add {
    state: State,
}

const OPTIONS: &[(&str, char, Template)] = &[
    ("Normal", 'n', Template::Normal),
    ("Full", 'f', Template::Full),
    ("Empty", 'e', Template::Empty),
    ("Holiday", 'h', Template::Holiday),
    ("Time off", 't', Template::TimeOff),
    ("Ill", 'i', Template::Ill),
];

impl Add {
    pub fn new() -> Add {
        Self {
            state: State::Template(Box::new(ListSelect::new(
                OPTIONS
                    .iter()
                    .map(|(text, key, _)| ((*text).into(), Some(KeyCode::Char(*key))))
                    .collect(),
                0,
            ))),
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<Control> {
        if let Event::Key(key) = event
            && event.is_key_press()
            && let KeyCode::Esc = key.code
        {
            return Some(Control::Hide);
        }

        match &mut self.state {
            State::Template(list) => {
                if let Some(offset) = list.handle_event(event) {
                    let (name, _, template) = OPTIONS[offset];
                    let no_args = match template {
                        Template::Empty | Template::Holiday | Template::TimeOff | Template::Ill => {
                            true
                        }
                        Template::Normal | Template::Full => false,
                    };
                    if no_args {
                        return Some(Control::Add {
                            template,
                            args: vec![],
                        });
                    }
                    self.state = State::Parameters {
                        template,
                        name,
                        input: Input::default(),
                    };
                }
                None
            }
            State::Parameters {
                input, template, ..
            } => {
                if let Event::Key(key) = event
                    && event.is_key_press()
                    && key.code == KeyCode::Enter
                    && !input.value().is_empty()
                {
                    return Some(Control::Add {
                        template: *template,
                        args: vec![input.value_and_reset()],
                    });
                }
                input.handle_event(event);
                None
            }
        }
    }

    pub fn controls() -> &'static [(&'static str, KeyCode)] {
        &[("Confirm", KeyCode::Enter), ("Cancel", KeyCode::Esc)]
    }

    pub fn render(&mut self, area: Rect, frame: &mut Frame<'_>) {
        let height = match &self.state {
            State::Template(select) => select.height(),
            State::Parameters { .. } => 1,
        } + 2;
        let area = popup_rect(area, height);
        Clear.render(area, frame.buffer_mut());
        let title = match &self.state {
            State::Template(..) => Line::from(Span::from("Select template")),
            State::Parameters { name, .. } => Line::from(Span::from(format!(
                "Enter parameters for template {name:?}"
            ))),
        };
        let block = Block::bordered()
            .title_top(Span::default())
            .title_top(title)
            .border_set(BORDERS)
            .border_style(BORDER);
        (&block).render(area, frame.buffer_mut());
        let area = block.inner(area);
        match &mut self.state {
            State::Template(list) => {
                list.render(area, frame.buffer_mut());
            }
            State::Parameters { input, .. } => {
                Line::from(input.value()).render(area, frame.buffer_mut());
                let scroll = input.visual_scroll(area.width as usize);

                // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
                // end of the input text and one line down from the border to the input line
                let x = input.visual_cursor().max(scroll) - scroll;
                frame.set_cursor_position((area.x + u16::try_from(x).unwrap(), area.y));
            }
        }
    }
}

use std::io::stdout;
use std::path::Path;

use editor_command::EditorBuilder;
use ratatui::DefaultTerminal;
use ratatui::crossterm::ExecutableCommand;
use ratatui::crossterm::terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode};

pub fn run_editor(terminal: &mut DefaultTerminal, path: &Path, line: u32) -> std::io::Result<()> {
    disable_raw_mode()?;
    EditorBuilder::new()
        .environment()
        .build()
        .map_err(std::io::Error::other)?
        .open_at(path, line, 0)
        .status()?;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;
    Ok(())
}

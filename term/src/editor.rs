use std::io::stdout;
use std::path::Path;

use editor_command::EditorBuilder;
use ratatui::DefaultTerminal;
use ratatui::crossterm::ExecutableCommand;
use ratatui::crossterm::terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode};

pub fn run_editor(terminal: &mut DefaultTerminal, path: &Path, line: u32) -> std::io::Result<()> {
    disable_raw_mode()?;
    EditorBuilder::edit_file(path)
        .unwrap()
        .arg(format!("+{line}"))
        .status()?;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;
    Ok(())
}

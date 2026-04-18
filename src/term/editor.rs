use std::io::stdout;
use std::path::Path;

use anyhow::Context;
use editor_command::EditorBuilder;
use ratatui::DefaultTerminal;
use ratatui::crossterm::ExecutableCommand;
use ratatui::crossterm::terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode};

pub fn run_editor(terminal: &mut DefaultTerminal, path: &Path, line: u32) -> anyhow::Result<()> {
    disable_raw_mode()?;
    EditorBuilder::new()
        .environment()
        .build()
        .context("Failed to extract editor command from env")?
        .open_at(path, line, 0)
        .status()
        .context("Failed to run editor")?;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;
    Ok(())
}

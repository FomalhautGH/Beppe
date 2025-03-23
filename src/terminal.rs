use crossterm::cursor;
use crossterm::queue;
use crossterm::style;
use crossterm::terminal::{self, ClearType, disable_raw_mode, enable_raw_mode, size};
use std::io::Write;
use std::io::stdout;

#[derive(Clone, Copy)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Default)]
pub struct TerminalPosition {
    pub x: u16,
    pub y: u16,
}

impl TerminalPosition {
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

pub struct Terminal;

impl Terminal {
    pub fn terminate() -> Result<(), std::io::Error> {
        disable_raw_mode()
    }

    pub fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(TerminalPosition::zero())
    }

    pub fn clear_screen() -> Result<(), std::io::Error> {
        queue!(stdout(), terminal::Clear(ClearType::All))
    }

    pub fn clear_line() -> Result<(), std::io::Error> {
        queue!(stdout(), terminal::Clear(ClearType::CurrentLine))
    }

    pub fn move_cursor_to(pos: TerminalPosition) -> Result<(), std::io::Error> {
        queue!(stdout(), cursor::MoveTo(pos.x, pos.y))
    }

    pub fn hide_cursor() -> Result<(), std::io::Error> {
        queue!(stdout(), cursor::Hide)
    }

    pub fn show_cursor() -> Result<(), std::io::Error> {
        queue!(stdout(), cursor::Show)
    }

    pub fn print(printable: &str) -> Result<(), std::io::Error> {
        queue!(stdout(), style::Print(printable))
    }

    pub fn execute() -> Result<(), std::io::Error> {
        stdout().flush()
    }

    pub fn size() -> Result<TerminalSize, std::io::Error> {
        let (width, height) = size()?;
        Ok(TerminalSize { width, height })
    }
}

use crossterm::cursor;
use crossterm::queue;
use crossterm::style;
use crossterm::terminal::{self, ClearType, disable_raw_mode, enable_raw_mode, size};
use std::io::Write;
use std::io::stdout;

#[derive(Clone, Copy, Default)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

impl TerminalSize {
    pub fn new(dim: (u16, u16)) -> Self {
        Self {
            width: dim.0,
            height: dim.1,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct TerminalPosition {
    pub x: usize,
    pub y: usize,
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
        let (x, y): (u16, u16) = (pos.x.try_into().unwrap(), pos.y.try_into().unwrap());
        queue!(stdout(), cursor::MoveTo(x, y))
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

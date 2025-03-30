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

pub struct Terminal;

impl Terminal {
    pub fn terminate() -> Result<(), std::io::Error> {
        queue!(stdout(), terminal::LeaveAlternateScreen)?;
        Self::show_cursor()?;
        Self::execute()?;
        disable_raw_mode()
    }

    pub fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        queue!(stdout(), terminal::EnterAlternateScreen)?;
        Self::clear_screen()?;
        Self::execute()
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

    pub fn print(string: &str) -> Result<(), std::io::Error> {
        queue!(stdout(), style::Print(string))
    }
    
    pub fn print_row(row: usize, text: &str) -> Result<(), std::io::Error> {
        Self::move_cursor_to(TerminalPosition { x: 0, y: row })?;
        Self::clear_line()?;
        Self::print(text)
    }

    pub fn execute() -> Result<(), std::io::Error> {
        stdout().flush()
    }

    pub fn size() -> Result<TerminalSize, std::io::Error> {
        let (width, height) = size()?;
        Ok(TerminalSize { width, height })
    }
}

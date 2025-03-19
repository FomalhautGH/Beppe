use crossterm::cursor;
use crossterm::queue;
use crossterm::style;
use crossterm::terminal::{self, ClearType, disable_raw_mode, enable_raw_mode, size};
use std::fmt::Display;
use std::io::stdout;
use std::io::Write;

pub struct TermPosition {
    pub x: u16,
    pub y: u16
}

impl TermPosition {
    pub fn zero() -> Self {
        Self {
            x: 0,
            y: 0
        }
    }
}

pub struct TermSize {
    pub width: u16,
    pub height: u16
}

pub struct Terminal;

impl Terminal {
    pub fn terminate() -> Result<(), std::io::Error> {
        disable_raw_mode()?;
        Ok(())
    }

    pub fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(&TermPosition::zero())?;
        Ok(())
    }

    pub fn clear_screen() -> Result<(), std::io::Error> {
        queue!(stdout(), terminal::Clear(ClearType::All))?;
        Ok(())
    }

    pub fn move_cursor_to(pos: &TermPosition) -> Result<(), std::io::Error> {
        queue!(stdout(), cursor::MoveTo(pos.x, pos.y))?;
        Ok(())
    }

    pub fn hide_cursor() -> Result<(), std::io::Error> {
        queue!(stdout(), cursor::Hide)?;
        Ok(())
    }

    pub fn show_cursor() -> Result<(), std::io::Error> {
        queue!(stdout(), cursor::Show)?;
        Ok(())
    }

    pub fn print<T: Display>(printable: T) -> Result<(), std::io::Error> {
        queue!(stdout(), style::Print(printable))?;
        Ok(())
    }

    pub fn flush() -> Result<(), std::io::Error> {
        stdout().flush()
    }

    pub fn size() -> Result<TermSize, std::io::Error> {
        let term_size = size()?;

        Ok(TermSize {
            width: term_size.0,
            height: term_size.1
        })
    }
}

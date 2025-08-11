use crossterm::cursor;
use crossterm::queue;
use crossterm::style;
use crossterm::style::Attribute;
use crossterm::terminal::{self, ClearType, disable_raw_mode, enable_raw_mode, size};
use std::io::Write;
use std::io::stdout;

#[derive(Clone, Copy, Default)]
pub struct TerminalSize {
    pub width: usize,
    pub height: usize,
}

/// If we imagine the terminal as an infinite grid where
/// our cursor stays in one of the cells this struct rapresents
/// exacly that.
#[derive(Clone, Copy, Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub const fn subtract(&self, rhs: &Self) -> Self {
        Self {
            x: self.x.saturating_sub(rhs.x),
            y: self.y.saturating_sub(rhs.y),
        }
    }
}

pub struct Terminal;
impl Terminal {
    /// Initializes the terminal entering the [raw mode](https://docs.rs/crossterm/0.28.1/crossterm/terminal/index.html#raw-mode)
    /// and also entering the alternate screen in order to preserve
    /// precedent output on the terminal (and for visualizing panic outputs)
    pub fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        queue!(
            stdout(),
            terminal::EnterAlternateScreen,
            terminal::DisableLineWrap
        )?;
        Self::clear_screen()?;
        Self::execute()
    }

    /// Terminates the terminal leaving the alternate screen and
    /// disabling raw mode.
    pub fn terminate() -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            terminal::EnableLineWrap,
            terminal::LeaveAlternateScreen
        )?;
        Self::show_cursor()?;
        Self::execute()?;
        disable_raw_mode()
    }

    pub fn set_title(title: &str) -> Result<(), std::io::Error> {
        queue!(stdout(), terminal::SetTitle(title))
    }

    pub fn clear_screen() -> Result<(), std::io::Error> {
        queue!(stdout(), terminal::Clear(ClearType::All))
    }

    pub fn clear_line() -> Result<(), std::io::Error> {
        queue!(stdout(), terminal::Clear(ClearType::CurrentLine))
    }

    pub fn move_cursor_to(pos: Position) -> Result<(), std::io::Error> {
        let (x, y): (u16, u16) = (pos.x.try_into().unwrap(), pos.y.try_into().unwrap());
        queue!(stdout(), cursor::MoveTo(x, y))
    }

    pub fn cursor_bar() -> Result<(), std::io::Error> {
        queue!(stdout(), cursor::SetCursorStyle::SteadyBar)
    }

    pub fn cursor_block() -> Result<(), std::io::Error> {
        queue!(stdout(), cursor::SetCursorStyle::SteadyBlock)
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

    pub fn print_inverted_row(row: usize, text: &str) -> Result<(), std::io::Error> {
        Self::move_cursor_to(Position { x: 0, y: row })?;
        Self::clear_line()?;
        let string = &format!("{}{}{}", Attribute::Reverse, text, Attribute::Reset);
        Self::print(string)
    }

    /// Prints a string on a specific row.
    pub fn print_row(row: usize, text: &str) -> Result<(), std::io::Error> {
        Self::move_cursor_to(Position { x: 0, y: row })?;
        Self::clear_line()?;
        Self::print(text)
    }

    /// Executes the instructions waiting in the queue.
    /// We do this becouse execute!() is inefficient since
    /// writing is an expensive operation and we need to execute
    /// the operations only after evalutating the keypresses.
    pub fn execute() -> Result<(), std::io::Error> {
        stdout().flush()
    }

    pub fn size() -> Result<TerminalSize, std::io::Error> {
        let (width, height) = size()?;
        let (width, height) = (width.into(), height.into());
        Ok(TerminalSize { width, height })
    }
}

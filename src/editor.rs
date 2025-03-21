use crate::terminal::{Terminal, TerminalPosition};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};

const EDITOR_NAME: &str = env!("CARGO_PKG_NAME");
const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Location {
    pub x: u16,
    pub y: u16,
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    ceret_pos: Location,
}

impl Editor {
    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        loop {
            self.refresh_screen()?;

            if self.should_quit {
                break;
            }

            let event = read()?;
            self.evaluate_event(&event)?;
        }

        Ok(())
    }

    fn move_point(&mut self, key_code: KeyCode) -> Result<(), std::io::Error> {
        let size = Terminal::size()?;
        let (mut x, mut y) = (self.ceret_pos.x, self.ceret_pos.y);

        match key_code {
            KeyCode::Up => y = y.saturating_sub(1),
            KeyCode::Down if y < size.height.saturating_sub(1) => y = y.saturating_add(1),
            KeyCode::Right if x < size.width.saturating_sub(1) => x = x.saturating_add(1),
            KeyCode::Left => x = x.saturating_sub(1),

            KeyCode::PageUp => y = 0,
            KeyCode::PageDown => y = size.height.saturating_sub(1),
            KeyCode::Home => x = 0,
            KeyCode::End => x = size.width.saturating_sub(1),

            KeyCode::Down => (),
            KeyCode::Right => (),
            _ => unreachable!(),
        }

        self.ceret_pos.x = x;
        self.ceret_pos.y = y;
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<(), std::io::Error> {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }

                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End => self.move_point(*code)?,

                _ => (),
            }
        }

        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::hide_cursor()?;
        Terminal::move_cursor_to(TerminalPosition::zero())?;

        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n")?;
        } else {
            Self::draw_rows()?;
            Terminal::move_cursor_to(TerminalPosition {
                x: self.ceret_pos.x,
                y: self.ceret_pos.y,
            })?;
        }

        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }

    fn draw_title() -> Result<(), std::io::Error> {
        let width: usize = Terminal::size()?.width.into();
        let msg = format!("{EDITOR_NAME}::{EDITOR_VERSION}");

        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(msg.len())) / 2;
        let padding = " ".repeat(padding.saturating_sub(1));

        let mut msg = format!("~{padding}{msg}");
        msg.truncate(width);

        Terminal::print(msg)
    }

    fn draw_empty_row() -> Result<(), std::io::Error> {
        Terminal::print("~")
    }

    fn draw_rows() -> Result<(), std::io::Error> {
        let ht = Terminal::size()?.height;

        for i in 0..ht {
            Terminal::clear_line()?;

            #[allow(clippy::integer_division)]
            if i == ht / 3 {
                Self::draw_title()?;
            } else {
                Self::draw_empty_row()?;
            }

            if i < ht.saturating_add(1) {
                Terminal::print("\r\n")?;
            }
        }

        Ok(())
    }
}

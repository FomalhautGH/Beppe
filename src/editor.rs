use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};
use crate::terminal::{TermPosition, Terminal};

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn default() -> Self {
        Self { should_quit: false }
    }

    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        loop {
            Terminal::hide_cursor()?;
            self.refresh_screen()?;
            Terminal::show_cursor()?;
            Terminal::flush()?;

            if self.should_quit {
                break;
            }

            let event = read()?;
            self.evaluate_event(&event);
        }

        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => (),
            }
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        if self.should_quit {
            Terminal::clear_screen()?;
            print!("Goodbye.\r\n");
        } else {
            Self::draw_rows()?;
            Terminal::move_cursor_to(&TermPosition::zero())?;
        }

        Ok(())
    }

    fn draw_rows() -> Result<(), std::io::Error> {
        let ht = Terminal::size()?.height;

        for i in 0..ht {
            Terminal::print("~")?;
            if i < ht + 1 {
                Terminal::print("\r\n")?;
            }
        }

        Ok(())
    }
}

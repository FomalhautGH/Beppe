use crate::{
    terminal::{Terminal, TerminalPosition, TerminalSize},
    view::View,
};
use crossterm::event::{Event, KeyCode, KeyModifiers, read};

#[derive(Clone, Copy, Default)]
pub struct Location {
    pub x: u16,
    pub y: u16,
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    ceret_pos: Location,
    view: View,
}

impl Editor {
    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        self.handle_args();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn handle_args(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        if let Some(path) = args.get(1) {
            self.view.load(path);
        }
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        loop {
            self.refresh_screen()?;

            if self.should_quit {
                break Ok(());
            }

            let event = read()?;
            self.evaluate_event(&event)?;
        }
    }

    fn move_point(&mut self, key_code: KeyCode) -> Result<(), std::io::Error> {
        let size = Terminal::size()?;
        let (width, height) = (size.width, size.height);
        let (mut x, mut y) = (self.ceret_pos.x, self.ceret_pos.y);

        match key_code {
            KeyCode::Up | KeyCode::Char('k') => y = y.saturating_sub(1),
            KeyCode::Left | KeyCode::Char('h') => x = x.saturating_sub(1),

            KeyCode::Down | KeyCode::Char('j') if y < height.saturating_sub(1) => {
                y = y.saturating_add(1);
            }
            KeyCode::Right | KeyCode::Char('l') if x < width.saturating_sub(1) => {
                x = x.saturating_add(1);
            }

            KeyCode::PageUp => y = 0,
            KeyCode::PageDown => y = height.saturating_sub(1),
            KeyCode::Home => x = 0,
            KeyCode::End => x = width.saturating_sub(1),

            KeyCode::Down | KeyCode::Right | KeyCode::Char('j' | 'l') => (),

            _ => unreachable!(),
        }

        self.ceret_pos.x = x;
        self.ceret_pos.y = y;
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<(), std::io::Error> {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }

                KeyCode::Up
                | KeyCode::Char('k' | 'j' | 'l' | 'h')
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End => self.move_point(key_event.code)?,

                _ => (),
            },

            Event::Resize(w, h) => {
                self.view.resize(TerminalSize::new((*w, *h)));
            }

            _ => (),
        }

        Ok(())
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Terminal::hide_cursor()?;
        Terminal::move_cursor_to(TerminalPosition::default())?;

        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n")?;
        } else {
            self.view.render()?;
            Terminal::move_cursor_to(TerminalPosition {
                x: self.ceret_pos.x.into(),
                y: self.ceret_pos.y.into(),
            })?;
        }

        Terminal::show_cursor()?;
        Terminal::execute()
    }
}

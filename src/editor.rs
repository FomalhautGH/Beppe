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

pub struct Editor {
    should_quit: bool,
    ceret_pos: Location,
    view: View,
}

impl Editor {
    pub fn new() -> Result<Self, std::io::Error> {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            default_hook(panic_info);
        }));

        Terminal::initialize()?;

        let mut view = View::new();
        let args: Vec<String> = std::env::args().collect();
        if let Some(path) = args.get(1) {
            view.load(path);
        }

        Ok(Self {
            should_quit: false,
            ceret_pos: Location::default(),
            view,
        })
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();

            if self.should_quit {
                break;
            }

            let event = read();
            match event {
                Ok(event) => self.evaluate_event(&event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    panic!("Unrecognized event, error: {err:?}");
                }
            }
        }
    }

    fn move_point(&mut self, key_code: KeyCode) {
        let size = Terminal::size().unwrap_or_default();
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
    }

    fn evaluate_event(&mut self, event: &Event) {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }

                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Char('k' | 'j' | 'l' | 'h')
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End => self.move_point(key_event.code),

                _ => (),
            },

            Event::Resize(w, h) => {
                self.view.resize(TerminalSize::new((*w, *h)));
            }

            _ => (),
        }
    }

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_cursor();

        self.view.render();
        let _ = Terminal::move_cursor_to(TerminalPosition {
            x: self.ceret_pos.x.into(),
            y: self.ceret_pos.y.into(),
        });

        let _ = Terminal::show_cursor();
        let _ = Terminal::execute();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            Terminal::print("Goodbye.\r\n").unwrap();
        }
    }
}

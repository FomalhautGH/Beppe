use super::terminal::TerminalSize;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy)]
pub enum Direction {
    PageUp,
    PageDown,
    Up,
    Left,
    Right,
    Down,
    Home,
    End,
}

#[derive(Clone, Copy)]
pub enum EditorCommand {
    Move(Direction),
    Resize(TerminalSize),
    Quit,
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),

                (KeyCode::Up | KeyCode::Char('k'), _) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Right | KeyCode::Char('l'), _) => Ok(Self::Move(Direction::Right)),
                (KeyCode::Left | KeyCode::Char('h'), _) => Ok(Self::Move(Direction::Left)),
                (KeyCode::Down | KeyCode::Char('j'), _) => Ok(Self::Move(Direction::Down)),

                (KeyCode::Home | KeyCode::Char('0'), _) => Ok(Self::Move(Direction::Home)),
                (KeyCode::End | KeyCode::Char('$'), _) => Ok(Self::Move(Direction::End)),

                (KeyCode::PageUp, _) | (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                    Ok(Self::Move(Direction::PageUp))
                }
                (KeyCode::PageDown, _) | (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                    Ok(Self::Move(Direction::PageDown))
                }

                _ => Err(String::from("KeyEvent is not convertible in EditorCommand")),
            },

            Event::Resize(w, h) => {
                let (width, height): (usize, usize) = (w.into(), h.into());
                Ok(Self::Resize(TerminalSize { width, height }))
            }

            _ => Err(String::from("Event is not convertible in EditorCommand")),
        }
    }
}

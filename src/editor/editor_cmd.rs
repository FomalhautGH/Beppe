use super::terminal::TerminalSize;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy)]
pub enum EditorCommandInsert {
    ExitInsert,
    Write(char),
    Enter,
    Deletion,
    Backspace,
}

impl TryFrom<Event> for EditorCommandInsert {
    type Error = String;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    Ok(EditorCommandInsert::ExitInsert)
                }
                (KeyCode::Backspace, _) => Ok(EditorCommandInsert::Backspace),
                (KeyCode::Delete, _) => Ok(EditorCommandInsert::Deletion),
                (KeyCode::Char(symbol), _) => Ok(EditorCommandInsert::Write(symbol)),
                (KeyCode::Tab, _) => Ok(EditorCommandInsert::Write('\t')),
                (KeyCode::Enter, _) => Ok(EditorCommandInsert::Enter),
                _ => Err(String::from("todo!")),
            },

            _ => Err(String::from(
                "Event is not convertible in EditorCommandInsert",
            )),
        }
    }
}

/// Rapresents the different directions we
/// can take on the view.
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

/// Rapresents the commands on the editor that we
/// support.
#[derive(Clone, Copy)]
pub enum EditorCommand {
    Move(Direction),
    Resize(TerminalSize),
    EnterInsert,
    Save,
    Quit,
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;

    /// Allows conversion from a crossterm `Event` to a `EditorCommand`
    /// we support if it exists one.
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => Ok(Self::Save),
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::Char('i'), _) => Ok(Self::EnterInsert),

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

use crate::editor::terminal::TerminalPosition;

#[derive(Clone, Copy, Default)]
pub struct Location {
    pub x: usize,
    pub y: usize,
}

impl Location {
    pub fn from(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub const fn subtract(&self, rhs: &Self) -> Self {
        Self {
            x: self.x.saturating_sub(rhs.x),
            y: self.y.saturating_sub(rhs.y),
        }
    }
}

impl From<Location> for TerminalPosition {
    fn from(loc: Location) -> Self {
        TerminalPosition { x: loc.x, y: loc.y }
    }
}

use super::{
    editor_cmd::{Direction, EditorCommand},
    terminal::{TerminalPosition, TerminalSize},
};
use crate::editor::Terminal;

mod buffer;
use buffer::Buffer;
mod location;
use location::Location;
mod line;

const EDITOR_NAME: &str = env!("CARGO_PKG_NAME");
const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buf: Buffer,
    needs_redraw: bool,
    size: TerminalSize,

    location: Location,
    scroll_offset: Location,
}

impl View {
    pub fn new() -> Self {
        View {
            buf: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            location: Location::default(),
            scroll_offset: Location::default(),
        }
    }

    pub fn resize(&mut self, size: TerminalSize) {
        self.size = size;
        self.needs_redraw = true;
    }

    pub fn load(&mut self, path: &str) {
        if let Ok(buffer) = Buffer::load(path) {
            self.buf = buffer;
            self.needs_redraw = true;
        }
    }

    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }

        let TerminalSize { width, height } = self.size;
        if width == 0 || height == 0 {
            return;
        }

        #[allow(clippy::integer_division)]
        let vertical_center: usize = height / 3;

        let (sx, sy) = (self.scroll_offset.x, self.scroll_offset.y);
        for i in 0..height {
            if let Some(line) = self.buf.lines.get(i.saturating_add(sy)) {
                Self::render_line(i, &line.get(sx..sx.saturating_add(width)));
            } else if vertical_center == i && self.buf.is_empty() {
                Self::draw_title(i, width);
            } else {
                Self::render_line(i, "~");
            }
        }

        self.needs_redraw = false;
    }

    pub fn handle_command(&mut self, cmd: EditorCommand) {
        match cmd {
            EditorCommand::Move(cmd) => self.handle_movement(cmd),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Quit => unreachable!(),
        }
    }

    pub fn handle_movement(&mut self, mov: Direction) {
        let (width, height) = (self.size.width, self.size.height);
        let (mut x, mut y) = (self.location.x, self.location.y);

        match mov {
            Direction::Up => y = y.saturating_sub(1),
            Direction::Down => y = y.saturating_add(1),
            Direction::Right => x = x.saturating_add(1),
            Direction::Left => x = x.saturating_sub(1),
            Direction::Home => x = 0,
            Direction::PageUp => y = 0,
            Direction::End => x = width.saturating_sub(1),
            Direction::PageDown => y = height.saturating_sub(1),
        }

        self.location = Location::from(x, y);
        self.scroll_location();
    }

    fn scroll_location(&mut self) {
        let (width, height) = (self.size.width, self.size.height);
        let (x, y) = (self.location.x, self.location.y);
        let mut offset_changed = false;

        // Scroll orizontally
        if x < self.scroll_offset.x {
            self.scroll_offset.x = x;
            offset_changed = true;
        } else if x >= self.scroll_offset.x.saturating_add(width) {
            self.scroll_offset.x = x.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }

        // Scroll vertically
        if y < self.scroll_offset.y {
            self.scroll_offset.y = y;
            offset_changed = true;
        } else if y >= self.scroll_offset.y.saturating_add(height) {
            self.scroll_offset.y = y.saturating_sub(height).saturating_add(1);
            offset_changed = true;
        }

        self.needs_redraw = offset_changed;
    }

    fn render_line(row_num: usize, line: &str) {
        let result = Terminal::print_row(row_num, line);
        debug_assert!(result.is_ok(), "Failed to render line.");
    }

    fn draw_title(row_num: usize, width: usize) {
        let msg = format!("{EDITOR_NAME}::{EDITOR_VERSION}");

        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(msg.len())) / 2;
        let padding = " ".repeat(padding.saturating_sub(1));

        let mut msg = format!("~{padding}{msg}");
        msg.truncate(width);

        Self::render_line(row_num, &msg);
    }

    pub fn cursor_position(&self) -> TerminalPosition {
        self.location.subtract(&self.scroll_offset).into()
    }
}

use std::cmp;

use super::{
    editor_cmd::{Direction, EditorCommand},
    terminal::{Position, TerminalSize},
};
use crate::editor::Terminal;

mod buffer;
use buffer::Buffer;
mod location;
use line::Line;
use location::Location;
mod grapheme;
mod line;

const EDITOR_NAME: &str = env!("CARGO_PKG_NAME");
const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buf: Buffer,
    needs_redraw: bool,
    size: TerminalSize,

    text_location: Location,
    scroll_offset: Position,
}

impl View {
    pub fn new() -> Self {
        View {
            needs_redraw: true,
            buf: Buffer::default(),
            size: Terminal::size().unwrap_or_default(),
            text_location: Location::default(),
            scroll_offset: Position::default(),
        }
    }

    pub fn cursor_position(&self) -> Position {
        self.text_location_to_position()
            .subtract(&self.scroll_offset)
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
        let height = self.size.height;

        match mov {
            Direction::Up => self.move_up_by(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::Down => self.move_down_by(1),
            Direction::End => self.move_end_of_line(),
            Direction::Home => self.move_start_of_line(),
            Direction::PageUp => self.move_up_by(height.saturating_sub(1)),
            Direction::PageDown => self.move_down_by(height.saturating_sub(1)),
        }

        self.scroll_location();
    }

    fn move_up_by(&mut self, count: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_sub(count);
        self.snap_to_grapheme();
    }

    fn move_down_by(&mut self, count: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_add(count);
        self.snap_to_grapheme();
        self.snap_to_valid_line();
    }

    fn move_right(&mut self) {
        let line_num = self.buf.lines.len();
        let line_width = self
            .buf
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);

        if self.text_location.grapheme_index < line_width.saturating_sub(1) {
            self.text_location.grapheme_index = self.text_location.grapheme_index.saturating_add(1);
        } else if self.text_location.line_index < line_num {
            self.move_down_by(1);
            self.move_start_of_line();
        }
    }

    fn move_left(&mut self) {
        if self.text_location.grapheme_index > 0 {
            self.text_location.grapheme_index = self.text_location.grapheme_index.saturating_sub(1);
        } else if self.text_location.line_index > 0 {
            self.move_up_by(1);
            self.move_end_of_line();
        }
    }

    fn move_start_of_line(&mut self) {
        self.text_location.grapheme_index = 0;
    }

    fn move_end_of_line(&mut self) {
        self.text_location.grapheme_index = self
            .buf
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count)
            .saturating_sub(1);
    }

    fn snap_to_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .buf
            .lines
            .get(self.text_location.line_index)
            .map_or(0, |line| {
                cmp::min(self.text_location.grapheme_index, line.grapheme_count())
            });
    }

    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index =
            cmp::min(self.text_location.line_index, self.buf.lines.len());
    }

    fn scroll_location(&mut self) {
        let Position {
            x: current_row,
            y: current_line,
        } = self.text_location_to_position();

        self.scroll_orizontally(current_row);
        self.scroll_vertically(current_line);
    }

    fn scroll_orizontally(&mut self, to: usize) {
        let width = self.size.width;

        let offset_changed = if to < self.scroll_offset.x {
            self.scroll_offset.x = to;
            true
        } else if to >= self.scroll_offset.x.saturating_add(width) {
            self.scroll_offset.x = to.saturating_sub(width).saturating_add(1);
            true
        } else {
            false
        };

        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    fn scroll_vertically(&mut self, to: usize) {
        let height = self.size.height;

        let offset_changed = if to < self.scroll_offset.y {
            self.scroll_offset.y = to;
            true
        } else if to >= self.scroll_offset.y.saturating_add(height) {
            self.scroll_offset.y = to.saturating_sub(height).saturating_add(1);
            true
        } else {
            false
        };

        self.needs_redraw = self.needs_redraw || offset_changed;
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

    fn text_location_to_position(&self) -> Position {
        let y = self.text_location.line_index;
        let x = self.buf.lines.get(y).map_or(0, |line| {
            line.width_until(self.text_location.grapheme_index)
        });
        Position { x, y }
    }
}

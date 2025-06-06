use super::{
    editor_cmd::{Direction, EditorCommand},
    terminal::{Position, TerminalSize},
};
use crate::editor::Terminal;
use std::cmp;

mod buffer;
use buffer::Buffer;
mod line;
use line::Line;
mod grapheme;

const EDITOR_NAME: &str = env!("CARGO_PKG_NAME");
const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Rapresents a valid grapheme on the terminal, it is
/// different from position since in only point to a valid
/// character and not to a specific cell in the terminal.
#[derive(Clone, Copy, Default)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

/// This struct rapresents what we are showing on the screen.
/// The field `need_redraw` is needed for when something is changed
/// on the screen and we need to refresh the screen, otherwise nothing
/// is performed.
/// The field `scroll_offset` is needed for enabling scrolling by tracking
/// the offset Position of the origin (0, 0).
pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: TerminalSize,

    text_location: Location,
    scroll_offset: Position,
}

impl View {
    pub fn new() -> Self {
        View {
            needs_redraw: true,
            buffer: Buffer::default(),
            size: Terminal::size().unwrap_or_default(),
            text_location: Location::default(),
            scroll_offset: Position::default(),
        }
    }

    /// In renders the content of the file on the screen with the respective offset
    /// if it is present, otherwise is it gonna simply print
    /// the name of the editor and the version.
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
            if let Some(line) = self.buffer.lines.get(i.saturating_add(sy)) {
                Self::render_line(i, &line.get(sx..sx.saturating_add(width)));
            } else if vertical_center == i && self.buffer.is_empty() {
                Self::draw_title(i, width);
            } else {
                Self::render_line(i, "~");
            }
        }

        self.needs_redraw = false;
    }

    /// Calculates the position of the cursor on the visible
    /// screen subtracting the offset from the position.
    /// (See struct Position definition)
    pub fn cursor_position(&self) -> Position {
        self.text_location_to_position()
            .subtract(&self.scroll_offset)
    }

    /// When we resize the terminal we need to refresh what's
    /// on the screen.
    pub fn resize(&mut self, size: TerminalSize) {
        self.size = size;
        self.needs_redraw = true;
    }

    /// Loads the buffer with the content of the file we are
    /// rendering.
    pub fn load(&mut self, path: &str) {
        if let Ok(buffer) = Buffer::load(path) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }

    /// Handles the `EditorCommand` sent to view.
    pub fn handle_command(&mut self, cmd: EditorCommand) {
        match cmd {
            EditorCommand::Move(cmd) => self.handle_movement(cmd),
            EditorCommand::Resize(size) => self.resize(size),
            _ => unreachable!(),
        }
    }

    fn current_line_len(&self) -> usize {
        self.buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count)
    }

    pub fn handle_insertion(&mut self, sy: char) {
        let old_len = self.current_line_len();
        self.buffer.insert_char(sy, self.text_location);
        let new_len = self.current_line_len();

        #[allow(clippy::arithmetic_side_effects)]
        if new_len - old_len > 0 {
            self.handle_movement(Direction::Right);
            self.needs_redraw = true;
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.text_location.line_index != 0 || self.text_location.grapheme_index != 0 {
            self.handle_movement(Direction::Left);
            self.handle_deletion();
        }
    }

    pub fn handle_deletion(&mut self) {
        self.buffer.delete(self.text_location);
        self.needs_redraw = true;
    }

    pub fn save(&self) {
        let _ = self.buffer.save();
    }

    pub fn handle_enter(&mut self) {
        self.buffer.insert_newline(self.text_location);
        self.handle_movement(Direction::Down);
        self.handle_movement(Direction::Home);
        self.needs_redraw = true;
    }

    /// Handles the movement of view.
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

    /// Enables moving to the right even when reached the end of the line
    /// by moving down by 1.
    fn move_right(&mut self) {
        let line_num = self.buffer.lines.len();
        let line_width = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);

        if self.text_location.grapheme_index < line_width {
            self.text_location.grapheme_index = self.text_location.grapheme_index.saturating_add(1);
        } else if self.text_location.line_index < line_num {
            self.move_down_by(1);
            self.move_start_of_line();
        }
    }

    /// Enables moving to the left even when reached the start of the line
    /// by moving up by 1.
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
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
    }

    /// Avoids the cursor going after the actual lenght of the line
    /// counting the graphemes.
    fn snap_to_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, |line| {
                cmp::min(
                    self.text_location.grapheme_index,
                    line.grapheme_count().saturating_sub(1),
                )
            });
    }

    /// Avoids the cursor going after the actual height of the
    /// entire file.
    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index =
            cmp::min(self.text_location.line_index, self.buffer.lines.len());
    }

    /// Enables scrolling by converting the Location
    /// to the Position and moving towards it.
    /// If the `scroll_offset` is changed we then need to
    /// refresh the screen by setting `needs_redraw` to `true`.
    fn scroll_location(&mut self) {
        let Position {
            x: current_row,
            y: current_line,
        } = self.text_location_to_position();

        self.scroll_orizontally(current_row);
        self.scroll_vertically(current_line);
    }

    /// Sets the `scroll_offset` based on how much we are
    /// far from the Position origin x coordinate.
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

    /// Sets the `scroll_offset` based on how much we are
    /// far from the Position origin y coordinate.
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

    /// Renders a single line on a specific row, in debug if something
    /// goes wrong we report it by panicking.
    fn render_line(row_num: usize, line: &str) {
        let result = Terminal::print_row(row_num, line);
        debug_assert!(result.is_ok(), "Failed to render line.");
    }

    /// Converts the current Location to the correspective Position
    /// on the infinite grid.
    fn text_location_to_position(&self) -> Position {
        let y = self.text_location.line_index;
        let x = self.buffer.lines.get(y).map_or(0, |line| {
            line.width_until(self.text_location.grapheme_index)
        });
        Position { x, y }
    }

    /// Draws the title screen.
    fn draw_title(row_num: usize, width: usize) {
        let msg = format!("{EDITOR_NAME}::{EDITOR_VERSION}");

        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(msg.len())) / 2;
        let padding = " ".repeat(padding.saturating_sub(1));

        let mut msg = format!("~{padding}{msg}");
        msg.truncate(width);

        Self::render_line(row_num, &msg);
    }
}

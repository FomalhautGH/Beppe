use super::{
    editor_cmd::{Direction, EditorCommand},
    terminal::{Position, TerminalSize},
};

use crate::editor::{
    Terminal, annotated_line::AnnotatedLine, document_status::DocumentStatus, line::Line,
    ui_component::UiComponent,
};

use std::cmp;

mod buffer;
use buffer::Buffer;
mod file_info;

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
#[derive(Default)]
pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: TerminalSize,
    text_location: Location,
    scroll_offset: Position,
    search_term: String,
}

impl View {
    /// Calculates the position of the cursor on the visible
    /// screen subtracting the offset from the position.
    /// (See struct Position definition)
    pub fn cursor_position(&self) -> Position {
        self.text_location_to_position()
            .subtract(&self.scroll_offset)
    }

    /// Loads the buffer with the content of the file we are
    /// rendering.
    pub fn load(&mut self, path: &str) -> Result<(), std::io::Error> {
        let buf = Buffer::load(path)?;

        self.buffer = buf;
        self.set_needs_redraw(true);

        Ok(())
    }

    /// Handles the `EditorCommand` sent to view.
    pub fn handle_command(&mut self, cmd: EditorCommand) {
        match cmd {
            EditorCommand::Move(mov) => self.handle_movement(mov),
            EditorCommand::Resize(_) => {}
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
            self.set_needs_redraw(true);
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
        self.set_needs_redraw(true);
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<(), std::io::Error> {
        self.buffer.save_as(file_name)
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        self.buffer.save()
    }

    pub fn is_file_modified(&self) -> bool {
        self.buffer.is_dirty()
    }

    pub fn handle_enter(&mut self) {
        self.buffer.insert_newline(self.text_location);
        self.handle_movement(Direction::Down);
        self.handle_movement(Direction::Home);
        self.set_needs_redraw(true);
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
    fn render_line(row_num: usize, line: &str) -> Result<(), std::io::Error> {
        Terminal::print_row(row_num, line)
    }

    fn render_annotated_line(row_num: usize, line: AnnotatedLine) -> Result<(), std::io::Error> {
        Terminal::print_annotated_row(row_num, line)
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
    fn build_title(width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let msg = format!("{EDITOR_NAME}::{EDITOR_VERSION}");
        let len = msg.len();
        let width_sub1 = width.saturating_sub(1);

        // If the title doesn't fit we simply hide the title screen
        if width_sub1 < len {
            return String::from("~");
        }

        format!("{:<}{:^width_sub1$}", "~", msg)
    }

    pub fn set_search_term(&mut self, term: String) {
        self.search_term = term;
    }

    pub fn search(&mut self) {
        if self.search_term.is_empty() {
            return;
        }

        if let Some(location) = self
            .buffer
            .search_forward(&self.search_term, self.text_location)
        {
            self.text_location = location;
            self.scroll_vertically(self.text_location.line_index);
            self.center_screen();
        }
    }

    pub fn search_next(&mut self) {
        if self.search_term.is_empty() {
            return;
        }
        self.move_right();

        if let Some(location) = self
            .buffer
            .search_forward(&self.search_term, self.text_location)
        {
            self.text_location = location;
            self.scroll_vertically(self.text_location.line_index);
            self.center_screen();
        } else {
            self.move_left();
        }
    }

    pub fn search_prev(&mut self) {
        if self.search_term.is_empty() {
            return;
        }
        self.move_left();

        if let Some(location) = self
            .buffer
            .search_backwards(&self.search_term, self.text_location)
        {
            self.text_location = location;
            self.scroll_vertically(self.text_location.line_index);
            self.center_screen();
        } else {
            self.move_right();
        }
    }

    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            file_name: self.buffer.file_info.to_string(),
            num_of_lines: self.buffer.height(),
            current_line: self.text_location.line_index,
            modified: self.buffer.is_dirty(),
        }
    }

    fn center_screen(&mut self) {
        let TerminalSize { height, width } = self.size;
        let Position { x, y } = self.text_location_to_position();

        let vertical_mid = height.div_ceil(2);
        let horizontal_mid = width.div_ceil(2);

        self.scroll_offset.y = y.saturating_sub(vertical_mid);
        self.scroll_offset.x = x.saturating_sub(horizontal_mid);

        self.set_needs_redraw(true);
    }
}

impl UiComponent for View {
    fn set_needs_redraw(&mut self, val: bool) {
        self.needs_redraw = val;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: TerminalSize) {
        self.size = size;
        self.scroll_location();
    }

    /// In renders the content of the file on the screen with the respective offset
    /// if it is present, otherwise is it gonna simply print
    /// the name of the editor and the version.
    fn draw(&mut self, pos_y: usize) -> Result<(), std::io::Error> {
        let TerminalSize { width, height } = self.size;
        let end_y = pos_y.saturating_add(height);

        #[allow(clippy::integer_division)]
        let vertical_center: usize = height / 3;

        let scroll_top = self.scroll_offset.y;
        for current_row in pos_y..end_y {
            let line_idx = current_row.saturating_sub(pos_y).saturating_add(scroll_top);
            if let Some(line) = self.buffer.lines.get(line_idx) {
                let left = self.scroll_offset.x;
                let right = self.scroll_offset.x.saturating_add(width);
                Self::render_annotated_line(
                    current_row,
                    line.get(
                        left..right,
                        if !self.search_term.is_empty() {
                            Some(&self.search_term)
                        } else {
                            None
                        },
                    ),
                )?;
            } else if current_row == vertical_center && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_title(width))?;
            } else {
                Self::render_line(current_row, "~")?;
            }
        }

        Ok(())
    }
}

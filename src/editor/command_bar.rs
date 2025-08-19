use crate::editor::{
    editor_cmd::Direction,
    line::Line,
    terminal::{Terminal, TerminalSize},
    ui_component::UiComponent,
};

#[derive(Clone, Copy)]
pub enum Cmd {
    SaveAs,
    Search,
}

#[derive(Default)]
pub struct CommandBar {
    prompt: String,
    line: Line,
    command: Option<Cmd>,
    cursor_location: usize,
    needs_redraw: bool,
}

impl CommandBar {
    pub fn set_command(&mut self, command: Cmd) {
        self.command = Some(command);

        self.prompt = match command {
            Cmd::SaveAs => "Save As: ",
            Cmd::Search => "Search: ",
        }
        .to_string();

        self.cursor_location = self.prompt.len();
        self.set_needs_redraw(true);
    }

    pub fn get_command(&self) -> Option<Cmd> {
        self.command
    }

    pub fn get_line(&self) -> String {
        self.line.to_string()
    }

    pub fn clear(&mut self) {
        self.prompt.clear();
        self.line.clear();
        self.set_needs_redraw(true);
    }

    pub fn cursor_location(&self) -> usize {
        self.cursor_location
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_location != 0 {
            self.handle_movement(Direction::Left);
            self.handle_deletion();
        }
    }

    pub fn handle_deletion(&mut self) {
        self.line.pop();
        self.set_needs_redraw(true);
    }

    pub fn handle_insertion(&mut self, sy: char) {
        let old_len = self.line.grapheme_count();
        self.line.push_chr(sy);
        let new_len = self.line.grapheme_count();

        #[allow(clippy::arithmetic_side_effects)]
        if new_len - old_len > 0 {
            self.handle_movement(Direction::Right);
            self.set_needs_redraw(true);
        }
    }

    fn handle_movement(&mut self, mov: Direction) {
        match mov {
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            _ => unreachable!(),
        }
    }

    fn move_left(&mut self) {
        if self.cursor_location > self.prompt.len() {
            self.cursor_location = self.cursor_location.saturating_sub(1);
        }
    }

    fn move_right(&mut self) {
        self.cursor_location = self.cursor_location.saturating_add(1);
    }
}

impl UiComponent for CommandBar {
    fn set_size(&mut self, _size: TerminalSize) {}

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_needs_redraw(&mut self, val: bool) {
        self.needs_redraw = val;
    }

    fn draw(&mut self, pos_y: usize) -> Result<(), std::io::Error> {
        Terminal::print_row(pos_y, &format!("{}{}", self.prompt, self.line))?;
        Ok(())
    }
}

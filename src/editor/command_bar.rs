use crate::editor::{
    editor_cmd::Direction,
    line::Line,
    terminal::{Terminal, TerminalSize},
    ui_component::UiComponent,
};

#[derive(Default)]
pub struct CommandBar {
    prompt: String,
    command: Line,
    cursor_location: usize,
    needs_redraw: bool,
}

impl CommandBar {
    pub fn set_prompt(&mut self, msg: &str) {
        let mut msg = msg.to_string();
        msg.push_str(": ");

        self.prompt = msg;
        self.cursor_location = self.prompt.len();
        self.set_needs_redraw(true);
    }

    pub fn get_command(&self) -> String {
        self.command.to_string()
    }

    pub fn clear(&mut self) {
        self.prompt.clear();
        self.command.clear();
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
        self.command.pop();
        self.set_needs_redraw(true);
    }

    pub fn handle_insertion(&mut self, sy: char) {
        let old_len = self.command.grapheme_count();
        self.command.push_chr(sy);
        let new_len = self.command.grapheme_count();

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
        Terminal::print_row(pos_y, &format!("{}{}", self.prompt, self.command))?;
        Ok(())
    }
}

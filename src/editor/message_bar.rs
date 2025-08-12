use crate::editor::{
    terminal::{Terminal, TerminalSize},
    ui_component::UiComponent,
};

#[derive(Default)]
pub struct MessageBar {
    message: String,
    needs_redraw: bool,
}

impl MessageBar {
    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.needs_redraw = true;
    }
}

impl UiComponent for MessageBar {
    fn set_size(&mut self, _size: TerminalSize) {}

    fn mark_redraw(&mut self, val: bool) {
        self.needs_redraw = val;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn draw(&mut self, pos_y: usize) -> Result<(), std::io::Error> {
        Terminal::print_row(pos_y, &self.message)
    }
}

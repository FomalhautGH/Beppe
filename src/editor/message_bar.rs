use std::time::Instant;

use crate::editor::{
    MESSAGE_DURATION,
    terminal::{Terminal, TerminalSize},
    ui_component::UiComponent,
};

pub struct MessageBar {
    message: String,
    when: Instant,
    needs_redraw: bool,
    cleared_after_expired: bool,
}

impl MessageBar {
    pub fn is_message_expired(&self) -> bool {
        self.when.elapsed() > MESSAGE_DURATION
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.needs_redraw = true;
        self.cleared_after_expired = false;
        self.when = Instant::now();
    }
}

impl UiComponent for MessageBar {
    fn set_size(&mut self, _size: TerminalSize) {}

    fn set_needs_redraw(&mut self, val: bool) {
        self.needs_redraw = val;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw || (self.is_message_expired() && !self.cleared_after_expired)
    }

    fn draw(&mut self, pos_y: usize) -> Result<(), std::io::Error> {
        if self.is_message_expired() {
            self.cleared_after_expired = true;
            Terminal::print_row(pos_y, "")
        } else {
            Terminal::print_row(pos_y, &self.message)
        }
    }
}

impl Default for MessageBar {
    fn default() -> Self {
        Self {
            when: Instant::now(),
            message: String::default(),
            needs_redraw: false,
            cleared_after_expired: false,
        }
    }
}

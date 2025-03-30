use crate::terminal::{Terminal, TerminalSize};
use buffer::Buffer;

mod buffer;

const EDITOR_NAME: &str = env!("CARGO_PKG_NAME");
const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buf: Buffer,
    needs_redraw: bool,
    size: TerminalSize,
}

impl View {
    pub fn new() -> Self {
        View {
            buf: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
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
        let vertical_center: usize = (height / 3).into();

        for i in 0..height.into() {
            if let Some(line) = self.buf.lines.get(i) {
                Self::render_line(i, if line.len() >= width.into() {
                    &line[0..width.into()]
                } else {
                    line
                });
            } else if vertical_center == i && self.buf.is_empty() {
                Self::draw_title(i, width.into());
            } else {
                Self::render_line(i, "~");
            }
        }

        self.needs_redraw = false;
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
}

use buffer::Buffer;
use crate::terminal::Terminal;

mod buffer;

const EDITOR_NAME: &str = env!("CARGO_PKG_NAME");
const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct View {
    buf: Buffer,
}

impl View {
    pub fn load(&mut self, path: &str) {
        if let Ok(buffer) = Buffer::load(path) {
            self.buf = buffer;
        }
    }

    pub fn render(&self) -> Result<(), std::io::Error> {
        if self.buf.is_empty() {
            Self::render_welcome_title()
        } else {
            self.render_buffer()
        }
    }

    fn render_buffer(&self) -> Result<(), std::io::Error> {
        let height = Terminal::size()?.height;
        
        for i in 0..height {
            Terminal::clear_line()?;

            if let Some(line) = self.buf.lines.get::<usize>(i.into()) {
                Terminal::print(line)?;
            } else {
                Self::draw_empty_row()?;
            }

            if i.saturating_add(1) < height {
                Terminal::print("\r\n")?;
            }
        }

        Ok(())
    }

    fn render_welcome_title() -> Result<(), std::io::Error> {
        let height = Terminal::size()?.height;
        
        for i in 0..height {
            Terminal::clear_line()?;

            #[allow(clippy::integer_division)]
            if i == height / 3 {
                Self::draw_title()?;
            } else {
                Self::draw_empty_row()?;
            }

            if i.saturating_add(1) < height {
                Terminal::print("\r\n")?;
            }
        }

        Ok(())
    }

    fn draw_title() -> Result<(), std::io::Error> {
        let width: usize = Terminal::size()?.width.into();
        let msg = format!("{EDITOR_NAME}::{EDITOR_VERSION}");

        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(msg.len())) / 2;
        let padding = " ".repeat(padding.saturating_sub(1));

        let mut msg = format!("~{padding}{msg}");
        msg.truncate(width);

        Terminal::print(&msg)
    }

    fn draw_empty_row() -> Result<(), std::io::Error> {
        Terminal::print("~")
    }
}

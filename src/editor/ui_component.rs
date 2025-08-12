use crate::editor::terminal::TerminalSize;

pub trait UiComponent {
    fn mark_redraw(&mut self, val: bool);
    fn needs_redraw(&self) -> bool;
    fn set_size(&mut self, size: TerminalSize);
    fn draw(&mut self, origin_y: usize) -> Result<(), std::io::Error>;

    fn resize(&mut self, size: TerminalSize) {
        self.set_size(size);
        self.mark_redraw(true);
    }

    fn render(&mut self, pos_y: usize) {
        if self.needs_redraw() {
            match self.draw(pos_y) {
                Ok(()) => self.mark_redraw(false),

                Err(err) => {
                    #[cfg(debug_assertions)]
                    panic!("Could not render component: {err:?}");
                }
            }
        }
    }
}

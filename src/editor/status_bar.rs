use crate::editor::{
    DocumentStatus, EditorMode,
    terminal::{Terminal, TerminalSize},
};

pub struct StatusBar {
    editor_mode: EditorMode,
    doc_status: DocumentStatus,
    pos_y: usize,
    width: usize,
    margin_bottom: usize,
    needs_redraw: bool,
}

impl StatusBar {
    pub fn new(margin_bottom: usize) -> Self {
        let size = Terminal::size().unwrap_or_default();
        Self {
            editor_mode: EditorMode::Normal,
            doc_status: DocumentStatus::default(),
            needs_redraw: true,
            margin_bottom,
            width: size.width,
            pos_y: size.height.saturating_sub(margin_bottom),
        }
    }

    pub fn resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.pos_y = size.height.saturating_sub(self.margin_bottom);
        self.needs_redraw = true;
    }

    pub fn update_status(&mut self, new_status: DocumentStatus) {
        if new_status != self.doc_status {
            self.doc_status = new_status;
            self.needs_redraw = true;
        }
    }

    pub fn update_editor_mode(&mut self, mode: EditorMode) {
        if mode != self.editor_mode {
            self.editor_mode = mode;
            self.needs_redraw = true;
        }
    }

    pub fn render(&mut self) {
        if self.needs_redraw {
            let mut bar = format!("{} {:?}", self.editor_mode, self.doc_status);
            bar.truncate(self.width);
            let result = Terminal::print_row(self.pos_y, &bar);

            debug_assert!(result.is_ok(), "Failed to render line.");
            self.needs_redraw = false;
        }
    }
}

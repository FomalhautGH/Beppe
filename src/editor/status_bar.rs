use crate::editor::{
    EditorMode,
    document_status::DocumentStatus,
    terminal::{Terminal, TerminalSize},
};

pub struct StatusBar {
    editor_mode: EditorMode,
    doc_status: DocumentStatus,
    pos_y: usize,
    width: usize,
    margin_bottom: usize,
    needs_redraw: bool,
    is_visible: bool,
}

impl StatusBar {
    pub fn new(margin_bottom: usize) -> Self {
        let size = Terminal::size().unwrap_or_default();

        let mut status_bar = Self {
            editor_mode: EditorMode::Normal,
            doc_status: DocumentStatus::default(),
            needs_redraw: true,
            is_visible: false,
            margin_bottom,
            width: size.width,
            pos_y: 0,
        };

        status_bar.resize(size);
        status_bar
    }

    pub fn resize(&mut self, size: TerminalSize) {
        self.width = size.width;

        if let Some(result) = size.height.checked_sub(self.margin_bottom) {
            self.pos_y = result;
            self.is_visible = true;
        } else {
            self.pos_y = 0;
            self.is_visible = false;
        }

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
        if !self.needs_redraw || !self.is_visible {
            return;
        }

        if let Ok(size) = Terminal::size() {
            let line_count = self.doc_status.line_count_to_string();
            let modified_indicator = self.doc_status.modified_indicator_to_string();

            let line = format!(
                "{} - {} - {line_count} {modified_indicator}",
                self.doc_status.file_name, self.editor_mode
            );

            let position_indicator = self.doc_status.position_indicator_to_string();
            let remainder_len = size.width.saturating_sub(line.len());
            let status = format!("{line}{position_indicator:>remainder_len$}");
            let to_print = if status.len() <= size.width {
                status
            } else {
                String::new()
            };

            let result = Terminal::print_inverted_row(self.pos_y, &to_print);
            debug_assert!(result.is_ok(), "Failed to render line.");
            self.needs_redraw = false;
        }
    }
}

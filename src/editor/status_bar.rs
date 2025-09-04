use crate::editor::{
    EditorMode,
    document_status::DocumentStatus,
    terminal::{Terminal, TerminalSize},
    ui_component::UiComponent,
};

#[derive(Default)]
pub struct StatusBar {
    editor_mode: EditorMode,
    doc_status: DocumentStatus,
    needs_redraw: bool,
    size: TerminalSize,
}

impl StatusBar {
    pub fn update_editor_mode(&mut self, mode: EditorMode) {
        if mode != self.editor_mode {
            self.editor_mode = mode;
            self.needs_redraw = true;
        }
    }

    pub fn update_status(&mut self, new_status: DocumentStatus) {
        if new_status != self.doc_status {
            self.doc_status = new_status;
            self.set_needs_redraw(true);
        }
    }
}

impl UiComponent for StatusBar {
    fn set_needs_redraw(&mut self, val: bool) {
        self.needs_redraw = val;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: TerminalSize) {
        self.size = size;
    }

    fn draw(&mut self, pos_y: usize) -> Result<(), std::io::Error> {
        let line_count = self.doc_status.line_count_to_string();
        let modified_indicator = self.doc_status.modified_indicator_to_string();

        let line = format!(
            "{} - {} - {line_count} {modified_indicator}",
            self.doc_status.file_name, self.editor_mode,
        );

        let separator = " | ";
        let position_indicator = self.doc_status.position_indicator_to_string();
        let ty = self.doc_status.file_type.to_string();
        let remainder_len = self
            .size
            .width
            .saturating_sub(line.len())
            .saturating_sub(position_indicator.len())
            .saturating_sub(separator.len())
            .saturating_sub(1);

        let status = format!("{line} {ty:>remainder_len$}{separator}{position_indicator}",);
        let to_print = if status.len() <= self.size.width {
            status
        } else {
            String::default()
        };

        Terminal::print_inverted_row(pos_y, &to_print)
    }
}

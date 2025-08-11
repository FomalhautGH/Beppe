#[derive(Clone, Default, PartialEq, Eq)]
pub struct DocumentStatus {
    pub file_name: String,
    pub num_of_lines: usize,
    pub current_line: usize,
    pub modified: bool,
}

impl DocumentStatus {
    pub fn modified_indicator_to_string(&self) -> String {
        if self.modified {
            String::from("(modified)")
        } else {
            String::new()
        }
    }

    pub fn line_count_to_string(&self) -> String {
        format!("{} lines", self.num_of_lines)
    }

    pub fn position_indicator_to_string(&self) -> String {
        format!(
            "{}/{}",
            self.current_line.saturating_add(1),
            self.num_of_lines
        )
    }
}

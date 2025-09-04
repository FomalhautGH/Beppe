use std::fmt::Display;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FileType {
    #[default]
    PlainText,
    Rust,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FileType::PlainText => "Text",
                FileType::Rust => "Rust",
            }
        )
    }
}

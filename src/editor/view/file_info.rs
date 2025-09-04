use std::{
    fmt::{self, Display},
    path::PathBuf,
};

use crate::editor::file_type::FileType;

#[derive(Default, Debug, Clone)]
pub struct FileInfo {
    pub file_type: FileType,
    pub path: Option<PathBuf>,
}

impl FileInfo {
    pub fn from(file_name: &str) -> Self {
        let path = PathBuf::from(file_name);

        let mut file_type = FileType::PlainText;
        if let Some(ext) = path.extension()
            && ext.eq_ignore_ascii_case("rs")
        {
            file_type = FileType::Rust;
        }

        Self {
            file_type,
            path: Some(path),
        }
    }
}

impl Display for FileInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("[No Name]");
        write!(formatter, "{name}")
    }
}

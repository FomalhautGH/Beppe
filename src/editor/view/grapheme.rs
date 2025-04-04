use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
pub enum GraphemeWidth {
    Zero,
    Half,
    Full,
}

impl GraphemeWidth {
    pub const fn saturating_add(self, other: usize) -> usize {
        match self {
            Self::Zero | Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
    }
}

pub struct TextFragment {
    grapheme: String,
    width: GraphemeWidth,
    replacement: Option<char>,
}

impl TextFragment {
    pub fn from(g: &str) -> Self {
        let grapheme = String::from(g);
        match grapheme.width() {
            0 => Self {
                grapheme,
                width: GraphemeWidth::Zero,
                replacement: Some('·'),
            },
            1 => Self {
                grapheme,
                width: GraphemeWidth::Half,
                replacement: None,
            },
            _ => Self {
                grapheme,
                width: GraphemeWidth::Full,
                replacement: None,
            },
        }
    }

    pub fn grapheme(&self) -> &str {
        &self.grapheme
    }

    pub fn width(&self) -> GraphemeWidth {
        self.width
    }

    pub fn replacement(&self) -> Option<char> {
        self.replacement
    }
}

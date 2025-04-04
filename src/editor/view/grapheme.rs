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
        let (width, replacement) = match grapheme.width() {
            0 => {
                if grapheme.chars().next().is_some_and(char::is_control) {
                    (GraphemeWidth::Zero, Some('▯'))
                } else {
                    (GraphemeWidth::Zero, Some('·'))
                }
            }

            1 => {
                if matches!(g, "\t") {
                    (GraphemeWidth::Half, Some(' '))
                } else if grapheme.trim().is_empty() {
                    (GraphemeWidth::Half, Some('␣'))
                } else {
                    (GraphemeWidth::Half, None)
                }
            }

            _ => (GraphemeWidth::Full, None),
        };

        Self {
            grapheme,
            width,
            replacement,
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

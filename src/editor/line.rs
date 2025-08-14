use std::{fmt::Display, ops::Range};
use unicode_segmentation::UnicodeSegmentation;
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

/// Rapresents a single grapheme width its width and
/// replacement character if needed.
pub struct TextFragment {
    grapheme: String,
    width: GraphemeWidth,
    replacement: Option<char>,
}

impl TextFragment {
    /// Creates a `TextFragment` from a &str.
    pub fn from(grapheme: &str) -> Self {
        let owned_grapheme = String::from(grapheme);
        let (width, replacement) = match owned_grapheme.width() {
            0 => {
                if owned_grapheme.chars().next().is_some_and(char::is_control) {
                    (GraphemeWidth::Zero, Some('▯'))
                } else {
                    (GraphemeWidth::Zero, Some('·'))
                }
            }

            1 => {
                if matches!(grapheme, "\t") {
                    (GraphemeWidth::Half, Some('→'))
                } else if owned_grapheme.trim().is_empty() {
                    (GraphemeWidth::Half, Some('␣'))
                } else {
                    (GraphemeWidth::Half, None)
                }
            }

            _ => (GraphemeWidth::Full, None),
        };

        Self {
            grapheme: owned_grapheme,
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

/// Rapresents a Line in our text with a
/// Vector of `TextFragments`.
#[derive(Default)]
pub struct Line {
    line: Vec<TextFragment>,
}

impl Line {
    /// Converts the graphemes in the string to a Vector of
    /// `TextFragmets` and creates a Line with it.
    pub fn from(line_str: &str) -> Self {
        let line: Vec<TextFragment> = line_str.graphemes(true).map(TextFragment::from).collect();
        Self { line }
    }

    /// It returs the String rapresenting the characters
    /// visible in the supplied range.
    pub fn get(&self, range: Range<usize>) -> String {
        if range.start >= range.end {
            return String::new();
        }

        let mut result = String::new();
        let mut current_pos = 0;

        for fragment in &self.line {
            if current_pos >= range.end {
                break;
            }

            let fragment_end = fragment.width().saturating_add(current_pos);

            if fragment_end > range.start {
                if fragment_end > range.end || current_pos < range.start {
                    result.push('⋯');
                } else if let Some(replacement) = fragment.replacement() {
                    result.push(replacement);
                } else {
                    result.push_str(fragment.grapheme());
                }
            }

            current_pos = fragment_end;
        }

        result
    }

    // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.split_off
    pub fn split_off(&mut self, at: usize) -> Self {
        if at > self.line.len() {
            Self::default()
        } else {
            let rem = self.line.split_off(at);
            Self { line: rem }
        }
    }

    pub fn append(&mut self, other: &Self) {
        let mut line = self.to_string();
        let other = other.to_string();
        line.push_str(&other);
        *self = Self::from(&line);
    }

    /// Calculates the width of the characters until a
    /// specific index.
    pub fn width_until(&self, index: usize) -> usize {
        self.line
            .iter()
            .take(index)
            .map(|fragment| match fragment.width() {
                GraphemeWidth::Half | GraphemeWidth::Zero => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    pub fn insert_char_at(&mut self, index: usize, tf: char) {
        self.line.insert(index, TextFragment::from(&tf.to_string()));
    }

    pub fn remove_at(&mut self, index: usize) -> TextFragment {
        self.line.remove(index)
    }

    pub fn grapheme_count(&self) -> usize {
        self.line.len()
    }

    pub fn push_chr(&mut self, value: char) {
        self.line.push(TextFragment::from(&value.to_string()));
    }

    pub fn pop(&mut self) {
        self.line.pop();
    }

    pub fn clear(&mut self) {
        self.line.clear();
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: String = self.line.iter().map(TextFragment::grapheme).collect();
        write!(f, "{result}")
    }
}

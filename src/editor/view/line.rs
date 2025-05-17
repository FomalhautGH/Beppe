use super::grapheme::{GraphemeWidth, TextFragment};
use std::{fmt::Display, ops::Range};
use unicode_segmentation::UnicodeSegmentation;

/// Rapresents a Line in our text with a
/// Vector of `TextFragments`.
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
    
    pub fn append(&mut self, other: &Self) {
        let mut line = self.to_string();
        let other = other.to_string();
        line.push_str(&other);
        *self = Self::from(&line);
    }

    pub fn is_valid_index(&self, index: usize) -> bool {
        self.line.get(index).is_some()
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

    pub fn insert_at(&mut self, index: usize, tf: TextFragment) {
        self.line.insert(index, tf);
    }

    pub fn remove_at(&mut self, index: usize) -> TextFragment {
        self.line.remove(index)
    }

    pub fn grapheme_count(&self) -> usize {
        self.line.len()
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get(0..self.line.len()))
    }
}

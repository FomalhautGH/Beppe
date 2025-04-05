use super::grapheme::{GraphemeWidth, TextFragment};
use std::ops::Range;
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

    pub fn grapheme_count(&self) -> usize {
        self.line.len()
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
}

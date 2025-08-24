use std::{fmt::Display, ops::Range};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

type GraphemeIndex = usize;
type ByteIndex = usize;

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
    start_index: ByteIndex,
}

impl TextFragment {
    /// Creates a `TextFragment` from a &str.
    pub fn from(grapheme: &str, start_index: ByteIndex) -> Self {
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
            start_index,
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
    string: String,
}

impl Line {
    /// Converts the graphemes in the string to a Vector of
    /// `TextFragmets` and creates a Line with it.
    pub fn from(line_str: &str) -> Self {
        let line: Vec<TextFragment> = Self::string_to_fragments(line_str);
        Self {
            line,
            string: line_str.to_string(),
        }
    }

    /// It returs the String rapresenting the characters
    /// visible in the supplied range.
    pub fn get(&self, range: Range<GraphemeIndex>) -> String {
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

    pub fn split_off(&mut self, at: GraphemeIndex) -> Self {
        if let Some(fragment) = self.line.get(at) {
            let rem = self.string.split_off(fragment.start_index);
            self.rebuild_fragments();
            Self::from(&rem)
        } else {
            Self::default()
        }
    }

    pub fn append(&mut self, other: &Self) {
        self.string.push_str(&other.string);
        self.rebuild_fragments();
    }

    /// Calculates the width of the characters until a
    /// specific index.
    pub fn width_until(&self, index: GraphemeIndex) -> GraphemeIndex {
        self.line
            .iter()
            .take(index)
            .map(|fragment| match fragment.width() {
                GraphemeWidth::Half | GraphemeWidth::Zero => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    pub fn insert_char_at(&mut self, index: GraphemeIndex, tf: char) {
        if let Some(fragment) = self.line.get(index) {
            self.string.insert(fragment.start_index, tf);
        } else {
            self.string.push(tf);
        }
        self.rebuild_fragments();
    }

    pub fn remove_at(&mut self, index: GraphemeIndex) {
        if let Some(fragment) = self.line.get(index) {
            let start = fragment.start_index;
            let end = start.saturating_add(fragment.grapheme.len());
            self.string.drain(start..end);
            self.rebuild_fragments();
        }
    }

    pub fn grapheme_count(&self) -> GraphemeIndex {
        self.line.len()
    }

    pub fn pop(&mut self) {
        self.remove_at(self.line.len().saturating_sub(1));
    }

    pub fn push_chr(&mut self, ch: char) {
        self.string.push(ch);
        self.rebuild_fragments();
    }

    pub fn clear(&mut self) {
        self.string.clear();
        self.rebuild_fragments();
    }

    pub fn search_backwards(&self, needle: &str, mut to: GraphemeIndex) -> Option<GraphemeIndex> {
        if self.line.is_empty() {
            return None;
        }

        to = to.saturating_sub(1);
        let (to_byte, grapheme_len) = self.grapheme_index_to_byte_index(to);
        to = to_byte.saturating_add(grapheme_len);

        self.find_all(needle, 0..to)
            .last()
            .map(|(_, grapheme_index)| *grapheme_index)
    }

    pub fn search_forward(&self, needle: &str, from: GraphemeIndex) -> Option<GraphemeIndex> {
        if self.line.is_empty() {
            return None;
        }

        let (start, _) = self.grapheme_index_to_byte_index(from);
        let end = self.string.len();

        self.find_all(needle, start..end)
            .first()
            .map(|(_, grapheme_index)| *grapheme_index)
    }

    fn find_all(&self, needle: &str, range: Range<ByteIndex>) -> Vec<(ByteIndex, GraphemeIndex)> {
        let start = range.start;

        self.string.get(range).map_or_else(Vec::new, |haystack| {
            haystack
                .match_indices(needle)
                .map(|(relative_byte_index, _)| {
                    let absolute_byte_index = relative_byte_index.saturating_add(start);
                    (
                        absolute_byte_index,
                        self.byte_index_to_grapheme_index(absolute_byte_index),
                    )
                })
                .collect()
        })
    }

    fn byte_index_to_grapheme_index(&self, index: ByteIndex) -> GraphemeIndex {
        for (i, fragment) in self.line.iter().enumerate() {
            if index <= fragment.start_index {
                return i;
            }
        }
        panic!("Invalid byte index");
    }

    fn grapheme_index_to_byte_index(&self, index: GraphemeIndex) -> (ByteIndex, usize) {
        debug_assert!(index < self.grapheme_count());
        self.line.get(index).map_or((0, 0), |fragment| {
            (fragment.start_index, fragment.grapheme.len())
        })
    }

    fn rebuild_fragments(&mut self) {
        self.line = Self::string_to_fragments(&self.string);
    }

    fn string_to_fragments(string: &str) -> Vec<TextFragment> {
        string
            .grapheme_indices(true)
            .map(|(i, fragment)| TextFragment::from(fragment, i))
            .collect()
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

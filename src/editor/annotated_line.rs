use crate::editor::line::ByteIndex;
use std::ops::Range;

#[derive(Clone, Copy)]
pub enum AnnotationType {
    None,
    Match,
    SelectedMatch,
}

pub struct AnnotatedLineIterator<'a> {
    pub annotated_line: &'a AnnotatedLine,
    pub index: usize,
}

impl<'a> Iterator for AnnotatedLineIterator<'a> {
    type Item = AnnotatedLinePart<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.annotated_line.line.len() {
            return None;
        }

        let current_annotation: Vec<&Annotation> = self
            .annotated_line
            .annotations
            .iter()
            .filter(|ann| ann.range.start <= self.index && self.index < ann.range.end)
            .collect();
        let current_annotation = current_annotation.last();

        if let Some(ann) = current_annotation {
            self.index = ann.range.end;
            return Some(AnnotatedLinePart {
                str: &self.annotated_line.line[ann.range.start..ann.range.end],
                ty: ann.ty,
            });
        }

        for ann in &self.annotated_line.annotations {
            if ann.range.start >= self.index {
                let start_index = self.index;
                self.index = ann.range.start;
                return Some(AnnotatedLinePart {
                    str: &self.annotated_line.line[start_index..self.index],
                    ty: AnnotationType::None,
                });
            }
        }

        let start_index = self.index;
        self.index = self.annotated_line.line.len();
        Some(AnnotatedLinePart {
            str: &self.annotated_line.line[start_index..],
            ty: AnnotationType::None,
        })
    }
}

pub struct AnnotatedLinePart<'a> {
    pub str: &'a str,
    pub ty: AnnotationType,
}

pub struct Annotation {
    pub range: Range<ByteIndex>,
    pub ty: AnnotationType,
}

#[derive(Default)]
pub struct AnnotatedLine {
    line: String,
    annotations: Vec<Annotation>,
}

impl<'a> IntoIterator for &'a AnnotatedLine {
    type Item = AnnotatedLinePart<'a>;
    type IntoIter = AnnotatedLineIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AnnotatedLineIterator {
            annotated_line: self,
            index: 0,
        }
    }
}

impl AnnotatedLine {
    pub fn from(str: &str) -> Self {
        Self {
            line: str.to_owned(),
            annotations: Vec::new(),
        }
    }

    pub fn push_annotation(&mut self, range: Range<ByteIndex>, ty: AnnotationType) {
        if !range.is_empty() {
            self.annotations.push(Annotation { range, ty });
        }
    }

    pub fn append_str(&mut self, str: &str) {
        self.line.push_str(str);
    }

    pub fn replace(&mut self, range: Range<ByteIndex>, replacement: &str) {
        if range.is_empty() { return; }

        let prev_len = self.line.len();
        self.line.replace_range(range.clone(), replacement);
        let len = self.line.len();

        let diff = len.abs_diff(prev_len);
        if diff == 0 { return; }
        let widened = len > prev_len;

        for ann in self.annotations.iter_mut() {
            let ann_start = &mut ann.range.start;
            let ann_end = &mut ann.range.end;
            if *ann_end <= range.start { continue; }

            if *ann_start >= range.end {
                (*ann_start, *ann_end) = if widened {
                    (ann_start.saturating_add(diff), ann_end.saturating_add(diff))
                } else {
                    (ann_start.saturating_sub(diff), ann_end.saturating_sub(diff))
                }
            } else {
                if widened {
                    *ann_end = ann_end.saturating_add(diff);
                } else {
                    *ann_end = ann_end.saturating_sub(diff);
                }
            }
        }

        self.annotations.retain(|ann| !ann.range.is_empty());
    }

    pub fn get_line(&self) -> &str {
        &self.line
    }
}

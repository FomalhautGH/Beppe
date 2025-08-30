use crate::editor::annotated_line::{AnnotatedLine, Annotation, AnnotationType};

pub struct AnnotatedLinePart<'a> {
    pub str: &'a str,
    pub ty: AnnotationType,
}

pub struct AnnotatedLineIterator<'a> {
    pub annotated_line: &'a AnnotatedLine,
    pub index: usize,
}

impl<'a> Iterator for AnnotatedLineIterator<'a> {
    type Item = AnnotatedLinePart<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.annotated_line.get_line();
        let annotations = self.annotated_line.get_annotations();

        if self.index >= line.len() {
            return None;
        }

        let current_annotation: Vec<&Annotation> = annotations
            .iter()
            .filter(|ann| ann.range.start <= self.index && self.index < ann.range.end)
            .collect();
        let current_annotation = current_annotation.last();

        if let Some(ann) = current_annotation {
            self.index = ann.range.end;
            return Some(AnnotatedLinePart {
                str: &line[ann.range.start..ann.range.end],
                ty: ann.ty,
            });
        }

        for ann in annotations {
            if ann.range.start >= self.index {
                let start_index = self.index;
                self.index = ann.range.start;
                return Some(AnnotatedLinePart {
                    str: &line[start_index..self.index],
                    ty: AnnotationType::None,
                });
            }
        }

        let start_index = self.index;
        self.index = line.len();
        Some(AnnotatedLinePart {
            str: &line[start_index..],
            ty: AnnotationType::None,
        })
    }
}

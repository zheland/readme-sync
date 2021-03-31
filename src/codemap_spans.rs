#![cfg(all(feature = "codemap", feature = "codemap-diagnostic"))]

use std::vec::Vec;

use codemap_diagnostic::SpanLabel;

use crate::CodemapFiles;
#[cfg(feature = "pulldown-cmark")]
use crate::{CMarkSpan, TextSource};

/// Codemap span labels temporary storage used to create diagnostic messages.
#[derive(Debug)]
pub struct CodemapSpans<'a> {
    codemap_files: &'a mut CodemapFiles,
    span_labels: Vec<SpanLabel>,
}

impl<'a> CodemapSpans<'a> {
    /// Creates a new codemap spans storage.
    pub fn new(codemap_files: &'a mut CodemapFiles) -> Self {
        CodemapSpans {
            codemap_files,
            span_labels: Vec::new(),
        }
    }

    /// Returns codemap files storage.
    pub fn codemap_files(&self) -> &CodemapFiles {
        &self.codemap_files
    }

    /// Returns a slice of span labels.
    pub fn span_labels(&self) -> &[SpanLabel] {
        &self.span_labels
    }

    /// Converts this codemap spans to span labels.
    pub fn into_span_labels(self) -> Vec<SpanLabel> {
        self.span_labels
    }

    #[cfg(feature = "pulldown-cmark")]
    /// Generate span labels from the given codemap files and CMark spans.
    pub fn span_labels_from<I>(codemap_files: &'a mut CodemapFiles, iter: I) -> Vec<SpanLabel>
    where
        I: IntoIterator<Item = CMarkSpan<'a>>,
    {
        let mut codemap_spans = Self::new(codemap_files);
        codemap_spans.extend(iter);
        codemap_spans.into_span_labels()
    }
}

trait FileSubSpan {
    fn subspan(&self, range: &core::ops::Range<usize>) -> codemap::Span;
}

impl FileSubSpan for codemap::File {
    fn subspan(&self, range: &core::ops::Range<usize>) -> codemap::Span {
        self.span.subspan(range.start as u64, range.end as u64)
    }
}

#[cfg(feature = "pulldown-cmark")]
impl<'a> Extend<CMarkSpan<'a>> for CodemapSpans<'_> {
    fn extend<T: IntoIterator<Item = CMarkSpan<'a>>>(&mut self, iter: T) {
        use codemap_diagnostic::SpanStyle;

        let iter = iter.into_iter();
        if let Some(upper) = iter.size_hint().1 {
            self.span_labels.reserve(upper);
        }
        for item in iter {
            match item.text_source {
                TextSource::File(file) => {
                    let span = self
                        .codemap_files
                        .get_or_insert_codemap_file(file)
                        .subspan(item.range);
                    self.span_labels.push(SpanLabel {
                        span,
                        style: SpanStyle::Primary,
                        label: None,
                    });
                }
                TextSource::FileDocs(file_docs) => {
                    let span = self
                        .codemap_files
                        .get_or_insert_codemap_docs_file(file_docs)
                        .subspan(item.range);
                    self.span_labels.push(SpanLabel {
                        span,
                        style: SpanStyle::Primary,
                        label: None,
                    });

                    let file = file_docs.file();
                    let file_range = file_docs.remap_to_file(item.range.clone());
                    if let Some(file_range) = file_range {
                        let span = self
                            .codemap_files
                            .get_or_insert_codemap_file(file)
                            .subspan(&file_range);
                        self.span_labels.push(SpanLabel {
                            span,
                            style: SpanStyle::Secondary,
                            label: None,
                        });
                    }
                }
            }
        }
    }
}

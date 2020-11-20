use core::cmp::Ordering;
use core::ops::Range;
use std::string::String;
use std::sync::Arc;
use std::vec::Vec;

#[cfg(all(feature = "syn", feature = "thiserror"))]
use thiserror::Error;

#[cfg(all(feature = "syn", feature = "thiserror"))]
use crate::Config;
use crate::File;

/// Parsed `.rs` file documentation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FileDocs {
    /// The source file.
    file: Arc<File>,
    /// Parsed documentation text.
    docs: String,
    /// Text ranges remap from parsed documentation to the original file contents.
    remap: Vec<TextRemap>,
}

/// The pair of a source and the corresponding target text remap range.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextRemap {
    /// Source text range.
    pub source: Range<usize>,
    /// Target text range.
    pub target: Range<usize>,
}

impl FileDocs {
    /// Creates file documentations from the specified file with the specified features.
    #[cfg(all(feature = "syn", feature = "thiserror"))]
    pub fn from_file(file: Arc<File>, config: &Config<'_>) -> Result<Self, FileDocsFromFileError> {
        use crate::build_attr_docs;

        let file_text = file.text();
        let line_offsets: Vec<_> = file_text
            .split('\n')
            .map(|slice| slice.as_ptr() as usize - file_text.as_ptr() as usize)
            .collect();

        let ast = syn::parse_file(file_text)?;
        let chunks: Result<Vec<_>, _> = ast
            .attrs
            .iter()
            .map(|attr| build_attr_docs(attr, config))
            .collect();
        let chunks = chunks?;

        let (docs, mut remap, _) = chunks.into_iter().flatten().fold(
            (String::new(), Vec::new(), None),
            |(text, mut remap, last), item| {
                let range = item.span.map(|span| {
                    line_offsets[span.start.line] + span.start.column
                        ..line_offsets[span.end.line] + span.end.column
                });
                if let Some(range) = range.clone() {
                    remap.push(TextRemap {
                        source: text.len()..text.len() + item.text.len(),
                        target: range,
                    });
                }
                (
                    text + &item.text,
                    remap,
                    range.map_or_else(|| last, |range| Some(range.end)),
                )
            },
        );

        remap.sort();
        Ok(FileDocs { file, docs, remap })
    }

    /// Returns file file.
    pub fn file(&self) -> &Arc<File> {
        &self.file
    }

    /// Returns file docs.
    pub fn docs(&self) -> &str {
        &self.docs
    }

    /// Returns file remap.
    pub fn remap(&self) -> &[TextRemap] {
        &self.remap
    }

    /// Remaps range from parsed documentation to source file content.
    pub fn remap_to_file(&self, range: Range<usize>) -> Option<Range<usize>> {
        let remap_idx = self
            .remap
            .binary_search_by(|remap| {
                if range.start < remap.source.start {
                    Ordering::Greater
                } else if range.start < remap.source.end {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            })
            .ok()?;

        let remap = &self.remap[remap_idx];
        Some(remap.target.start..remap.target.end)
    }
}

impl PartialOrd for TextRemap {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TextRemap {
    fn cmp(&self, other: &Self) -> Ordering {
        self.source.start.cmp(&other.source.start)
    }
}

/// An error which can occur when creating file documentation form a given file.
#[cfg(all(feature = "syn", feature = "thiserror"))]
#[derive(Clone, Debug, Error)]
pub enum FileDocsFromFileError {
    /// File parsing error
    #[error("File parser error: {0}")]
    SynError(#[from] syn::Error),
    /// Attribute or meta parsing error.
    #[error(transparent)]
    AttrError(#[from] crate::BuildAttrDocsError),
}

use std::sync::Arc;

use crate::{File, FileDocs};

/// Markdown parser text source which may be either readme file contents
/// or package target parsed documentation.
#[derive(Clone, Debug, PartialEq)]
pub enum TextSource {
    /// File text contents
    File(Arc<File>),
    /// File documentation text contents.
    FileDocs(Arc<FileDocs>),
}

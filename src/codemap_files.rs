use std::collections::HashMap;
use std::sync::Arc;

use crate::{File, FileDocs};

/// Storage for codemap and corresponding codemap files.
#[derive(Debug, Default)]
pub struct CodemapFiles {
    codemap: codemap::CodeMap,
    files: HashMap<Arc<File>, Arc<codemap::File>>,
    file_docs: HashMap<Arc<FileDocs>, Arc<codemap::File>>,
}

impl CodemapFiles {
    /// Creates a new codemap files storage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns codemap.
    pub fn codemap(&self) -> &codemap::CodeMap {
        &self.codemap
    }

    /// Returns readme and documentation files.
    pub fn files(&self) -> &HashMap<Arc<File>, Arc<codemap::File>> {
        &self.files
    }

    /// Returns parsed documentation collection.
    pub fn file_docs(&self) -> &HashMap<Arc<FileDocs>, Arc<codemap::File>> {
        &self.file_docs
    }

    /// Inserts the given file into the storage if it is not present, then returns a reference to the appropriate file from codemap.
    pub fn get_or_insert_codemap_file(&mut self, file: &Arc<File>) -> &Arc<codemap::File> {
        use std::string::ToString;

        let codemap = &mut self.codemap;
        self.files.entry(Arc::clone(file)).or_insert_with(|| {
            let path = file.path().to_string_lossy().into_owned();
            codemap.add_file(path, file.text().to_string())
        })
    }

    /// Inserts the given documentation into the storage if it is not present, then returns a reference to the appropriate file from codemap.
    pub fn get_or_insert_codemap_docs_file(
        &mut self,
        file_docs: &Arc<FileDocs>,
    ) -> &Arc<codemap::File> {
        use std::string::ToString;

        let codemap = &mut self.codemap;
        self.file_docs
            .entry(Arc::clone(file_docs))
            .or_insert_with(|| {
                let path = file_docs.file().path().to_string_lossy().into_owned() + "/parsed";
                codemap.add_file(path, file_docs.docs().to_string())
            })
    }
}

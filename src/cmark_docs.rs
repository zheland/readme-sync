use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

use pulldown_cmark::Event;
use thiserror::Error;

use crate::{
    CMarkData, CMarkDataIter, Config, DisallowUrlsWithPrefixError, File, FileDocs,
    FileDocsFromFileError, FileFromPathError, Manifest, Package,
};

/// Parsed documentation Markdown with optionally specified package path and package manifest.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CMarkDocs<P, M> {
    data: CMarkData,
    package_path: P,
    manifest: M,
}

impl<'a> CMarkDocs<&'a Path, &'a Manifest> {
    /// Creates docs from package and default config.
    ///
    /// First it reads docs file by path specified in the package manifest.
    /// Then it parses it with default configuration.
    pub fn from_package_with_default_config(
        package: &'a Package,
    ) -> Result<Self, CMarkDocsFromPackageError> {
        Self::from_package_and_config(package, &Config::default())
    }

    /// Creates docs from package and the specified config.
    ///
    /// First it reads docs file by path specified in the package manifest.
    /// Then it parses it with the specified configuration.
    pub fn from_package_and_config(
        package: &'a Package,
        config: &Config<'_>,
    ) -> Result<Self, CMarkDocsFromPackageError> {
        let path = package.manifest().default_relative_target_path();
        let file = Arc::new(File::from_path(path.to_path_buf(), Some(package.path()))?);
        let package_path = package.path();
        let manifest = package.manifest();
        Ok(Self::from_file_and_config_and_package_path_and_manifest(
            file,
            config,
            package_path,
            manifest,
        )?)
    }
}

impl CMarkDocs<(), ()> {
    /// Creates docs from file and the specified config.
    ///
    /// The method parses a file with the specified configuration.
    pub fn from_file_and_config(
        file: Arc<File>,
        config: &Config<'_>,
    ) -> Result<Self, FileDocsFromFileError> {
        Self::from_file_and_config_and_package_path_and_manifest(file, config, (), ())
    }
}

impl<'a, P, M> CMarkDocs<P, M> {
    /// Adding the specified package path to the docs.
    pub fn with_package_path(self, package_path: &'a Package) -> CMarkDocs<&'a Package, M> {
        CMarkDocs {
            data: self.data,
            package_path,
            manifest: self.manifest,
        }
    }

    /// Adding the specified manifest to the docs.
    pub fn with_manifest(self, manifest: &'a Manifest) -> CMarkDocs<P, &'a Manifest> {
        CMarkDocs {
            data: self.data,
            package_path: self.package_path,
            manifest,
        }
    }

    /// Creates docs from file, config, package path and manifest.
    pub fn from_file_and_config_and_package_path_and_manifest(
        file: Arc<File>,
        config: &Config<'_>,
        package_path: P,
        manifest: M,
    ) -> Result<Self, FileDocsFromFileError> {
        let file_docs = Arc::new(FileDocs::from_file(file, config)?);
        Ok(Self::from_file_docs_and_package_path_and_manifest(
            file_docs,
            package_path,
            manifest,
        ))
    }

    /// Creates docs from file docs content, package path and manifest.
    pub fn from_file_docs_and_package_path_and_manifest(
        file_docs: Arc<FileDocs>,
        package_path: P,
        manifest: M,
    ) -> Self {
        let data = CMarkData::from_file_docs(file_docs);
        Self::from_data_chunks_package_pach_and_manifest(data, package_path, manifest)
    }

    /// Creates docs from CMark items, package path and manifest.
    pub fn from_data_chunks_package_pach_and_manifest(
        data: CMarkData,
        package_path: P,
        manifest: M,
    ) -> Self {
        Self {
            data,
            package_path,
            manifest,
        }
    }

    /// Returns CMark items.
    pub fn data(&self) -> &CMarkData {
        &self.data
    }

    /// Consumes the `CMarkDocs`, returning `CMarkData`.
    pub fn into_data(self) -> CMarkData {
        self.data
    }

    /// Returns the package path.
    pub fn package_path(&self) -> &P {
        &self.package_path
    }

    /// Returns the manifest.
    pub fn manifest(&self) -> &M {
        &self.manifest
    }

    /// Iterate over `CMarkItem`s.
    pub fn iter(&self) -> CMarkDataIter<'_> {
        self.data.iter()
    }

    /// Iterate over pulldown-cmark events.
    pub fn iter_events(&self) -> impl Iterator<Item = &Event<'_>> {
        self.data.iter().filter_map(|item| item.event())
    }

    fn map<F>(mut self, func: F) -> CMarkDocs<P, M>
    where
        F: FnOnce(CMarkData) -> CMarkData,
    {
        self.data = func(self.data);
        self
    }

    fn map_result<F, E>(mut self, func: F) -> Result<CMarkDocs<P, M>, E>
    where
        F: FnOnce(CMarkData) -> Result<CMarkData, E>,
    {
        self.data = func(self.data)?;
        Ok(self)
    }

    /// Concatenate adjacent text events.
    ///
    /// Use this transformation if you deleted some nodes manually
    /// and want to merge the neighboring text nodes.
    ///
    /// This transformation is always applied right after
    /// readme and docs parsing, because some text events remain ununited.
    /// For example Rust attribute parser generate seperate text events
    /// for every line of source code, and pulldown_cmark generate
    /// seperate text events for character entity reference.
    pub fn concat_texts(self) -> CMarkDocs<P, M> {
        self.map(|data| data.concat_texts())
    }

    /// Increment levels of all headings.
    ///
    /// In readme, the first level heading is usually used only for the project title.
    /// The second level header is usually used in for text section headings in readme.
    /// Rustdoc automatically adds the header of a crate name and the first level headers are used for text sections.
    ///
    /// So it is necessary to increase the level of all headings in the documentation in order to synchronize the headings.
    pub fn increment_heading_levels(self) -> CMarkDocs<P, M> {
        self.map(|data| data.increment_heading_levels())
    }

    /// Add a first level heading with the specified text.
    ///
    /// This function could be useful after heading level incremented.
    pub fn add_title(self, text: &str) -> CMarkDocs<P, M> {
        self.map(|data| data.add_title(text))
    }

    /// Remove section with the specified heading text and level and its subsections.
    pub fn remove_section(self, heading: &str, level: u32) -> Self {
        self.map(|data| data.remove_section(heading, level))
    }

    /// Remove the specified fenced code block tag.
    pub fn remove_codeblock_tag(self, tag: &str) -> CMarkDocs<P, M> {
        self.map(|data| data.remove_codeblock_tag(tag))
    }

    /// Remove the specified fenced code block tags.
    pub fn remove_codeblock_tags(self, tags: &[&str]) -> CMarkDocs<P, M> {
        self.map(|data| data.remove_codeblock_tags(tags))
    }

    /// Remove fenced code block tags that are used by `cargo test`.
    ///
    /// See <https://doc.rust-lang.org/rustdoc/documentation-tests.html> for more details.
    pub fn remove_codeblock_rust_test_tags(self) -> CMarkDocs<P, M> {
        self.map(|data| data.remove_codeblock_rust_test_tags())
    }

    /// Use the specified codeblock tag, if they are not specified
    pub fn use_default_codeblock_tag(self, tag: &str) -> CMarkDocs<P, M> {
        self.map(|data| data.use_default_codeblock_tag(tag))
    }

    /// Use rust fenced codeblock highlight as default.
    pub fn use_default_codeblock_rust_tag(self) -> CMarkDocs<P, M> {
        self.map(|data| data.use_default_codeblock_rust_tag())
    }

    /// Remove hidden rust code from rust fenced codeblocks.
    ///
    /// See <https://doc.rust-lang.org/rustdoc/documentation-tests.html#hiding-portions-of-the-example> for more details.
    pub fn remove_hidden_rust_code(self) -> CMarkDocs<P, M> {
        self.map(|data| data.remove_hidden_rust_code())
    }

    /// Returns self if absolute docs links to the specified repository not found,
    /// otherwise returns an error.
    pub fn disallow_absolute_docs_links(
        self,
        package_name: &str,
        documentation_url: &str,
    ) -> Result<CMarkDocs<P, M>, DisallowUrlsWithPrefixError> {
        self.map_result(|data| data.disallow_absolute_docs_links(package_name, documentation_url))
    }

    /// Convert all relative links into absolute ones using
    /// the specified package documentation url as the root address.
    pub fn use_absolute_docs_urls(
        self,
        package_name: &str,
        documentation_url: &str,
    ) -> CMarkDocs<P, M> {
        self.map(|data| data.use_absolute_docs_urls(package_name, documentation_url))
    }
}

impl<'a, P> CMarkDocs<P, &'a Manifest> {
    /// Add a first level heading with the manifest package name.
    ///
    /// This function could be useful after heading level incremented.
    pub fn add_package_title(self) -> CMarkDocs<P, &'a Manifest> {
        let name = self.manifest.package.name.clone();
        self.add_title(&name)
    }

    /// Returns self if absolute docs links to the manifest repository not found,
    /// otherwise returns an error.
    pub fn disallow_absolute_package_docs_links(
        self,
    ) -> Result<CMarkDocs<P, &'a Manifest>, DisallowAbsolutePackageDocsLinksError> {
        let name = self.manifest.package.name.clone();
        let documentation = self
            .manifest
            .package
            .documentation
            .clone()
            .ok_or(DisallowAbsolutePackageDocsLinksError::DocsUrlNotFound)?;
        Ok(self.disallow_absolute_docs_links(&name, &documentation)?)
    }

    /// Convert all relative links into absolute ones
    /// using the manifest package documentation url as the root address.
    pub fn use_absolute_package_docs_urls(
        self,
    ) -> Result<CMarkDocs<P, &'a Manifest>, UseAbsolutePackageDocsUrlsError> {
        let name = self.manifest.package.name.clone();
        let documentation = self
            .manifest
            .package
            .documentation
            .clone()
            .ok_or(UseAbsolutePackageDocsUrlsError::DocsUrlNotFound)?;
        Ok(self.use_absolute_docs_urls(&name, &documentation))
    }

    /// Converts all links with function `func` applied to each link address.
    pub fn map_links<F>(self, func: F, note: impl Into<Cow<'static, str>>) -> Self
    where
        for<'b> F: FnMut(&'b str) -> Cow<'b, str>,
    {
        self.map(|data| data.map_links(func, note))
    }
}

/// An error which can occur when creating docs from package.
#[derive(Debug, Error)]
pub enum CMarkDocsFromPackageError {
    /// File reading failed.
    #[error(transparent)]
    FileError(#[from] FileFromPathError),
    /// Documentation parsing failed.
    #[error("Documentation parsing failed: {0}")]
    ParseError(#[from] FileDocsFromFileError),
}

/// An error which can occur when checking for disallowed absolute package docs links.
#[derive(Clone, Debug, Error)]
pub enum DisallowAbsolutePackageDocsLinksError {
    /// A disallowed prefix found.
    #[error(transparent)]
    DisallowUrlsWithPrefixError(#[from] DisallowUrlsWithPrefixError),
    /// Manifest does not contain `package.documentation` field.
    #[error("Manifest does not contain package.documentation field")]
    DocsUrlNotFound,
}

/// An error which can occur when converting relative links into absolute ones,
/// using the manifest package documentation url as the root address.
#[derive(Clone, Copy, Debug, Error)]
pub enum UseAbsolutePackageDocsUrlsError {
    #[error("Manifest does not contain package.documentation field")]
    DocsUrlNotFound,
}

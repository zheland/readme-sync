//! The `readme-sync` crate makes it easy to add an integration test
//! that checks that your readme and crate documentation are synchronized.
//!
//! # About
//!
//! This crate provides several abstractions for readme and documentation front page content
//! as well as multiple readme and documentation parsing and transformation functions.
//! With them, readme and documentation can be converted
//! to a set of markup nodes that are expected to be the same.
//! Their equality can be checked with the `assert_sync` function,
//! which also provides useful diagnostic messages about the differences found.
//!
//! Documentation parser accepts not only inner doc-comments (`//!`) but also
//! inner doc-attributes (`#[!cfg(...)]` and `#[!cfg_attr(...)]`).
//! This is useful when some doc-tests require certain features to compile and run.
//!
//! # Usage
//!
//! First, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! readme-sync = "0.3.0"
//! ```
//!
//! Then add an integration test using the necessary readme and docs modifiers,
//! and check their synchronization using the `assert_sync` function.
//!
//! The example below is used to test the synchronization
//! of the readme and documentation of this crate.
//! You can copy it and follow the diagnostic messages
//! to adjust the modifiers used and to correct your readme and documentation.
//!
//! ```rust
//! # /*
//! #[cfg(test)]
//! #[test]
//! fn readme_sync_test() {
//! # */
//!     use readme_sync::{assert_sync, CMarkDocs, CMarkReadme, Config, Package};
//!     use std::borrow::ToOwned;
//!
//!     let package = Package::from_path(env!("CARGO_MANIFEST_DIR").into()).unwrap();
//!     let config = Config::from_package_docs_rs_features(&package);
//!     let readme = CMarkReadme::from_package(&package).unwrap();
//!     let docs = CMarkDocs::from_package_and_config(&package, &config).unwrap();
//!
//!     let readme = readme
//!         .remove_badges_paragraph()
//!         .remove_documentation_section()
//!         .remove_codeblock_tag("no_sync")
//!         .disallow_absolute_repository_blob_links()
//!         .unwrap()
//!         .use_absolute_repository_blob_urls()
//!         .unwrap();
//!
//!     let docs = docs
//!         .increment_heading_levels()
//!         .add_package_title()
//!         .remove_codeblock_rust_test_tags()
//!         .use_default_codeblock_rust_tag()
//!         .remove_hidden_rust_code()
//!         .map_links(
//!             |link| match link {
//!                 "CMarkDocs::map_links" => "struct.CMarkDocs.html#method.map_links".into(),
//!                 link => link.into(),
//!             },
//!             "workaround for intra-doc links",
//!         )
//!         .disallow_absolute_package_docs_links()
//!         .unwrap()
//!         .use_absolute_package_docs_urls()
//!         .unwrap();
//!
//!     assert_sync(&readme, &docs);
//! # /*
//! }
//! # */
//! ```
//!
//! Note that both `cargo build` and `cargo test` enable features from dev-dependencies,
//! so if you want to test your crate without them (for example in `no_std` environment)
//! you can use `readme-sync` with `default-features = false`.
//! See [this](#how-to-prevent-readme-sync-dependency-features-enabled-for-dependencies-of-my-crate)
//! FAQ section for more details.
//!
//! # Feature Flags
//!
//! - `codemap` (enabled by default): Enables `codemap` dependency and required
//!   for `assert_sync` and other diagnostic functions.
//! - `codemap-diagnostic` (enabled by default): Enables `codemap-diagnostic` dependency
//!   and required for `assert_sync` and other diagnostic functions.
//! - `glob` (enabled by default): Enables `gloc` dependency and required
//!   for badges detection and methods like `CMarkReadme::remove_badges_paragraph`.
//! - `platforms`: Enables `platforms` dependency and method `Config::with_target_arch_os_env`.
//! - `proc-macro2` (enabled by default): Enables `proc-macro2` dependency
//!   with `span-locations` feature that allows the crate
//!   to show the errors location for source Rust files.
//! - `pulldown-cmark` (enabled by default): Enables `pulldown-cmark` dependency
//!   and required for almost everything except manifest
//!   and documentation parsing and some utility functions.
//! - `serde` (enabled by default): Enables `serde` dependency
//!   and required for manifest deserializing.
//! - `syn` (enabled by default): Enables `syn` dependency and required for documentation parsing.
//! - `thiserror` (enabled by default): Enables `thiserror` dependency
//!   and required by all functions and methods that can return errors.
//! - `toml` (enabled by default): Enables `toml` dependency and required for manifest parsing.
//!
//! # Other crates
//!
//! - [`cargo-sync-readme`]: generates readme section from documentation.
//!   It does not support doc-attributes and does not provide diagnostics for differences found.
//!   But if you just need to synchronize readme and docs text
//!   or check if they are synchronized it might be a better choice.
//! - [`version-sync`]: crate makes it easy to add an integration test that checks
//!   that README.md and documentation are updated when the crate version changes.
//!
//! # FAQ
//!
//! ## Are rust intra-doc links supported?
//!
//! Currently intra-doc link resolution is not supported.
//! References to structures in the documentation can be changed with [`CMarkDocs::map_links`].
//! The pulldown cmark also requires the link address to be specified.
//!
//! ## Why is the example integration test so long and there is no function that would do it all at once?
//!
//! Readme and documentation transformations are very different
//! between different crates and the API of this crate is not yet stabilized.
//!
//! At the moment, however, it supports extensive customization.
//! You can specify the paths to readme and docs, their contents,
//! the features and transformations used, and use your own transformations.
//!
//! So any feedback is welcome!
//!
//! ## Why use `syn` instead of just parsing documentation comments?
//!
//! Because of `cfg` and `cfg_attr` that are useful for documentation tests
//! that require some specific features and can only be compiled with them.
//!
//! ## Why Markdown instead of text comparison?
//!
//! It simplifies the Markdown transformations.
//! Transformations are necessary,
//! because of some differences between readme content and documentation front page
//! including: the presence of a crate name, different heading levels,
//! the presence of badges, different relative url root, etc.
//!
//! # License
//!
//! Licensed under either of
//!
//! - Apache License, Version 2.0
//!   ([LICENSE-APACHE](https://github.com/zheland/readme-sync/blob/master/LICENSE-APACHE) or
//!   [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))
//! - MIT license
//!   ([LICENSE-MIT](https://github.com/zheland/readme-sync/blob/master/LICENSE-MIT) or
//!   [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))
//!
//! at your option.
//!
//! ## Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in the work by you, as defined in the Apache-2.0 license,
//! shall be dual licensed as above, without any
//! additional terms or conditions.
//!
//! [`CMarkDocs::map_links`]: CMarkDocs::map_links
//! [API Documentation]: https://docs.rs/readme-sync
//! [`cargo-sync-readme`]: https://crates.io/crates/cargo-sync-readme
//! [`version-sync`]: https://crates.io/crates/version-sync

#![warn(
    clippy::all,
    rust_2018_idioms,
    missing_copy_implementations,
    missing_debug_implementations,
    single_use_lifetimes,
    missing_docs,
    trivial_casts,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
#![no_std]

extern crate std;

mod badges;
mod cmark_data;
mod cmark_docs;
mod cmark_item;
mod cmark_readme;
mod cmark_util;
mod codemap_files;
mod codemap_spans;
mod config;
mod docs_parser;
mod file;
mod file_docs;
mod manifest;
mod package;
mod sync;
mod tags;
mod text_source;

pub use badges::badge_url_patterns;
pub use cmark_data::{CMarkData, CMarkDataIter, DisallowUrlsWithPrefixError};
pub use cmark_docs::CMarkDocs;
pub use cmark_item::{
    CMarkItem, CMarkItemAsModified, CMarkItemAsRemoved, CMarkItemWithNote, CMarkSpan,
};
pub use cmark_readme::{CMarkReadme, CMarkReadmeFromPackageError};
pub use codemap_files::CodemapFiles;
pub use codemap_spans::CodemapSpans;
pub use config::Config;
pub use docs_parser::{
    build_attr_docs, build_meta_docs, eval_cfg_predicate, BuildAttrDocsError, BuildMetaDocsError,
    EvalCfgPredicateError,
};
pub use docs_parser::{DocsItem, DocsSpan};
pub use file::{File, FileFromPathError};
pub use file_docs::{FileDocs, FileDocsFromFileError, TextRemap};
pub use manifest::{
    BinPathError, Manifest, ManifestBinTarget, ManifestDocsRsMetadata, ManifestLibTarget,
    ManifestPackage, ManifestReadmePath, TomlParseError, TomlReadError,
};
pub use package::Package;
pub use sync::{assert_sync, check_sync, CheckSyncError, MatchFailed};
pub use tags::codeblock_rust_test_tags;
pub use text_source::TextSource;

use cmark_util::IntoStatic;

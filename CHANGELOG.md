# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `CMarkDocs::map_links` function to fix links including intra-doc links.
- `{CMarkDocs,CMarkReadme}::into_data` and `CMarkData::{into_items,from_items}`.

### Fixed
- Fix url with scheme check

## [0.2.0] - 2021-03-31
### Changed
- Better diagnostics readability.
- Fix prefixes from `as_` to `into_` for methods `as_modified`, `as_removed`,
  and function `as_removed_section_if_matched`.

## [0.1.1] - 2020-11-22
### Changed
- Update `platforms` and `pulldown-cmark` dependencies.
- Use the exact version for dependencies with version 0.y.z.

## [0.1.0] - 2020-11-22
### Added
- Crate version tests.
- `CMarkReadme::remove_codeblock_tag` and `CMarkDocs::remove_section` methods.

### Changed
- Text nodes are automatically concatenated after parsing.
- Minimum supported rustc version is reduced from 1.42.0 to 1.40.0.
- Unused features `alloc` and `std` are removed.
- Feature `proc-macro2-span-locations` is replaced to `proc-macro2`.
- Better readme and documentation front page.
- Better features description.

### Fixed
- Fix changelog links

## [0.0.1] - 2020-11-21
### Added
- Package and manifest parsing.
- Readme and documentation parsing.
- Markdown transformations.
- Synchronization status diagnostics.

[Unreleased]: https://github.com/zheland/readme-sync/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/zheland/readme-sync/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/zheland/readme-sync/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zheland/readme-sync/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/zheland/readme-sync/releases/tag/v0.0.1

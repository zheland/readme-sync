# Migration guide

## [0.3.0]
- All features were removed and all the functionality behind them is now enabled by default.
  Their use is no longer needed due to the update of the cargo feature resolver in Rust 2021.
  See [rust-lang/cargo#7916](https://github.com/rust-lang/cargo/issues/7916) for more details.

## [0.2.0]
- `CMarkItemAsModified::as_modified` replaced to `CMarkItemAsModified::into_modified`.
- `CMarkItemAsModified::as_removed` replaced to `CMarkItemAsModified::into_removed`.
- `as_removed_section_if_matched` replaced to `into_removed_section_if_matched`.

[0.3.0]: https://github.com/zheland/readme-sync/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/zheland/readme-sync/compare/v0.1.1...v0.2.0

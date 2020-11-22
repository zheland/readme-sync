# readme-sync

![CI](https://github.com/zheland/readme-sync/workflows/CI/badge.svg)
[![Latest Version](https://img.shields.io/crates/v/readme-sync.svg)](https://crates.io/crates/readme-sync)
[![Documentation](https://docs.rs/readme-sync/badge.svg)](https://docs.rs/readme-sync)
[![GitHub license](https://img.shields.io/crates/l/readme-sync)](https://github.com/zheland/readme-sync/#license)
[![Rust Version](https://img.shields.io/badge/rustc-1.40+-lightgray.svg)](https://blog.rust-lang.org/2020/01/30/Rust-1.40.0.html)

The `readme-sync` crate makes it easy to add an integration test
that checks that your readme and crate documentation are synchronized.

## About

This crate provides several abstractions for readme and documentation content
as well as multiple readme and documentation parsing and transformation functions.
With them, readme and documentation are transformed into a set of markdown nodes
that are expected to be the same.
Their equality can be checked with the `assert_sync` function,
which also provides useful diagnostic messages about the differences found.

It accepts not only inner doc-comments (`//!`) but also
inner doc-attributes (`#[!cfg(...)]` and `#[!cfg_attr(...)]`).
This is useful when some doc-tests require certain features to compile and run.

## Usage

First, add the following to your `Cargo.toml`:

```toml
[dev-dependencies]
readme-sync = "0.0.1"
```

Note that `cargo build` and `cargo test` use features from dev-dependencies,
so if you want to test your crate without them (for example in `no_std` environment)
you can use `readme-sync` with `default-features = false`.
See [this](#how-to-prevent-readme-sync-dependency-features-enabled-for-dependencies-of-my-crate)
FAQ section for more details.

Then add an integration test using the necessary readme and docs modifiers,
and check their synchronization using the `assert_sync` function.

```rust
#[cfg(test)]
#[test]
fn readme_sync_test() {
    use readme_sync::{assert_sync, CMarkDocs, CMarkReadme, Config, Package};

    let package = Package::from_path(env!("CARGO_MANIFEST_DIR").into()).unwrap();
    let config = Config::from_package_docs_rs_features(&package);
    let readme = CMarkReadme::from_package(&package).unwrap();
    let docs = CMarkDocs::from_package_and_config(&package, &config).unwrap();

    let readme = readme
        .remove_badges_paragraph()
        .remove_documentation_section()
        .remove_codeblock_tag("no_sync")
        .disallow_absolute_repository_blob_links()
        .unwrap()
        .use_absolute_repository_blob_urls()
        .unwrap();

    let docs = docs
        .increment_heading_levels()
        .add_package_title()
        .remove_codeblock_rust_test_tags()
        .use_default_codeblock_rust_tag()
        .remove_hidden_rust_code()
        .disallow_absolute_package_docs_links()
        .unwrap()
        .use_absolute_package_docs_urls()
        .unwrap();

    assert_sync(&readme, &docs);
}
```

## Documentation

[API Documentation]

## Feature Flags

By default, all crate features are enabled.

All crate dependencies are optional and are therefore enabled by features:
`codemap`, `codemap-diagnostic`, `glob`, `proc-macro2`, `proc-macro2-span-locations`
`pulldown-cmark`, `serde`, `syn`, `thiserror`, `toml`.

Also, it has `proc-macro2-span-locations` feature that enable `proc-macro2/span-locations`
feature and allow the crate to show the locations of errors for doc-attributes.

## Other crates

- [`cargo-sync-readme`]: generates readme section from documentation.
  It does not support doc-attributes and does not provide diagnostics.
  But if you just need to synchronize readme and docs text
  or check if they are synchronized it might be a better choice.
- [`version-sync`]: crate makes it easy to add an integration test that checks
  that README.md and documentation are updated when the crate version changes.

## FAQ

### Why is the example integration test so long and there is no function that does all this by itself?

Currently, this crate is at an early stage of development
and common readme and documentation transformations are not yet stabilized.

At the moment, however, it supports wide customization.
You can specify the paths to readme and docs, their contents,
the features and transformations used, or use your own transformations.

Any kind of feedback is welcome!

### Why use `syn` instead of just parsing documentation comments?

Because of `cfg` and `cfg_attr` that are useful for documentation tests
that require some specific features and can only be compiled with them.

### Why Markdown instead of text comparison?

It simplifies the Markdown transformations.
Transformations are necessary,
because of some differences between readme content and documentation
including: the presence of a crate name, different heading levels,
the presence of badges, different relative url root, etc.

### Why are all dependencies optional?

By default, Rust compiler enable features from dev-dependencies for normal dependencies
for commands like `cargo test` and `cargo build`.
As a result, the features used by dev-dependencies are implicitly enabled during testing.
Because in `readme-sync` all dependencies are optional,
you can easily protect your crate from implicitly enabled common features.

See [rust-lang/cargo#7916](https://github.com/rust-lang/cargo/issues/7916) for more details.

### How to prevent `readme-sync` dependency features enabled for dependencies of my crate.

If you use nightly Rust you can simply use `-Z features=dev_dep` flags.

Or, in any Rust release, you can disable all `readme-sync` dependencies with:
```toml
[dev-dependencies.readme-sync]
version = "0.0.1"
default-features = false
```

This will help you avoid feature injection from dev-dependencies.

In order to use `readme-sync` functionality in this case,
you need to add a feature that reenables `readme-sync` default features
and can be used to run readme synchronization integration tests:
```toml,no_sync
[features]
test-readme-sync = ["readme-sync/default"]
```

Then you need to add `test-readme-sync` conditional check to your readme sync integration test:
```rust
#[cfg(all(test, feature = "test-readme-sync"))]
//    ^^^^    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
#[test]
fn readme_sync_test() {
    // ...
}
```

And run it with
```bash
cargo test --features "test-readme-sync"
```

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or
  [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or
  [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any
additional terms or conditions.

[API Documentation]: https://docs.rs/readme-sync
[`cargo-sync-readme`]: https://crates.io/crates/cargo-sync-readme
[`version-sync`]: https://crates.io/crates/version-sync

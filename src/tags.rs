/// Returns a slice of currently known tags
/// used by `cargo test` in Markdown fenced code blocks.
///
/// See <https://doc.rust-lang.org/rustdoc/documentation-tests.html> for more details.
pub fn codeblock_rust_test_tags() -> &'static [&'static str] {
    &[
        "ignore",
        "no_run",
        "should_panic",
        "compile_fail",
        "edition2015",
        "edition2018",
    ]
}

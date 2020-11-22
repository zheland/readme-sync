#![cfg(feature = "pulldown-cmark")]

use core::slice::Iter;
use std::borrow::Cow;
use std::string::String;
use std::sync::Arc;
use std::vec::Vec;

#[cfg(feature = "thiserror")]
use thiserror::Error;

use crate::{CMarkItem, File, FileDocs, TextSource};

/// A `CMarkItem`s container storing a list of events with multiple transformation functions.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CMarkData(Vec<Arc<CMarkItem>>);

/// A `CMarkItem`s container iterator.
pub type CMarkDataIter<'a> = Iter<'a, Arc<CMarkItem>>;

impl CMarkData {
    /// Creates `CMarkData` from the specified `File`.
    pub fn from_file(file: Arc<File>) -> Self {
        Self::from_text_source(TextSource::File(file))
    }

    /// Creates `CMarkData` from the specified `FileDocs`.
    pub fn from_file_docs(file_docs: Arc<FileDocs>) -> Self {
        Self::from_text_source(TextSource::FileDocs(file_docs))
    }

    /// Creates `CMarkData` from the specified `TextSource`.
    pub fn from_text_source(text_source: TextSource) -> Self {
        use crate::IntoStatic;
        use pulldown_cmark::Parser;

        let text = match &text_source {
            TextSource::File(file) => file.text(),
            TextSource::FileDocs(file_docs) => file_docs.docs(),
        };

        Self(
            Parser::new(text)
                .into_offset_iter()
                .map(|(event, range)| {
                    CMarkItem::from(event.into_static(), range, text_source.clone())
                })
                .collect(),
        )
        .concat_texts()
    }

    /// Iterate over `CMarkItem`s.
    pub fn iter(&self) -> CMarkDataIter<'_> {
        self.0.iter()
    }

    fn map<F>(self, func: F) -> Self
    where
        F: FnMut(Arc<CMarkItem>) -> Arc<CMarkItem>,
    {
        Self(self.0.into_iter().map(func).collect())
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
    pub fn concat_texts(self) -> Self {
        use core::mem::take;
        use pulldown_cmark::Event;

        let mut result = Vec::new();
        let mut text_nodes = Vec::new();
        let mut text_value = String::new();

        for node in self.0.into_iter() {
            match node.event() {
                None => {
                    result.push(node);
                }
                Some(Event::Text(event_text)) => {
                    text_value += &event_text;
                    text_nodes.push(node);
                }
                Some(_) => {
                    if let Some(text_node) =
                        merge_text_nodes(take(&mut text_nodes), take(&mut text_value))
                    {
                        result.push(text_node);
                    }
                    result.push(node);
                }
            }
        }

        Self(result)
    }
}

fn merge_text_nodes(nodes: Vec<Arc<CMarkItem>>, text: String) -> Option<Arc<CMarkItem>> {
    use crate::CMarkItemAsModified;
    use pulldown_cmark::{CowStr, Event};

    match nodes.len() {
        0 => None,
        1 => Some(nodes.into_iter().next().unwrap()),
        _ => Some(nodes.as_modified(
            Event::Text(CowStr::Boxed(text.into_boxed_str())),
            Cow::from("concat_texts()"),
        )),
    }
}

impl CMarkData {
    /// Increment levels of all headings.
    ///
    /// In readme, the first level heading is usually used only for the project title.
    /// The second level header is usually used in for text section headings in readme.
    /// Rustdoc automatically adds the header of a crate name and the first level headers are used for text sections.
    ///
    /// So it is necessary to increase the level of all headings in the documentation in order to synchronize the headings.
    pub fn increment_heading_levels(self) -> Self {
        use crate::CMarkItemAsModified;
        use pulldown_cmark::{Event, Tag};

        self.map(|node| {
            let event = match node.event() {
                Some(Event::Start(Tag::Heading(level))) => {
                    Some(Event::Start(Tag::Heading(level + 1)))
                }
                Some(Event::End(Tag::Heading(level))) => Some(Event::End(Tag::Heading(level + 1))),
                _ => None,
            };
            if let Some(event) = event {
                node.as_modified(event, Cow::from("increment_heading_levels()"))
            } else {
                node
            }
        })
    }

    /// Add a first level heading with the specified text.
    ///
    /// This function could be useful after heading level incremented.
    pub fn add_title(self, text: &str) -> Self {
        use pulldown_cmark::{CowStr, Event, Tag};
        use std::string::ToString;

        let heading = std::vec![
            CMarkItem::new(Event::Start(Tag::Heading(1)), Cow::from("add_title()")),
            CMarkItem::new(
                Event::Text(CowStr::Boxed(text.to_string().into_boxed_str())),
                Cow::from("add_title()")
            ),
            CMarkItem::new(Event::End(Tag::Heading(1)), Cow::from("add_title()")),
        ];

        Self(heading.into_iter().chain(self.0.into_iter()).collect())
    }

    /// Removes first paragraph that contains only images and image-links,
    /// if the specified predicate returns true when passing image urls to it.
    #[allow(clippy::match_like_matches_macro)] // requires minimum rustc version 1.42.0
    pub fn remove_images_only_paragraph<P>(self, mut predicate: P) -> Self
    where
        P: FnMut(&[&str]) -> bool,
    {
        use crate::CMarkItemAsRemoved;
        use core::mem::take;
        use pulldown_cmark::{Event, Tag};
        use std::string::ToString;

        let mut result = Vec::new();
        let mut paragraph = Vec::new();
        let mut image_urls = Vec::new();
        let mut is_image = false;
        let mut is_already_removed = false;

        for node in self.0.into_iter() {
            if is_already_removed {
                result.push(node);
                continue;
            }

            if !paragraph.is_empty() {
                if is_image {
                    let event = node.event();
                    is_image = if let Some(Event::End(Tag::Image(..))) = event {
                        false
                    } else {
                        true
                    };
                    paragraph.push(node);
                } else {
                    paragraph.push(node);
                    let node = paragraph.last().unwrap();
                    let event = node.event();
                    match event {
                        Some(Event::End(Tag::Paragraph)) => {
                            let urls: Vec<String> = take(&mut image_urls);
                            let urls: Vec<&str> = urls.iter().map(|url| url.as_str()).collect();
                            if !urls.is_empty() && predicate(&urls) {
                                result.push(
                                    take(&mut paragraph)
                                        .as_removed(Cow::from("remove_images_only_paragraphs()")),
                                );
                                is_already_removed = true;
                            } else {
                                result.append(&mut paragraph);
                            }
                        }
                        Some(Event::Start(Tag::Image(_, url, _))) => {
                            image_urls.push(url.as_ref().to_string());
                            is_image = true;
                        }
                        Some(Event::Start(Tag::Link(..)))
                        | Some(Event::End(Tag::Link(..)))
                        | Some(Event::SoftBreak)
                        | None => {}
                        Some(_) => {
                            result.append(&mut paragraph);
                        }
                    }
                }
            } else {
                let event = node.event();
                match event {
                    Some(Event::Start(Tag::Paragraph)) => paragraph.push(node),
                    _ => result.push(node),
                }
            }
        }

        result.append(&mut paragraph);

        Self(result)
    }

    /// Removes first paragraph that contains only badges.
    #[cfg(feature = "glob")]
    pub fn remove_badges_paragraph(self) -> Self {
        let patterns = crate::badge_url_patterns();
        self.remove_images_only_paragraph(|image_urls| {
            image_urls
                .iter()
                .any(|url| patterns.iter().any(|pattern| pattern.matches(url)))
        })
    }

    /// Remove section with the specified heading text and level and its subsections.
    pub fn remove_section(self, heading: &str, level: u32) -> Self {
        use core::mem::take;
        use pulldown_cmark::{Event, Tag};

        let mut section = Vec::new();
        let mut result = Vec::new();
        let mut is_already_removed = false;

        for node in self.0.into_iter() {
            if !is_already_removed {
                let event = node.event();
                if let Some(Event::Start(Tag::Heading(node_level))) = event {
                    if *node_level <= level {
                        let (mut section, is_removed) =
                            as_removed_section_if_matched(take(&mut section), &heading, level);
                        result.append(&mut section);
                        is_already_removed = is_removed;
                    }
                }
            }
            if is_already_removed {
                result.push(node);
            } else {
                section.push(node);
            }
        }

        result.append(&mut as_removed_section_if_matched(take(&mut section), &heading, level).0);

        Self(result)
    }

    /// Remove sections with heading `Documentation` and level 2.
    pub fn remove_documentation_section(self) -> Self {
        self.remove_section("Documentation", 2)
    }
}

fn as_removed_section_if_matched(
    section: Vec<Arc<CMarkItem>>,
    heading: &str,
    level: u32,
) -> (Vec<Arc<CMarkItem>>, bool) {
    use crate::CMarkItemAsRemoved;
    use std::vec;

    if is_matched_section(&section, heading, level) {
        (
            vec![section.as_removed(Cow::from(std::format!(
                "remove_section(name = \"{}\", level = {})",
                heading,
                level
            )))],
            true,
        )
    } else {
        (section, false)
    }
}

fn is_matched_section(section: &[Arc<CMarkItem>], heading: &str, level: u32) -> bool {
    use pulldown_cmark::{Event, Tag};

    let first_event = section.get(0).and_then(|node| node.event());
    let second_event = section.get(1).and_then(|node| node.event());
    if let (Some(Event::Start(Tag::Heading(node_level))), Some(Event::Text(node_text))) =
        (first_event, second_event)
    {
        *node_level == level && node_text.as_ref() == heading
    } else {
        false
    }
}

impl CMarkData {
    /// Returns self if absolute blob links to the specified repository not found,
    /// otherwise returns an error.
    #[cfg(feature = "thiserror")]
    pub fn disallow_absolute_blob_links(
        self,
        repository_url: &str,
    ) -> Result<Self, DisallowUrlsWithPrefixError> {
        self.disallow_urls_with_prefix(&blob_path_prefix(repository_url))
    }

    /// Returns self if absolute docs links to the specified repository not found,
    /// otherwise returns an error.
    #[cfg(feature = "thiserror")]
    pub fn disallow_absolute_docs_links(
        self,
        package_name: &str,
        documentation_url: &str,
    ) -> Result<Self, DisallowUrlsWithPrefixError> {
        self.disallow_urls_with_prefix(&docs_path_prefix(package_name, documentation_url))
    }

    /// Returns self if links with the specified prefix not found, otherwise returns an error.
    #[cfg(feature = "thiserror")]
    pub fn disallow_urls_with_prefix(
        self,
        prefix: &str,
    ) -> Result<Self, DisallowUrlsWithPrefixError> {
        use pulldown_cmark::{Event, Tag};
        use std::string::ToString;

        for node in &self.0 {
            if let Some(Event::Start(Tag::Link(_, url, _))) = node.event() {
                if url.starts_with(&*prefix) {
                    return Err(DisallowUrlsWithPrefixError::PrefixFound {
                        url: url.as_ref().to_string(),
                        prefix: prefix.to_string(),
                    });
                }
            }
        }

        Ok(self)
    }

    /// Convert all relative links into absolute ones using
    /// the repository url as the root address.
    pub fn use_absolute_blob_urls(self, repository_url: &str) -> Self {
        self.with_absolute_urls(&blob_path_prefix(repository_url))
    }

    /// Convert all relative links into absolute ones using
    /// the package documentation url as the root address.
    pub fn use_absolute_docs_urls(self, package_name: &str, documentation_url: &str) -> Self {
        self.with_absolute_urls(&docs_path_prefix(package_name, documentation_url))
    }

    /// Convert all relative links into absolute ones using specified url prefix.
    pub fn with_absolute_urls(self, prefix: &str) -> Self {
        use crate::CMarkItemAsModified;
        use pulldown_cmark::CowStr;
        use pulldown_cmark::{Event, Tag};

        fn add_link_prefix<'a>(tag: &Tag<'a>, prefix: &str) -> Option<Tag<'a>> {
            if let Tag::Link(ty, url, title) = tag {
                if !is_absolute_url(url) && !is_fragment(url) {
                    return Some(Tag::Link(
                        *ty,
                        CowStr::Boxed([prefix, url].concat().into_boxed_str()),
                        title.clone(),
                    ));
                }
            }
            None
        }

        self.map(|node| {
            let event = match node.event() {
                Some(Event::Start(tag)) => add_link_prefix(tag, &prefix).map(Event::Start),
                Some(Event::End(tag)) => add_link_prefix(tag, &prefix).map(Event::End),
                _ => None,
            };
            match event {
                Some(event) => node.as_modified(
                    event,
                    Cow::from(std::format!("with_absolute_urls(prefix = \"{}\")", prefix)),
                ),
                None => node,
            }
        })
    }
}

fn is_absolute_url(url: &str) -> bool {
    is_url_with_scheme(url)
}

fn is_fragment(url: &str) -> bool {
    url.starts_with('#')
}

#[allow(clippy::match_like_matches_macro)] // requires minimum rustc version 1.42.0
fn is_url_with_scheme(url: &str) -> bool {
    if let Some(colon) = url.find(':') {
        colon > 0
            && if let b'a'..=b'z' | b'A'..=b'Z' = url.as_bytes()[0] {
                true
            } else {
                false
            }
            && url.as_bytes()[1..colon].iter().all(|ch| {
                if let b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'+' | b'.' | b'-' = ch {
                    true
                } else {
                    false
                }
            })
    } else {
        false
    }
}

fn without_trailing_slash(value: &str) -> &str {
    match value.as_bytes().last() {
        Some(b'/') => &value[..value.len() - 1],
        _ => value,
    }
}

fn blob_path_prefix(repository_url: &str) -> String {
    use std::string::ToString;
    without_trailing_slash(repository_url).to_string() + "/blob/master/"
}

fn docs_path_prefix(package_name: &str, documentation_url: &str) -> String {
    use std::string::ToString;

    let url = without_trailing_slash(documentation_url);
    let name = package_name.to_string().replace('-', "_");
    [&url, "/*/", &name, "/"].concat()
}

impl CMarkData {
    /// Remove the specified fenced code block tag.
    pub fn remove_codeblock_tag(self, tag: &str) -> Self {
        self.remove_codeblock_tags(&[tag])
    }

    /// Remove the specified fenced code block tags.
    pub fn remove_codeblock_tags(self, tags: &[&str]) -> Self {
        use crate::CMarkItemAsModified;
        use pulldown_cmark::Event;

        self.map(|node| {
            let event = match node.event() {
                Some(Event::Start(tag)) => remove_codeblock_tag_tags(tag, &tags).map(Event::Start),
                Some(Event::End(tag)) => remove_codeblock_tag_tags(tag, &tags).map(Event::End),
                _ => None,
            };
            match event {
                Some(event) => node.as_modified(
                    event,
                    Cow::from(std::format!("remove_codeblock_tags(tags = {:?})", tags)),
                ),
                None => node,
            }
        })
    }
}

fn remove_codeblock_tag_tags<'a>(
    event_tag: &pulldown_cmark::Tag<'a>,
    tags: &[&str],
) -> Option<pulldown_cmark::Tag<'a>> {
    use pulldown_cmark::{CodeBlockKind, CowStr, Tag};

    if let Tag::CodeBlock(CodeBlockKind::Fenced(ref node_tags)) = event_tag {
        let has_tags = node_tags
            .split(',')
            .any(|node_tag| tags.iter().any(|tag| &node_tag == tag));
        if has_tags {
            let node_tags: Vec<_> = node_tags
                .split(',')
                .filter(|node_tag| !tags.iter().any(|tag| node_tag == tag))
                .collect();
            let node_tags = CowStr::Boxed(node_tags.join(",").into_boxed_str());
            return Some(Tag::CodeBlock(CodeBlockKind::Fenced(node_tags)));
        }
    }
    None
}

impl CMarkData {
    /// Remove fenced code block tags that are used by `cargo test`.
    ///
    /// See <https://doc.rust-lang.org/rustdoc/documentation-tests.html> for more details.
    pub fn remove_codeblock_rust_test_tags(self) -> Self {
        use crate::codeblock_rust_test_tags;

        self.remove_codeblock_tags(codeblock_rust_test_tags())
    }

    /// Use the specified codeblock tag, if they are not specified
    pub fn use_default_codeblock_tag(self, tag: &str) -> Self {
        use crate::CMarkItemAsModified;
        use pulldown_cmark::Event;

        self.map(|node| {
            let event = match node.event() {
                Some(Event::Start(node_tag)) => {
                    map_default_codeblock_tag(node_tag, &tag).map(Event::Start)
                }
                Some(Event::End(node_tag)) => {
                    map_default_codeblock_tag(node_tag, &tag).map(Event::End)
                }
                _ => None,
            };
            match event {
                Some(event) => node.as_modified(
                    event,
                    Cow::from(std::format!("use_default_codeblock_tag(tag = \"{}\")", tag)),
                ),
                None => node,
            }
        })
    }
}

fn map_default_codeblock_tag<'a>(
    event_tag: &pulldown_cmark::Tag<'a>,
    tag: &str,
) -> Option<pulldown_cmark::Tag<'a>> {
    use pulldown_cmark::{CodeBlockKind, CowStr, Tag};
    use std::string::ToString;

    if let Tag::CodeBlock(CodeBlockKind::Fenced(ref node_tag)) = event_tag {
        if node_tag.as_ref() == "" {
            return Some(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Boxed(
                tag.to_string().into_boxed_str(),
            ))));
        }
    }
    None
}

impl CMarkData {
    /// Use rust fenced codeblock highlight as default.
    pub fn use_default_codeblock_rust_tag(self) -> Self {
        self.use_default_codeblock_tag("rust")
    }

    /// Remove hidden rust code from rust fenced codeblocks.
    ///
    /// See <https://doc.rust-lang.org/rustdoc/documentation-tests.html#hiding-portions-of-the-example> for more details.
    pub fn remove_hidden_rust_code(self) -> Self {
        use crate::CMarkItemAsModified;
        use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};

        let mut is_rust_codeblock = false;

        self.map(|node| {
            match node.event() {
                Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(tags)))) => {
                    is_rust_codeblock |= tags.split(',').any(|tag| tag == "rust")
                }
                Some(Event::End(Tag::CodeBlock(..))) => {
                    is_rust_codeblock = false;
                }
                Some(Event::Text(text)) => {
                    if is_rust_codeblock {
                        let text: Vec<_> = text
                            .split('\n')
                            .filter(|line| *line != "#" && !line.starts_with("# "))
                            .collect();
                        let text = text.join("\n");
                        let event = Event::Text(CowStr::Boxed(text.into_boxed_str()));
                        return node.as_modified(event, Cow::from("remove_hidden_rust_code()"));
                    }
                }
                _ => {}
            };
            node
        })
    }
}

/// An error which can occur when checking for disallowed link prefixes.
#[cfg(feature = "thiserror")]
#[derive(Clone, Debug, Error)]
pub enum DisallowUrlsWithPrefixError {
    /// A prefix found
    #[error("The url `{url}` use a prohibited prefix `{prefix}`.")]
    PrefixFound {
        /// Full url
        url: String,
        /// Disallowed prefix
        prefix: String,
    },
}

#![cfg(all(
    feature = "codemap",
    feature = "codemap-diagnostic",
    feature = "pulldown-cmark",
    feature = "thiserror",
))]

use core::fmt::Display;
use std::path::Path;
use std::string::String;

use thiserror::Error;

use crate::{CMarkDocs, CMarkReadme};

/// Asserts that the given readme and docs are the same.
pub fn assert_sync<M1, M2>(readme: &CMarkReadme<&Path, M1>, docs: &CMarkDocs<&Path, M2>) {
    match check_sync(readme, docs) {
        Ok(()) => {}
        Err(CheckSyncError::MatchFailed(err)) => {
            err.emit_to_stderr_colored();
            panic!();
        }
    }
}

/// Returns `Ok(())` if the given readme and docs are the same, and `Err(CheckSyncError)` otherwise.
pub fn check_sync<P1, P2, M1, M2>(
    readme: &CMarkReadme<P1, M1>,
    docs: &CMarkDocs<P2, M2>,
) -> Result<(), CheckSyncError> {
    use std::vec::Vec;

    let mut readme_iter = readme.iter();
    let mut docs_iter = docs.iter();
    let mut matched_events = Vec::new();

    loop {
        let NextItem {
            node: readme_node,
            event: readme_event,
            removed: readme_removed_nodes,
        } = next_node(&mut readme_iter);

        let NextItem {
            node: docs_node,
            event: docs_event,
            removed: docs_removed_nodes,
        } = next_node(&mut docs_iter);

        if readme_node.is_none() && docs_node.is_none() {
            break;
        }

        if readme_event == docs_event {
            matched_events.push(readme_event.unwrap());
        } else {
            use crate::CodemapFiles;
            use std::sync::Arc;

            let mut codemap_files = CodemapFiles::new();
            let mut diags = std::vec![node_not_mached_diagnostic(
                &mut codemap_files,
                &readme_node,
                &docs_node,
            )];

            diags.extend(
                removed_nodes_note(&mut codemap_files, &readme_removed_nodes, "readme").into_iter(),
            );

            diags.extend(
                removed_nodes_note(&mut codemap_files, &docs_removed_nodes, "docs").into_iter(),
            );

            if let (Some(readme_event), Some(docs_event)) = (readme_event, docs_event) {
                diags.append(&mut event_diff_notes(&readme_event, &docs_event));
            }

            diags.push(previous_events_notes(&matched_events));

            let codemap_files = Arc::new(codemap_files);
            return Err(CheckSyncError::MatchFailed(MatchFailed {
                diags,
                codemap_files,
            }));
        }
    }
    Ok(())
}

/// An error which can occur when checking readme and docs for equality.
#[cfg(feature = "thiserror")]
#[derive(Clone, Debug, Error)]
pub enum CheckSyncError {
    /// Readme and docs are not the same.
    #[error(
        "CMarkReadme and CMarkDocs nodes are not the same. \
         Use `MatchFailed::emit_to_stderr` for details."
    )]
    MatchFailed(MatchFailed),
}

/// Readme and docs match failed diagnostics and codemap files.
#[derive(Clone, Debug)]
pub struct MatchFailed {
    diags: std::vec::Vec<codemap_diagnostic::Diagnostic>,
    codemap_files: std::sync::Arc<crate::CodemapFiles>,
}

impl MatchFailed {
    /// Print dianostic messages to console with colors.
    pub fn emit_to_stderr_colored(&self) {
        use codemap_diagnostic::{ColorConfig, Emitter};

        let mut emitter = Emitter::stderr(ColorConfig::Always, Some(&self.codemap_files.codemap()));
        emitter.emit(&self.diags);
    }
}

impl Display for MatchFailed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use codemap_diagnostic::Emitter;
        use std::vec::Vec;

        let mut raw = Vec::new();
        {
            let mut emitter = Emitter::vec(&mut raw, Some(&self.codemap_files.codemap()));
            emitter.emit(&self.diags);
        }
        let msg = String::from_utf8_lossy(&raw);
        write!(f, "{}", msg)
    }
}

struct NextItem<'a> {
    node: Option<std::sync::Arc<crate::CMarkItem>>,
    event: Option<pulldown_cmark::Event<'a>>,
    removed: std::vec::Vec<std::sync::Arc<crate::CMarkItem>>,
}

fn next_node<'a>(iter: &mut crate::CMarkDataIter<'a>) -> NextItem<'a> {
    use std::sync::Arc;
    use std::vec::Vec;

    let mut removed = Vec::new();
    loop {
        if let Some(node) = iter.next() {
            if let Some(event) = node.event() {
                return NextItem {
                    node: Some(Arc::clone(node)),
                    event: Some(event.clone()),
                    removed,
                };
            } else {
                removed.push(Arc::clone(node));
            }
        } else {
            return NextItem {
                node: None,
                event: None,
                removed,
            };
        }
    }
}

fn node_not_mached_diagnostic(
    codemap_files: &mut crate::CodemapFiles,
    readme_node: &Option<std::sync::Arc<crate::CMarkItem>>,
    docs_node: &Option<std::sync::Arc<crate::CMarkItem>>,
) -> codemap_diagnostic::Diagnostic {
    use crate::CodemapSpans;
    use codemap_diagnostic::{Diagnostic, Level};
    use std::format;

    let nodes = [readme_node, docs_node];
    let spans = nodes
        .iter()
        .filter_map(|node| node.as_ref())
        .flat_map(|node| node.spans());
    let span_labels = CodemapSpans::span_labels_from(codemap_files, spans);
    let readme_event = readme_node.as_ref().and_then(|node| node.event());
    let docs_event = docs_node.as_ref().and_then(|node| node.event());

    let message = match (readme_event, docs_event) {
        (Some(readme_event), Some(docs_event)) => format!(
            "readme node\n`{}`\n does not match docs node\n`{}`",
            CMarkDisplay(readme_event),
            CMarkDisplay(docs_event)
        ),
        (Some(readme_event), None) => format!(
            "readme node\n`{}`\n does not match any docs node",
            CMarkDisplay(readme_event)
        ),
        (None, Some(docs_event)) => format!(
            "docs node\n`{}`\n does not match any readme node",
            CMarkDisplay(docs_event)
        ),
        (None, None) => unreachable!(),
    };

    Diagnostic {
        level: Level::Error,
        message,
        code: None,
        spans: span_labels,
    }
}

fn removed_nodes_note(
    codemap_files: &mut crate::CodemapFiles,
    nodes: &[std::sync::Arc<crate::CMarkItem>],
    node_type: &str,
) -> Option<codemap_diagnostic::Diagnostic> {
    use crate::CodemapSpans;
    use codemap_diagnostic::{Diagnostic, Level};
    use std::format;

    if nodes.is_empty() {
        None
    } else {
        let spans = nodes.iter().flat_map(|node| node.spans());
        let span_labels = CodemapSpans::span_labels_from(codemap_files, spans);
        Some(Diagnostic {
            level: Level::Note,
            message: format!("some {} nodes were removed before these", node_type),
            code: None,
            spans: span_labels,
        })
    }
}

fn event_diff_notes(
    readme_event: &pulldown_cmark::Event<'_>,
    docs_event: &pulldown_cmark::Event<'_>,
) -> std::vec::Vec<codemap_diagnostic::Diagnostic> {
    use std::iter::repeat;
    use std::string::ToString;
    use std::{format, vec};

    use pulldown_cmark::{CodeBlockKind, Event, Tag};

    let readme_event_name = get_event_name(&readme_event);
    let docs_event_name = get_event_name(&docs_event);
    if readme_event_name != docs_event_name {
        return vec![
            text_note(std::format!(
                "readme node event name is \"{}\"",
                readme_event_name
            )),
            text_note(std::format!(
                "docs   node event name is \"{}\"",
                docs_event_name
            )),
        ];
    }

    let readme_tag = get_event_tag(&readme_event);
    let docs_tag = get_event_tag(&docs_event);
    if let (Some(readme_tag), Some(docs_tag)) = (readme_tag, docs_tag) {
        let readme_tag_name = get_tag_name(&readme_tag);
        let docs_tag_name = get_tag_name(&docs_tag);
        if readme_tag_name != docs_tag_name {
            let mut notes = vec![
                text_note(std::format!(
                    "readme node event tag name is \"{}\"",
                    readme_tag_name
                )),
                text_note(std::format!(
                    "docs   node event tag name is \"{}\"",
                    docs_tag_name
                )),
            ];
            if let Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) = docs_event {
                notes.push(text_note(
                    concat!(
                        "Possible issue: ",
                        "Rustdoc ignore indents in the consecutive ",
                        "doc-comments and doc-attributes. ",
                        "However, the four-space indents should be ",
                        "interpreted as Indented code blocks in CMark. ",
                        "Issue: https://github.com/rust-lang/rust/issues/70732",
                    )
                    .to_string(),
                ));
            }
            return notes;
        }
    }

    let readme_text = get_event_text(&readme_event);
    let docs_text = get_event_text(&docs_event);
    if let (Some(readme_text), Some(docs_text)) = (readme_text, docs_text) {
        if readme_text != docs_text {
            const OFFSET: usize = 32;
            const LEN: usize = 32;

            let readme_chars = readme_text.char_indices().map(Some).chain(repeat(None));
            let docs_chars = docs_text.char_indices().map(Some).chain(repeat(None));
            let mut chars = readme_chars.zip(docs_chars);
            let pos = chars
                .find_map(|pair| match pair {
                    (Some(lhs), Some(rhs)) => {
                        if lhs.1 != rhs.1 {
                            assert_eq!(lhs.0, rhs.0);
                            Some(lhs.0)
                        } else {
                            None
                        }
                    }
                    (Some(lhs), _) => Some(lhs.0),
                    (_, Some(rhs)) => Some(rhs.0),
                    (None, None) => unreachable!(),
                })
                .unwrap();
            let start = pos.saturating_sub(OFFSET);
            let end = pos + LEN;

            return vec![
                text_note(std::format!(
                    "readme node text part: \"{}\"",
                    formatted_subslice(readme_text, start, end)
                )),
                text_note(std::format!(
                    "docs   node text part: \"{}\"",
                    formatted_subslice(docs_text, start, end)
                )),
            ];
        }
    }

    vec![
        text_note(format!("readme node: {}", CMarkDisplay(readme_event))),
        text_note(format!("docs   node: {}", CMarkDisplay(docs_event))),
    ]
}

fn previous_events_notes(events: &[pulldown_cmark::Event<'_>]) -> codemap_diagnostic::Diagnostic {
    use std::format;
    use std::string::ToString;

    const MAX_EVENTS_SHOWN: usize = 16;

    if events.is_empty() {
        text_note("match failed on first events".to_string())
    } else if events.len() <= MAX_EVENTS_SHOWN {
        text_note(format!("previous events: [\n{}]", CMarkDisplay(events)))
    } else {
        text_note(format!(
            "previous events: [\n    ...,\n{}]",
            CMarkDisplay(&events[events.len() - MAX_EVENTS_SHOWN..])
        ))
    }
}

fn text_note(message: String) -> codemap_diagnostic::Diagnostic {
    use codemap_diagnostic::{Diagnostic, Level};
    use std::vec::Vec;

    Diagnostic {
        level: Level::Note,
        message,
        code: None,
        spans: Vec::new(),
    }
}

fn formatted_subslice(text: &str, start: usize, end: usize) -> String {
    use std::format;

    let skip_before = start > 3;
    let start = if skip_before { start } else { 0 };
    let skip_after = text.len().saturating_sub(end) > 3;
    let end = if skip_after { end } else { text.len() };

    format!(
        "{}{}{}",
        if skip_before { "..." } else { "" },
        &text[start..end],
        if skip_after { "..." } else { "" }
    )
}

fn get_event_tag<'a>(event: &'a pulldown_cmark::Event<'_>) -> Option<&'a pulldown_cmark::Tag<'a>> {
    use pulldown_cmark::Event;
    match event {
        Event::Start(tag) => Some(tag),
        Event::End(tag) => Some(tag),
        _ => None,
    }
}

fn get_event_text<'a>(event: &'a pulldown_cmark::Event<'_>) -> Option<&'a str> {
    use pulldown_cmark::Event;
    match event {
        Event::Text(text) => Some(&text),
        Event::Code(text) => Some(&text),
        Event::Html(text) => Some(&text),
        Event::FootnoteReference(text) => Some(&text),
        _ => None,
    }
}

fn get_event_name<'a>(event: &pulldown_cmark::Event<'_>) -> &'a str {
    use pulldown_cmark::Event;
    match event {
        Event::Start(..) => "Start",
        Event::End(..) => "End",
        Event::Text(..) => "Text",
        Event::Code(..) => "Code",
        Event::Html(..) => "Html",
        Event::FootnoteReference(..) => "FootnoteReference",
        Event::SoftBreak => "SoftBreak",
        Event::HardBreak => "HardBreak",
        Event::Rule => "Rule",
        Event::TaskListMarker(..) => "TaskListMarker",
    }
}

fn get_tag_name<'a>(tag: &'a pulldown_cmark::Tag<'_>) -> &'a str {
    use pulldown_cmark::Tag;
    match tag {
        Tag::Paragraph => "Paragraph",
        Tag::Heading(..) => "Heading",
        Tag::BlockQuote => "BlockQuote",
        Tag::CodeBlock(..) => "CodeBlock",
        Tag::List(..) => "List",
        Tag::Item => "Item",
        Tag::FootnoteDefinition(..) => "FootnoteDefinition",
        Tag::Table(..) => "Table",
        Tag::TableHead => "TableHead",
        Tag::TableRow => "TableRow",
        Tag::TableCell => "TableCell",
        Tag::Emphasis => "Emphasis",
        Tag::Strong => "Strong",
        Tag::Strikethrough => "Strikethrough",
        Tag::Link(..) => "Link",
        Tag::Image(..) => "Image",
    }
}

#[derive(Clone, Debug)]
pub struct CMarkDisplay<T>(T);

impl Display for CMarkDisplay<&[pulldown_cmark::Event<'_>]> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for item in self.0 {
            writeln!(fmt, "    {},", CMarkDisplay(item))?;
        }
        Ok(())
    }
}

impl Display for CMarkDisplay<Option<&pulldown_cmark::Event<'_>>> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Some(value) => write!(fmt, "Some({})", CMarkDisplay(value)),
            None => write!(fmt, "None"),
        }
    }
}

impl Display for CMarkDisplay<&pulldown_cmark::Event<'_>> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use pulldown_cmark::Event;

        let event_name = get_event_name(&self.0);
        write!(fmt, "{}", event_name)?;

        match &self.0 {
            Event::Start(tag) | Event::End(tag) => write!(fmt, "({})", CMarkDisplay(tag)),
            Event::Text(text)
            | Event::Code(text)
            | Event::Html(text)
            | Event::FootnoteReference(text) => write!(fmt, "(\"{}\")", &text),
            Event::SoftBreak => Ok(()),
            Event::HardBreak => Ok(()),
            Event::Rule => Ok(()),
            Event::TaskListMarker(ch) => write!(fmt, "({})", ch),
        }
    }
}

impl Display for CMarkDisplay<&pulldown_cmark::Tag<'_>> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use pulldown_cmark::Tag;

        let tag_name = get_tag_name(&self.0);
        write!(fmt, "{}", tag_name)?;

        match &self.0 {
            Tag::Paragraph => Ok(()),
            Tag::Heading(level) => write!(fmt, "({})", level),
            Tag::BlockQuote => Ok(()),
            Tag::CodeBlock(kind) => write!(fmt, "({})", CMarkDisplay(kind)),
            Tag::List(Some(first)) => write!(fmt, "(Some({}))", first),
            Tag::List(None) => write!(fmt, "(None)"),
            Tag::Item => Ok(()),
            Tag::FootnoteDefinition(label) => write!(fmt, "(\"{}\")", &label),
            Tag::Table(alignment) => write!(fmt, "({})", CMarkDisplay(&alignment[..])),
            Tag::TableHead => Ok(()),
            Tag::TableRow => Ok(()),
            Tag::TableCell => Ok(()),
            Tag::Emphasis => Ok(()),
            Tag::Strong => Ok(()),
            Tag::Strikethrough => Ok(()),
            Tag::Link(ty, url, title) => {
                write!(fmt, "({}, \"{}\", \"{}\")", CMarkDisplay(ty), url, title)
            }
            Tag::Image(ty, url, title) => {
                write!(fmt, "({}, \"{}\", \"{}\")", CMarkDisplay(ty), url, title)
            }
        }
    }
}

impl Display for CMarkDisplay<&pulldown_cmark::CodeBlockKind<'_>> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use pulldown_cmark::CodeBlockKind;

        match self.0 {
            CodeBlockKind::Indented => write!(fmt, "Indented"),
            CodeBlockKind::Fenced(tag) => write!(fmt, "Fenced({})", tag),
        }
    }
}

impl Display for CMarkDisplay<&[pulldown_cmark::Alignment]> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut iter = self.0.iter();
        let first = iter.next();
        if let Some(first) = first {
            write!(fmt, "{}", CMarkDisplay(first))?;
        }
        for item in iter {
            write!(fmt, ", {}", CMarkDisplay(item))?;
        }
        Ok(())
    }
}

impl Display for CMarkDisplay<&pulldown_cmark::Alignment> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use pulldown_cmark::Alignment;

        match self.0 {
            Alignment::None => write!(fmt, "None"),
            Alignment::Left => write!(fmt, "Left"),
            Alignment::Center => write!(fmt, "Center"),
            Alignment::Right => write!(fmt, "Right"),
        }
    }
}

impl Display for CMarkDisplay<&pulldown_cmark::LinkType> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use pulldown_cmark::LinkType;

        match self.0 {
            LinkType::Inline => write!(fmt, "Inline"),
            LinkType::Reference => write!(fmt, "Reference"),
            LinkType::ReferenceUnknown => write!(fmt, "ReferenceUnknown"),
            LinkType::Collapsed => write!(fmt, "Collapsed"),
            LinkType::CollapsedUnknown => write!(fmt, "CollapsedUnknown"),
            LinkType::Shortcut => write!(fmt, "Shortcut"),
            LinkType::ShortcutUnknown => write!(fmt, "ShortcutUnknown"),
            LinkType::Autolink => write!(fmt, "Autolink"),
            LinkType::Email => write!(fmt, "Email"),
        }
    }
}

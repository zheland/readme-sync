#![cfg(feature = "pulldown-cmark")]

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};

pub trait IntoStatic {
    type Output;

    fn into_static(self) -> Self::Output;
}
impl IntoStatic for Event<'_> {
    type Output = Event<'static>;

    fn into_static(self) -> Self::Output {
        match self {
            Self::Start(tag) => Event::Start(tag.into_static()),
            Self::End(tag) => Event::End(tag.into_static()),
            Self::Text(text) => Event::Text(text.into_static()),
            Self::Code(code) => Event::Code(code.into_static()),
            Self::Html(html) => Event::Html(html.into_static()),
            Self::FootnoteReference(label) => Event::FootnoteReference(label.into_static()),
            Self::SoftBreak => Event::SoftBreak,
            Self::HardBreak => Event::HardBreak,
            Self::Rule => Event::Rule,
            Self::TaskListMarker(ch) => Event::TaskListMarker(ch),
        }
    }
}

impl IntoStatic for Tag<'_> {
    type Output = Tag<'static>;

    fn into_static(self) -> Self::Output {
        match self {
            Self::Paragraph => Tag::Paragraph,
            Self::Heading(level) => Tag::Heading(level),
            Self::BlockQuote => Tag::BlockQuote,
            Self::CodeBlock(kind) => Tag::CodeBlock(kind.into_static()),
            Self::List(first) => Tag::List(first),
            Self::Item => Tag::Item,
            Self::FootnoteDefinition(label) => Tag::FootnoteDefinition(label.into_static()),
            Self::Table(alignment) => Tag::Table(alignment),
            Self::TableHead => Tag::TableHead,
            Self::TableRow => Tag::TableRow,
            Self::TableCell => Tag::TableCell,
            Self::Emphasis => Tag::Emphasis,
            Self::Strong => Tag::Strong,
            Self::Strikethrough => Tag::Strikethrough,
            Self::Link(ty, url, title) => Tag::Link(ty, url.into_static(), title.into_static()),
            Self::Image(ty, url, title) => Tag::Image(ty, url.into_static(), title.into_static()),
        }
    }
}

impl IntoStatic for CowStr<'_> {
    type Output = CowStr<'static>;

    fn into_static(self) -> Self::Output {
        use std::string::ToString;

        match self {
            Self::Boxed(boxed) => CowStr::Boxed(boxed),
            Self::Borrowed(borrowed) => CowStr::Boxed(borrowed.to_string().into_boxed_str()),
            Self::Inlined(inlined) => CowStr::Boxed(inlined.to_string().into_boxed_str()),
        }
    }
}

impl IntoStatic for CodeBlockKind<'_> {
    type Output = CodeBlockKind<'static>;

    fn into_static(self) -> Self::Output {
        match self {
            Self::Indented => CodeBlockKind::Indented,
            Self::Fenced(tag) => CodeBlockKind::Fenced(tag.into_static()),
        }
    }
}

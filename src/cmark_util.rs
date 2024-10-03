#![cfg(feature = "pulldown-cmark")]

use std::vec::Vec;

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

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
            Self::InlineMath(code) => Event::InlineMath(code.into_static()),
            Self::DisplayMath(code) => Event::DisplayMath(code.into_static()),
            Self::Html(html) => Event::Html(html.into_static()),
            Self::InlineHtml(html) => Event::InlineHtml(html.into_static()),
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
            Self::Heading {
                level,
                id,
                classes,
                attrs,
            } => Tag::Heading {
                level,
                id: id.into_static(),
                classes: classes.into_static(),
                attrs: attrs.into_static(),
            },
            Self::BlockQuote(kind) => Tag::BlockQuote(kind),
            Self::CodeBlock(kind) => Tag::CodeBlock(kind.into_static()),
            Self::HtmlBlock => Tag::HtmlBlock,
            Self::List(first) => Tag::List(first),
            Self::Item => Tag::Item,
            Self::FootnoteDefinition(label) => Tag::FootnoteDefinition(label.into_static()),
            Self::DefinitionList => Tag::DefinitionList,
            Self::DefinitionListTitle => Tag::DefinitionListTitle,
            Self::DefinitionListDefinition => Tag::DefinitionListDefinition,
            Self::Table(alignment) => Tag::Table(alignment),
            Self::TableHead => Tag::TableHead,
            Self::TableRow => Tag::TableRow,
            Self::TableCell => Tag::TableCell,
            Self::Emphasis => Tag::Emphasis,
            Self::Strong => Tag::Strong,
            Self::Strikethrough => Tag::Strikethrough,
            Self::Link {
                link_type,
                dest_url,
                title,
                id,
            } => Tag::Link {
                link_type,
                dest_url: dest_url.into_static(),
                title: title.into_static(),
                id: id.into_static(),
            },
            Self::Image {
                link_type,
                dest_url,
                title,
                id,
            } => Tag::Image {
                link_type,
                dest_url: dest_url.into_static(),
                title: title.into_static(),
                id: id.into_static(),
            },
            Self::MetadataBlock(kind) => Tag::MetadataBlock(kind),
        }
    }
}

impl IntoStatic for TagEnd {
    type Output = Self;

    fn into_static(self) -> Self::Output {
        self
    }
}

impl<T1, T2> IntoStatic for (T1, T2)
where
    T1: IntoStatic,
    T2: IntoStatic,
{
    type Output = (T1::Output, T2::Output);

    fn into_static(self) -> Self::Output {
        (self.0.into_static(), self.1.into_static())
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

impl<T> IntoStatic for Option<T>
where
    T: IntoStatic,
{
    type Output = Option<T::Output>;

    fn into_static(self) -> Self::Output {
        self.map(IntoStatic::into_static)
    }
}

impl<T> IntoStatic for Vec<T>
where
    T: IntoStatic,
{
    type Output = Vec<T::Output>;

    fn into_static(self) -> Self::Output {
        self.into_iter().map(IntoStatic::into_static).collect()
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

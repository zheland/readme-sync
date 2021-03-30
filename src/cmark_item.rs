#![cfg(feature = "pulldown-cmark")]

use core::ops::Range;
use std::borrow::Cow;
use std::boxed::Box;
use std::string::String;
use std::sync::Arc;
use std::vec::Vec;

use pulldown_cmark::Event;

use crate::TextSource;

/// An enum that store Markdown event, its origin, and modifications.
/// The event modification tree is stored to improve error messages.
#[derive(Clone, Debug, PartialEq)]
pub enum CMarkItem {
    /// A Markdown event parsed from the specified text source.
    Parsed {
        /// Parsed Markdown event.
        event: Event<'static>,
        /// The corresponding range of the event in the source text.
        range: Range<usize>,
        /// Text source.
        text_source: TextSource,
    },
    /// A Markdown event created by data transform functions.
    Created {
        /// Created Markdown event.
        event: Event<'static>,
        /// Data transformation note.
        note: Cow<'static, str>,
    },
    /// A Markdown event created from other Markdown events.
    Modified {
        /// A Markdown event created from other events.
        event: Event<'static>,
        /// Source `CMarkItem`s.
        nodes: Box<[Arc<CMarkItem>]>,
        /// Data transformation note.
        note: Cow<'static, str>,
    },
    /// A removed Markdown events.
    Removed {
        /// Removed `CMarkItem`s.
        nodes: Box<[Arc<CMarkItem>]>,
        /// Data transformation note.
        note: Cow<'static, str>,
    },
    /// A non-modified Markdown event that noted by data transform function.
    Noted {
        /// Noted `CMarkItem`.
        node: Arc<CMarkItem>,
        /// Data transformation note.
        note: Cow<'static, str>,
    },
}

/// Event's region of source text.
#[allow(single_use_lifetimes)] // false positive in PartialEq, issue: rust-lang/rust/#69952
#[derive(Clone, Debug, PartialEq)]
pub struct CMarkSpan<'a> {
    /// The corresponding range of the original event in the source text.
    pub range: &'a Range<usize>,
    /// Text source of original parsed event.
    pub text_source: &'a TextSource,
    /// Cumulative data transformation note.
    pub note: String,
}

impl<'a> CMarkItem {
    /// Creates `CMarkItem` with the specified Event and its note.
    pub fn new(event: Event<'static>, note: Cow<'static, str>) -> Arc<Self> {
        Arc::new(Self::Created { event, note })
    }

    /// Creates `CMarkItem` as an event parsed from the specified text source range.
    pub fn from(event: Event<'static>, range: Range<usize>, text_source: TextSource) -> Arc<Self> {
        Arc::new(Self::Parsed {
            event,
            range,
            text_source,
        })
    }

    /// Returns the event or None if it is removed.
    pub fn event(&self) -> Option<&Event<'static>> {
        match self {
            Self::Parsed { event, .. } => Some(event),
            Self::Created { event, .. } => Some(event),
            Self::Modified { event, .. } => Some(event),
            Self::Removed { .. } => None,
            Self::Noted { node, .. } => node.event(),
        }
    }

    /// Returns a `Vec` of event spans.
    pub fn spans(&self) -> Vec<CMarkSpan<'_>> {
        match self {
            Self::Parsed {
                event,
                range,
                text_source,
            } => std::vec![CMarkSpan {
                range: &range,
                text_source: &text_source,
                note: std::format!("{:?}", event),
            }],
            Self::Created { .. } => Vec::new(),
            Self::Modified { event, nodes, note } => nodes
                .iter()
                .flat_map(|node| node.spans())
                .map(|span| CMarkSpan {
                    note: span.note + " : " + note + " -> " + &std::format!("{:?}", event),
                    ..span
                })
                .collect(),
            Self::Removed { nodes, note } => nodes
                .iter()
                .flat_map(|node| node.spans())
                .map(|span| CMarkSpan {
                    note: span.note + " : " + note,
                    ..span
                })
                .collect(),
            Self::Noted { node, note } => node
                .spans()
                .into_iter()
                .map(|span| CMarkSpan {
                    note: span.note + " : " + note,
                    ..span
                })
                .collect(),
        }
    }
}

/// A helper trait to create an event as modified from other events.
pub trait CMarkItemAsModified {
    /// Сreate an event as modified from other events.
    fn into_modified(self, event: Event<'static>, note: Cow<'static, str>) -> Arc<CMarkItem>;
}

/// A helper trait to mark events as removed.
pub trait CMarkItemAsRemoved {
    /// Mark events as removed.
    fn into_removed(self, note: Cow<'static, str>) -> Arc<CMarkItem>;
}

/// A helper trait to mark an event with the specified note.
pub trait CMarkItemWithNote {
    /// Mark an event with the specified note.
    fn with_note(self, note: Cow<'static, str>) -> Arc<CMarkItem>;
}

impl CMarkItemAsModified for Arc<CMarkItem> {
    fn into_modified(self, event: Event<'static>, note: Cow<'static, str>) -> Arc<CMarkItem> {
        Arc::new(CMarkItem::Modified {
            nodes: Box::new([self]),
            event,
            note,
        })
    }
}

impl CMarkItemAsModified for Box<[Arc<CMarkItem>]> {
    fn into_modified(self, event: Event<'static>, note: Cow<'static, str>) -> Arc<CMarkItem> {
        assert!(!self.is_empty());
        Arc::new(CMarkItem::Modified {
            nodes: self,
            event,
            note,
        })
    }
}

impl CMarkItemAsModified for Vec<Arc<CMarkItem>> {
    fn into_modified(self, event: Event<'static>, note: Cow<'static, str>) -> Arc<CMarkItem> {
        assert!(!self.is_empty());
        Arc::new(CMarkItem::Modified {
            nodes: self.into_boxed_slice(),
            event,
            note,
        })
    }
}

impl CMarkItemAsRemoved for Arc<CMarkItem> {
    fn into_removed(self, note: Cow<'static, str>) -> Arc<CMarkItem> {
        Arc::new(CMarkItem::Removed {
            nodes: Box::new([self]),
            note,
        })
    }
}

impl CMarkItemAsRemoved for Box<[Arc<CMarkItem>]> {
    fn into_removed(self, note: Cow<'static, str>) -> Arc<CMarkItem> {
        assert!(!self.is_empty());
        Arc::new(CMarkItem::Removed { nodes: self, note })
    }
}

impl CMarkItemAsRemoved for Vec<Arc<CMarkItem>> {
    fn into_removed(self, note: Cow<'static, str>) -> Arc<CMarkItem> {
        assert!(!self.is_empty());
        Arc::new(CMarkItem::Removed {
            nodes: self.into_boxed_slice(),
            note,
        })
    }
}

impl CMarkItemWithNote for Arc<CMarkItem> {
    fn with_note(self, note: Cow<'static, str>) -> Arc<CMarkItem> {
        Arc::new(CMarkItem::Noted { node: self, note })
    }
}

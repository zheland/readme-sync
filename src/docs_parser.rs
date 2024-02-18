use std::borrow::Cow;

#[cfg(all(feature = "syn", feature = "thiserror"))]
use thiserror::Error;

#[cfg(all(feature = "syn", feature = "thiserror"))]
use crate::Config;

/// Parsed documentation text chunk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocsItem {
    /// Parsed or generated text.
    pub text: Cow<'static, str>,
    /// Source file span.
    pub span: Option<DocsSpan>,
}

/// Documentation text chunk span start and end.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocsSpan {
    /// Chunk start line and column.
    pub start: DocsLineColumn,
    /// Chunk end line and column.
    pub end: DocsLineColumn,
}

/// Documentation line and column.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocsLineColumn {
    /// The line number within the file (0-indexed).
    pub line: usize,
    /// The column within the line (0-indexed).
    pub column: usize,
}

#[cfg(feature = "syn")]
impl From<&syn::LitStr> for DocsItem {
    #[cfg(feature = "proc-macro2")]
    fn from(lit_str: &syn::LitStr) -> Self {
        let text = Cow::from(lit_str.value());
        let span = Some(DocsSpan::from(lit_str.span()));
        Self { text, span }
    }

    #[cfg(not(feature = "proc-macro2"))]
    fn from(lit_str: &syn::LitStr) -> Self {
        let text = Cow::from(lit_str.value());
        let span = None;
        Self { text, span }
    }
}

impl From<&'static str> for DocsItem {
    fn from(text: &'static str) -> Self {
        Self {
            text: Cow::from(text),
            span: None,
        }
    }
}

#[cfg(feature = "proc-macro2")]
impl From<proc_macro2::Span> for DocsSpan {
    fn from(span: proc_macro2::Span) -> Self {
        let start = span.start();
        let end = span.end();
        Self {
            start: DocsLineColumn {
                line: start.line - 1,
                column: start.column,
            },
            end: DocsLineColumn {
                line: end.line - 1,
                column: end.column,
            },
        }
    }
}

/// Builds documentation from the specified attribute.
#[cfg(all(feature = "syn", feature = "thiserror"))]
pub fn build_attr_docs(
    attr: &syn::Attribute,
    config: &Config<'_>,
) -> Result<impl Iterator<Item = DocsItem>, BuildAttrDocsError> {
    Ok(build_meta_docs(&attr.meta, config)?)
}

/// Builds documentation from the specified compile-time structured attribute.
#[cfg(all(feature = "syn", feature = "thiserror"))]
pub fn build_meta_docs(
    meta: &syn::Meta,
    config: &Config<'_>,
) -> Result<impl Iterator<Item = DocsItem>, BuildMetaDocsError> {
    use std::vec::Vec;

    if meta.path().is_ident("doc") {
        match meta {
            syn::Meta::NameValue(syn::MetaNameValue { value, .. }) => match value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    attrs,
                }) if attrs.is_empty() => {
                    Ok(std::vec![DocsItem::from(lit_str), DocsItem::from("\n")].into_iter())
                }
                _ => Err(BuildMetaDocsError::NonStringDocInput(meta.clone())),
            },
            _ => Ok(Vec::new().into_iter()),
        }
    } else if meta.path().is_ident("cfg_attr") {
        match meta {
            syn::Meta::List(meta_list) => {
                let mut it = meta_list
                    .parse_args::<PunctuatedMetaArgs>()
                    .map_err(|_| BuildMetaDocsError::CfgNonMetaAttribute(meta_list.clone()))?
                    .0
                    .into_iter();
                let predicate = it
                    .next()
                    .ok_or_else(|| BuildMetaDocsError::CfgAttrWithoutPredicate(meta.clone()))?;
                let mut it = it.peekable();
                let _ = it
                    .peek()
                    .ok_or_else(|| BuildMetaDocsError::CfgAttrWithoutAttribute(meta.clone()))?;

                let predicate_result = eval_cfg_predicate(&predicate, config)?;
                if predicate_result {
                    let doc: Result<Vec<DocsItem>, BuildMetaDocsError> = it
                        .map(|nested_meta| build_meta_docs(&nested_meta, config))
                        .try_fold(Vec::new(), |mut acc, doc| {
                            acc.extend(doc?);
                            Ok(acc)
                        });
                    let doc = doc?;
                    Ok(doc.into_iter())
                } else {
                    Ok(Vec::new().into_iter())
                }
            }
            _ => Err(BuildMetaDocsError::NonListCfgAttrInput(meta.clone())),
        }
    } else {
        Ok(Vec::new().into_iter())
    }
}

/// Evaluates configuration predicate.
#[cfg(all(feature = "syn", feature = "thiserror"))]
pub fn eval_cfg_predicate(
    meta: &syn::Meta,
    config: &Config<'_>,
) -> Result<bool, EvalCfgPredicateError> {
    use std::string::ToString;

    let ident = meta
        .path()
        .get_ident()
        .ok_or_else(|| EvalCfgPredicateError::NonIdentPath(meta.clone()))?;
    match meta {
        syn::Meta::Path(_) => Ok(config.idents.contains(ident.to_string().as_str())),
        syn::Meta::List(meta_list) => {
            let it = meta_list
                .parse_args::<PunctuatedMetaArgs>()
                .map_err(|_| EvalCfgPredicateError::CfgNonMetaAttribute(meta_list.clone()))?
                .0
                .into_iter();

            if ident == "all" {
                for nested_meta in it {
                    match eval_cfg_predicate(&nested_meta, config) {
                        Ok(true) => {}
                        value => {
                            return value;
                        }
                    }
                }
                Ok(true)
            } else if ident == "any" {
                for nested_meta in it {
                    match eval_cfg_predicate(&nested_meta, config) {
                        Ok(false) => {}
                        value => {
                            return value;
                        }
                    }
                }
                Ok(false)
            } else if ident == "not" {
                let mut iter = it;
                if let (Some(first), None) = (iter.next(), iter.next()) {
                    Ok(!eval_cfg_predicate(&first, config)?)
                } else {
                    Err(EvalCfgPredicateError::NonSingleNotInput(meta.clone()))
                }
            } else {
                Err(EvalCfgPredicateError::InvalidPredicateFn(meta.clone()))
            }
        }
        syn::Meta::NameValue(syn::MetaNameValue { value, .. }) => match value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                attrs,
            }) if attrs.is_empty() => Ok(config.name_values.contains(&(
                Cow::from(ident.to_string().as_str()),
                Cow::from(lit_str.value()),
            ))),
            _ => Err(EvalCfgPredicateError::NonStringOptionValue(meta.clone())),
        },
    }
}

struct PunctuatedMetaArgs(syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>);

impl syn::parse::Parse for PunctuatedMetaArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self(syn::punctuated::Punctuated::<
            syn::Meta,
            syn::Token![,],
        >::parse_terminated(input)?))
    }
}

/// An error which can occur when building documentation from attribute.
#[cfg(all(feature = "syn", feature = "thiserror"))]
#[derive(Clone, Debug, Error)]
pub enum BuildAttrDocsError {
    /// Attribute parser error.
    #[error("Attribute parser error: {0}")]
    SynError(#[from] syn::Error),
    /// Meta parser error.
    #[error("Meta parser error: {0}")]
    MetaError(#[from] BuildMetaDocsError),
}

/// An error which can occur when building documentation from meta-attribute.
#[cfg(all(feature = "syn", feature = "thiserror"))]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum BuildMetaDocsError {
    /// Non-string doc attribute input.
    #[error("Non-string doc attribute input: `{0:?}`.")]
    NonStringDocInput(syn::Meta),
    /// Non-list `cfg_attr` attribute input.
    #[error("Non-list `cfg_attr` attribute input: `{0:?}`.")]
    NonListCfgAttrInput(syn::Meta),
    /// `cfg_attr` without predicate argument.
    #[error("`cfg_attr` should contain predicate argument: `{0:?}`.")]
    CfgAttrWithoutPredicate(syn::Meta),
    /// `cfg_attr` has no attributes.
    #[error("`cfg_attr` should contain at least one attribute: `{0:?}`.")]
    CfgAttrWithoutAttribute(syn::Meta),
    /// `cfg_attr` non-meta attribute.
    #[error("`cfg_attr` attribute should be a meta attribute: `{0:?}`.")]
    CfgNonMetaAttribute(syn::MetaList),
    /// Predicate evaluation error.
    #[error(transparent)]
    PredicateError(#[from] EvalCfgPredicateError),
}

/// An error which can occur when evaluating configuration predicate.
#[cfg(all(feature = "syn", feature = "thiserror"))]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum EvalCfgPredicateError {
    /// Non-indentifier predicatge path.
    #[error("Predicate path should be an identifier: `{0:?}`.")]
    NonIdentPath(syn::Meta),
    /// Non-single `not()` input.
    #[error("Predicate `not()` accepts only a single inner predicate: `{0:?}`.")]
    NonSingleNotInput(syn::Meta),
    /// Unknown predicate function.
    #[error("Unknown predicate function: `{0:?}`.")]
    InvalidPredicateFn(syn::Meta),
    /// Non-string option value.
    #[error("Predicatge option values can only be a string or raw string literal: `{0:?}`.")]
    NonStringOptionValue(syn::Meta),
    /// Non-meta attribute.
    #[error("Predicate should be a meta attribute: `{0:?}`.")]
    CfgNonMetaAttribute(syn::MetaList),
}

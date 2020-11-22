use std::borrow::Cow;
use std::collections::HashSet;

use crate::Package;

/// A set of enabled named and key-value configuration options.
#[allow(single_use_lifetimes)] // false positive in PartialEq, issue: rust-lang/rust/#69952
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Config<'a> {
    /// Enabled named configuration options.
    pub idents: HashSet<Cow<'a, str>>,
    /// Enabled key-value configuration options.
    pub name_values: HashSet<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> Config<'a> {
    /// Creates an empty `Config`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a `Config` with features defined in `[package.metadata.docs.rs]` table in crates' Cargo.toml.
    pub fn from_package_docs_rs_features(package: &'a Package) -> Self {
        Self::new().with_features(package.manifest().docs_rs_features())
    }

    /// Extend `Config` with the feature names from an iterator.
    pub fn with_features<I, T>(mut self, features: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Cow<'a, str>>,
    {
        self.name_values.extend(
            features
                .into_iter()
                .map(|feature| (Cow::from("feature"), feature.into())),
        );
        self
    }

    /// Add target_arch, target_os and target_env `Config` options from the specified target.
    ///
    /// This method require non-default feature `platforms`.
    #[cfg(feature = "platforms")]
    pub fn with_target_arch_os_env(mut self, target: &str) -> Self {
        if let Some(platform) = platforms::find(target) {
            let _ = self.name_values.insert((
                Cow::from("target_arch"),
                Cow::from(platform.target_arch.as_str()),
            ));
            let _ = self.name_values.insert((
                Cow::from("target_os"),
                Cow::from(platform.target_os.as_str()),
            ));
            let _ = self.name_values.insert((
                Cow::from("target_env"),
                Cow::from(
                    platform
                        .target_env
                        .map_or("", |target_env| target_env.as_str()),
                ),
            ));
        }
        self
    }
}

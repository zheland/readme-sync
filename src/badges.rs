#![cfg(feature = "glob")]

use std::vec::Vec;

use glob::Pattern;

/// Returns a Vec of populatar badge url patterns.
pub fn badge_url_patterns() -> Vec<Pattern> {
    std::vec![
        // GitHub badges
        Pattern::new("http://github.com/*/badge.svg").unwrap(),
        Pattern::new("https://github.com/*/badge.svg").unwrap(),
        Pattern::new("http://github.com/*/badge.svg?*").unwrap(),
        Pattern::new("https://github.com/*/badge.svg?*").unwrap(),
        // CMarkDocs badges
        Pattern::new("http://docs.rs/repos/*/badge.svg").unwrap(),
        Pattern::new("https://docs.rs/repos/*/badge.svg").unwrap(),
        // Badge providers
        Pattern::new("http://img.shields.io/*").unwrap(),
        Pattern::new("https://img.shields.io/*").unwrap(),
        Pattern::new("http://meritbadge.herokuapp.com/*").unwrap(),
        Pattern::new("https://meritbadge.herokuapp.com/*").unwrap(),
        Pattern::new("http://badges.gitter.im/*").unwrap(),
        Pattern::new("https://badges.gitter.im/*").unwrap(),
        // Continuous integration
        Pattern::new("http://travis-ci.org/*").unwrap(),
        Pattern::new("https://travis-ci.org/*").unwrap(),
        Pattern::new("http://travis-ci.com/*").unwrap(),
        Pattern::new("https://travis-ci.com/*").unwrap(),
        Pattern::new("http://api.travis-ci.org/*").unwrap(),
        Pattern::new("https://api.travis-ci.org/*").unwrap(),
        Pattern::new("http://api.travis-ci.com/*").unwrap(),
        Pattern::new("https://api.travis-ci.com/*").unwrap(),
        Pattern::new("http://ci.appveyor.com/api/projects/status/*").unwrap(),
        Pattern::new("https://ci.appveyor.com/api/projects/status/*").unwrap(),
        Pattern::new("http://circleci.com/gh/*").unwrap(),
        Pattern::new("https://circleci.com/gh/*").unwrap(),
        // Code coverage
        Pattern::new("http://codecov.io/gh/*").unwrap(),
        Pattern::new("https://codecov.io/gh/*").unwrap(),
        Pattern::new("http://coveralls.io/repos/*").unwrap(),
        Pattern::new("https://coveralls.io/repos/*").unwrap(),
    ]
}

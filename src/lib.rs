//! library for easily working with version numbers in the SEM verison system (a.b.c)
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate serde;

mod versionpart;
mod version;

// passing through Version, since this will be the main interface in the library
pub use version::Version;

#[cfg(test)]
extern crate serde_test;
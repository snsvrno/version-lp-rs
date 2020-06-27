//! library for easily working with version numbers in the SEM verison system (a.b.c)

mod versionpart;
mod version;

// passing through Version, since this will be the main interface in the library
pub use crate::version::Version;

#[cfg(test)]
extern crate serde_test;
//! library for easily working with version numbers in the SEM verison system (a.b.c)
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate serde;

mod versionpart;
pub mod version;

#[cfg(test)]
extern crate serde_test;
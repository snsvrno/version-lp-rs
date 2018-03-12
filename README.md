# Version-LP-RS
A rust library for dealing with versions designed to be used with lovepack tools.

## Overview
Contains a custom version ***Struct*** that is based on the [Semantic Versioning System](https://semver.org/). Only supports the `a.b.c` format.

Also has support for wildcards when compairing `Versions` together.

```rust

let wild_version = Version::from_str("2.*.*");

Version::from_string("2.3.4").unwrap().is_compatible_with(&wild_version) // will return true

```
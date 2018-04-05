# Version-LP-RS
A rust library for dealing with versions designed to be used with lovepack tools.

## Overview
Contains a custom version ***Struct*** that is based on the [Semantic Versioning System](https://semver.org/). Only supports the `a.b.c` format.

Also has support for wildcards when compairing `Versions` together.

```rust

let wild_version = Version::from_str("2.*.*");

Version::from_string("2.3.4").unwrap().is_compatible_with(&wild_version) // will return true

```

And standard comparions can be used.

```rust

let ver_a = Version::from_str("2.1.4");
let ver_b = Version::from_str("2.2.3");
let ver_c = Version::from_str("2.1.4");


ver_a < ver_b // true
ver_a == ver_c // true

```

As a final note, you cannot compare against patterns, patterns can only be checked using the `is_compatible_with` function.

## Pattern Matching
Currently the only wildcard supported is `*`.
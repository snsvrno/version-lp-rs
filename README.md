# version-lp-rs
A rust library for dealing with versions designed to be used with lovepack tools.

## Overview
Contains a custom version ***Struct*** that is based on the [Semantic Versioning System](https://semver.org/). Only supports the `a.b.c` format, but with any number of points, i.e. `a.b`, `a.b.c.d` are also valid versions. Also has support for wildcards when compairing `Versions`.

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

You can also get the latest available version from a list.

```rust
let versions : Vec<Version> = vec![
    Version::from_str("1.0.0").unwrap(),
    Version::from_str("1.0.1").unwrap(),
    Version::from_str("1.1.0").unwrap(),
    Version::from_str("1.0.2").unwrap()
];

let requirement = Version::from_str("1").unwrap();

let version = requirement.latest_compatible_version(&versions); // would be Version (1.1.0)

```

## Notes for Success
- You cannot compare against patterns, patterns can only be checked using the `is_compatible_with` function.
- Wildcards will be assumed when compairing different length version numbers. `1.2` will be compatible with `1.2.3`

## Pattern Matching
Currently the only wildcard supported is `*`. But `^` can be achieved by using short versions: `1.2` would match with `1.2.1` to `1.2.100` and would return the latest version in a list using `::latest_compatible_version`.
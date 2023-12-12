# Rustifact &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![docs]][docs.rs]

[Build Status]: https://github.com/mbaulch/rustifact/actions/workflows/rust.yml/badge.svg?branch=master
[actions]: https://github.com/mbaulch/rustifact/actions?query=branch%3Amaster
[Latest Version]: https://img.shields.io/crates/v/rustifact.svg
[crates.io]: https://crates.io/crates/rustifact
[docs]: https://docs.rs/rustifact/badge.svg
[docs.rs]: https://docs.rs/rustifact


_A seamless bridge between a build script and the main crate._

# Motivation
When it comes to generating computationally intensive artifacts at compile time, we have
many tools at our disposal: build scripts (build.rs), declarative macros (macro_rules!),
procedural macros, and increasingly, const functions. Each of these methods, however,
brings its own set of challenges.

*Rustifact* has been designed as an abstraction layer that simplifies the creation of build scripts
that produce data for inclusion into the final binary.

# Types supported
Rustifact allows `static` and `const` declarations of data types composed of numeric types
(floats, ints, usize), booleans, strings, arrays, structs, and enums. Also supported are
(unordered and ordered) sets and maps with perfect-hash lookup.

(*) Sets and maps are provided with help from the excellent
[phf_codegen](https://crates.io/crates/phf_codegen) library, though these features are gated via
the `set` and `map` features.

(*) Jagged array support is available via the [rustifact_extra](https://crates.io/crates/rustifact_extra) crate.

# Usage steps
1. Generate the required data in your build script.

2. `#[derive(ToTokenStream)]` for any custom types(*) (not in the Rust standard library) exported from your
build script.

3. Export your data with any combination of the `write_X` macros.

4. In the main part of your crate (within `src/`) import your data with `use_symbols`.

(*) These types should be implemented in a separate crate, so they're usable from the build script
_and_ the main crate.

NOTE: We refer to exclusively to *data* in the above, but Rustifact is also capable of generating *types*
in some situations where doing so by hand would be burdensome.

# A simple example
build.rs
```rust
use rustifact::ToTokenStream;

fn main() {
    // Write a constant of type Option<(i32, i32)>
    let a = Some((1, 2));
    rustifact::write_const!(CONST_A, Option<(i32, i32)>, &a);
    // Write a static variable of type &'static str. Strings map to static string slices.
    let b = format!("Hello {}", "from Rustifact");
    rustifact::write_static!(STATIC_B, &'static str, &b);
    // Write a getter function returning Vec<Vec<i32>>
    let c = vec![vec![1], vec![2, 3], vec![4, 5, 6]];
    rustifact::write_fn!(get_c, Vec<Vec<i32>>, &c);
    // Write a static array of i32 with dimension two.
    let arr1: [[i32; 3]; 3] = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    rustifact::write_static_array!(ARRAY_1, i32 : 2, &arr1);
    // Write a const array of f32 with dimension one.
    let arr2: [f32; 3] = [1.1, 1.2, 1.3];
    rustifact::write_const_array!(ARRAY_2, f32 : 1, &arr2);
    // or equivalently: rustifact::write_const_array!(ARRAY_2, f32, &arr2);
}
```

src/main.rs
```rust
rustifact::use_symbols!(CONST_A, STATIC_B, get_c, ARRAY_1, ARRAY_2);

fn main() {
    assert!(CONST_A == Some((1, 2)));
    assert!(STATIC_B == "Hello from Rustifact");
    assert!(get_c() == vec![vec![1], vec![2, 3], vec![4, 5, 6]]);
    assert!(ARRAY_1 == [[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
    assert!(ARRAY_2 == [1.1, 1.2, 1.3]);
}
```

Cargo.toml
```toml
[package]
# ...

[build-dependencies]
rustifact = "0.10"

[dependencies]
rustifact = "0.10"
```

# More examples

* [array4d](examples/array4d) Generates and exports a 4 dimensional array using `write_static_array!`

* [city_data](examples/city_data) Generates and exports a one dimensional array using `write_static_array!`

* [coords](examples/coords) Demonstrates the use of custom types with `#[derive(ToTokenStream)]`.

* [html_tags](examples/html_tags) Exports a large collection of individual constants using `write_statics!`.

* [out_type](examples/out_type) Demonstrates struct export with `ToTokenStream`'s `OutType` attribute.

* [map](examples/map) Demonstrates construction of a map with lookup via a perfect hash function.

For more examples, inspect the `write_X` macros in the [crate documentation](https://docs.rs/rustifact).

# Development status
Please note that _Rustifact_ is in an early development stage.  Overall, it is unlikely to
cause unpleasant surprises, though there may be edge cases that haven't yet been discovered.
Some breaking changes may occur in the future, though we aim to preserve backward compatibility
where possible.

# License
Rustifact is free software, and is released under the terms of the [Mozilla Public License](https://www.mozilla.org/en-US/MPL/) version 2.0. See [LICENSE](LICENSE).

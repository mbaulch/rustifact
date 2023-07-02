# Rustifact
_A seamless bridge between a build script and the main crate._

# Motivation
When it comes to generating computationally intensive artifacts at compile time, we have
many tools at our disposal: build scripts (build.rs), declarative macros (macro_rules!),
procedural macros, and increasingly, const functions. Each of these methods, however,
brings its own set of challenges.

*Rustifact* has been designed as a streamlined abstraction layer that simplifies the creation of build scripts
that produce data for inclusion into the final binary.

# Usage steps
1. Generate the required data in your build script.

2. (Optional*#) Implement the `ToTokenStream` trait for each of your build script's 'exported' types.

3. Export your data with any combination of the `write_X` macros.

4. In the main part of your crate (within `src/`) import your data with `use_symbols`.

(*) `ToTokenStream` is implemented for primitive types (`u8`, `i32`, `char`, `bool`, ...),
`slice`s, `array`, `Vec`, and `Option`. This step is only necessary if you're exporting your
own types. We expect to automate this step soon by providing suitable `[#derive(...)]` macros.

(#) These types should be implemented in a separate crate, so they're usable from the build script
_and_ the main crate.

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
}```

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
rustifact = "0.5"

[dependencies]
rustifact = "0.5"
```

# More examples

* [array4d](examples/array4d) Generates and exports a 4 dimensional array using `write_static_array!`

* [city_data](examples/city_data) Generates and exports a one dimensional array using `write_static_array!`

* [html_tags](examples/html_tags) Exports a large collection of individual constants using `write_statics!`.

# Development status
Please note that _Rustifact_ is in an early development stage.  Overall, it is unlikely to
cause unpleasant surprises, though there may be edge cases that haven't yet been discovered.
Some breaking changes may occur in the future, though we aim to preserve backward compatibility
where possible.

# License
Rustifact is free software, and is released under the terms of the [Mozilla Public License](https://www.mozilla.org/en-US/MPL/) version 2.0. See [LICENSE](LICENSE).

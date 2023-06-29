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

fn generate_city_data() -> Vec<(String, u32)> {
    let mut city_data: Vec<(String, u32)> = Vec::new();
    for i in 1..=100 {
        let city_name = format!("City{}", i);
        let population = i * 1000; // Dummy population data
        city_data.push((city_name, population));
    }
    city_data
}

fn main() {
    let city_data = generate_city_data();
    //
    // Let's make city_data accessible from the main crate. We'll write it to
    // a static array CITY_DATA where the type of each element is (&'static str, u32).
    // Note that Strings are converted to static string slices by default.
    //
    rustifact::write_static_array!(CITY_DATA, (&'static str, u32), &city_data);
    //
    // We could have specified the dimension like so:
    //rustifact::write_static_array!(CITY_DATA, (&'static str, u32) : 1, &city_data);
    //
    // When the dimension is unspecified (as above) the default is dimension 1.
}
```

src/main.rs
```rust
rustifact::use_symbols!(CITY_DATA);
// The above line is equivalent to the declaration:
// static CITY_DATA: [(&'static str, u32); 100] = [/*.. data from build.rs */];

fn main() {
   for (name, population) in CITY_DATA.iter() {
       println!("{} has population {}", name, population)
   }
}
```

# More examples

* [array4d](examples/array4d) Generates and exports a 4 dimensional array using `write_static_array!`

* [html_tags](examples/html_tags) Exports a large collection of individual constants using `write_statics!`.


# Development status
Please note that _Rustifact_ is in an early development stage.  Overall, it is unlikely to
cause unpleasant surprises, though there may be edge cases that haven't yet been discovered.
Some breaking changes may occur in the future, though we aim to preserve backward compatibility
where possible.

# License
Rustifact is free software, and is released under the terms of the [Mozilla Public License](https://www.mozilla.org/en-US/MPL/) version 2.0. See [LICENSE](LICENSE).

# Rustifact
_A seamless bridge between a build script and the main crate._

# Usage steps

1. (Optional*) Implement the `ToTokenStream` trait for each of your build script's 'exported' types.
* This is predefined for primitive types (`u8`, `i32`, `char`, `bool`, ...), `slice`s,
`array`s, and `Vec`s. This step is only necessary if you're exporting your own types.
* These types should be implemented in a separate crate, so they're usable from the build script
_and_ the main crate.

2. Generate the required data in your build script.

3. Export your data with any combination of `write_array_fn`, `write_const_array`,
`write_static_array`, and `write_vector_fn`.

4. In the main part of your crate (within `src/`) import your data with `use_symbols`.

(*) We expect to automate this step soon by providing suitable `[#derive(...)]` macros.

# Motivation
When it comes to generating computationally intensive artifacts at compile time, we have
many tools at our disposal: build scripts (build.rs), declarative macros (macro_rules!),
procedural macros, and increasingly, const functions. Each of these methods, however,
brings its own set of challenges.

Issues with namespaces and types can arise from using build scripts and macros. Const functions,
while useful, may cause performance issues during compilation. Build scripts can make file management
complex. The number of library functions available to macros and const functions is limited.
Declarative macros suffer from a lack of expressiveness, and both macros and const functions can
encounter problems with environmental isolation.

Rustifact has been designed as a streamlined abstraction layer that simplifies the use of build scripts.
By mitigating these complexities, Rustifact offers a more efficient approach for handling
compile-time computations in Rust.

# A simple example
build.rs
```rust
use rustifact::ToTokenStream;

fn main() {
   let mut city_data: Vec<(String, u32)> = Vec::new();
   for i in 1..=1000 {
       let city_name = format!("City{}", i);
       let population = i * 1000; // Dummy population data
       city_data.push((city_name, population));
   }
   // Let's make city_data accessible from the main crate. We'll write it to
   // a static array CITY_DATA where the type of each element is (&'static str, u32).
   // Note that Strings are converted to static string slices by default.
   //
   rustifact::write_static_array!(CITY_DATA, (&'static str, u32), &city_data);
   //
   // Alternatively, this could be written:
   // rustifact::write_static_array!(CITY_DATA, (&'static str, u32) : 1, &city_data);
   //
   // When the dimension is unspecified, the default is dimension 1.
   //
   // Passing city_data as a slice allows it to be treated as an
   // array. Note that this would not have been possible if its elements were heap allocated.
   // In that case, write_array_fn or write_vector_fn would need to be used.
}
```

src/main.rs
```rust
rustifact::use_symbols!(CITY_DATA);
// The above line is equivalent to the declaration:
// static CITY_DATA: [(&'static str, u32); 1000] = [/*.. data from build.rs */];

fn main() {
   for (name, population) in CITY_DATA.iter() {
       println!("{} has population {}", name, population)
   }
}
```

# Development status
Please note that _Rustifact_ is in an early development stage. While it is utilised in at least one
commercial project, it lacks extensive testing and polishing. Overall, it is unlikely to cause unpleasant
surprises, though there may be edge cases that haven't yet been discovered.
As the API surface is minimal, it's unlikely that API changes would cause major headaches,
though be warned that some breaking changes may occur in the future.

## License

Rustifact is free software, and is released under the terms of the [Mozilla Public License](https://www.mozilla.org/en-US/MPL/) version 2.0. See [LICENSE](LICENSE).
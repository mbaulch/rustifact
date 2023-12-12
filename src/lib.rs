// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! # Rustifact
//!
//! _A seamless bridge between a build script and the main crate._
//!
//! # Motivation
//! When it comes to generating computationally intensive artifacts at compile time, we have
//! many tools at our disposal: build scripts (build.rs), declarative macros (macro_rules!),
//! procedural macros, and increasingly, const functions. Each of these methods, however,
//! brings its own set of challenges.
//!
//! *Rustifact* has been designed as an abstraction layer that simplifies the creation of build scripts
//! that produce data for inclusion into the final binary.
//!
//! # Types supported
//! Rustifact allows `static` and `const` declarations of data types composed of numeric types
//! (floats, ints, usize), booleans, strings, arrays, structs, and enums. Also supported are
//! (unordered and ordered) sets and maps with perfect-hash lookup.
//!
//! (*) Sets and maps are provided with help from the excellent
//! [phf_codegen](https://crates.io/crates/phf_codegen) library, though these features are gated via
//! the `set` and `map` features.
//!
//! (*) Jagged array support is available via the [rustifact_extra](https://crates.io/crates/rustifact_extra) crate.
//!
//! # Usage steps
//!
//! 1. Generate the required data in your build script.
//!
//! 2. `#[derive(ToTokenStream)]` for any custom types(*) (not in the Rust standard library) exported from your
//! build script.
//!
//! 3. Export your data with any combination of the `write_X` macros.
//!
//! 4. In the main part of your crate (within `src/`) import your data with [`use_symbols`].
//!
//! (*) These types should be implemented in a separate crate, so they're usable from the build script
//! _and_ the main crate.
//!
//! NOTE: We refer to exclusively to *data* in the above, but Rustifact is also capable of generating *types*
//! in some situations where doing so by hand would be burdensome.
//!
//! # A simple example
//! build.rs
//! ```no_run
//! use rustifact::ToTokenStream;
//!
//! fn main() {
//!     // Write a constant of type Option<(i32, i32)>
//!     let a = Some((1, 2));
//!     rustifact::write_const!(CONST_A, Option<(i32, i32)>, &a);
//!     // Write a static variable of type &'static str. Strings map to static string slices.
//!     let b = format!("Hello {}", "from Rustifact");
//!     rustifact::write_static!(STATIC_B, &'static str, &b);
//!     // Write a getter function returning Vec<Vec<i32>>
//!     let c = vec![vec![1], vec![2, 3], vec![4, 5, 6]];
//!     rustifact::write_fn!(get_c, Vec<Vec<i32>>, &c);
//!     // Write a static array of i32 with dimension two.
//!     let arr1: [[i32; 3]; 3] = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
//!     rustifact::write_static_array!(ARRAY_1, i32 : 2, &arr1);
//!     // Write a const array of f32 with dimension one.
//!     let arr2: [f32; 3] = [1.1, 1.2, 1.3];
//!     rustifact::write_const_array!(ARRAY_2, f32 : 1, &arr2);
//!     // or equivalently: rustifact::write_const_array!(ARRAY_2, f32, &arr2);
//! }
//!```
//!
//!src/main.rs
//! ```no_run
//! rustifact::use_symbols!(CONST_A, STATIC_B, get_c, ARRAY_1, ARRAY_2);
//!
//! fn main() {
//!     assert!(CONST_A == Some((1, 2)));
//!     assert!(STATIC_B == "Hello from Rustifact");
//!     assert!(get_c() == vec![vec![1], vec![2, 3], vec![4, 5, 6]]);
//!     assert!(ARRAY_1 == [[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
//!     assert!(ARRAY_2 == [1.1, 1.2, 1.3]);
//! }
//! ```
//!
//! Cargo.toml
//! ```no_run
//! [package]
//! ## ...
//!
//! [build-dependencies]
//! rustifact = "0.9"
//!
//! [dependencies]
//! rustifact = "0.9"
//! ```
//!
//! # Development status
//! Please note that _Rustifact_ is in an early development stage.  Overall, it is unlikely to
//! cause unpleasant surprises, though there may be edge cases that haven't yet been discovered.
//! Some breaking changes may occur in the future, though we aim to preserve backward compatibility
//! where possible.

mod tokens;

mod phf;

#[cfg(feature = "map")]
pub use crate::phf::{Map, MapBuilder, OrderedMap, OrderedMapBuilder};

#[cfg(feature = "set")]
pub use crate::phf::{OrderedSet, OrderedSetBuilder, Set, SetBuilder};

pub use rustifact_derive::ToTokenStream;
pub use tokens::ToTokenStream;

/// An implementation detail, exposing parts of external crates used by `rustifact`.
///
/// API stability is not guaranteed here.
pub mod internal {
    #[cfg(any(feature = "map", feature = "set"))]
    pub use phf;
    /// A re-export of `unparse` from the `prettyplease` crate.
    pub use prettyplease::unparse;
    /// A re-export of `TokenStream` from the `proc_macro2` crate.
    pub use proc_macro2::TokenStream;
    /// A re-export of `format_ident!` from the `quote` crate.
    pub use quote::format_ident;
    /// A re-export of `quote!` from the `quote` crate.
    pub use quote::quote;
    /// A re-export of `parse_file` from the `syn` crate.
    pub use syn::parse_file;
    /// A re-export of `parse_str` from the `syn` crate.
    pub use syn::parse_str;
    /// A re-export of `Type` from the `syn` crate.
    pub use syn::Type;
    #[doc(hidden)]
    pub fn allow_export_error(id: &str) -> String {
        format!(
            concat!(
                "Couldn't find symbol {} to setup export.",
                "Ensure you call write_static (or another write_... function)",
                "for {} before allow_export",
            ),
            id, id
        )
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __get_tokens_array_impl {
    (0, $data:expr) => {{
        let mut tokens = rustifact::internal::TokenStream::new();
        for i in $data {
            let i_toks = i.to_tok_stream();
            let element = rustifact::internal::quote! { #i_toks, };
            tokens.extend(element);
        }
        rustifact::internal::quote! { [#tokens] }
    }};
    ($dim:tt, $data:expr) => {
        rustifact::__get_tokens_array_multi!($data, |i| rustifact::__get_tokens_array!($dim, i))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __get_tokens_vector_fn_impl {
    (0, $data:expr) => {{
        let mut tokens = rustifact::internal::TokenStream::new();
        for i in $data {
            let i_toks = i.to_tok_stream();
            let element = rustifact::internal::quote! { #i_toks, };
            tokens.extend(element);
        }
        rustifact::internal::quote! { vec![#tokens] }
    }};
    ($dim:tt, $data:expr) => {
        rustifact::__get_tokens_vector_fn_multi!($data, |i| rustifact::__get_tokens_vector_fn!(
            $dim, i
        ))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __path_from_id {
    ($id_name:ident, private) => {{
        format!(
            "{}/rustifact_{}_{}.rs",
            std::env::var("OUT_DIR").unwrap(),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            stringify!($id_name),
        )
    }};
    ($id_name:ident, public) => {{
        format!(
            "{}/rustifact__pub__{}_{}.rs",
            std::env::var("OUT_DIR").unwrap(),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            stringify!($id_name),
        )
    }};
    ($id_name:expr, private) => {{
        format!(
            "{}/rustifact_{}_{}.rs",
            std::env::var("OUT_DIR").unwrap(),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            $id_name,
        )
    }};
    ($id_name:expr, public) => {{
        format!(
            "{}/rustifact__pub__{}_{}.rs",
            std::env::var("OUT_DIR").unwrap(),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            $id_name,
        )
    }};
}

/// Import the given symbols (generated by the build script) into scope.
///
/// # Limitations
/// Any types referenced by the imported symbols must be manually brought into scope.
/// This may not be necessary in future versions of *Rustifact*.
/// See the relevant [tracking issue](https://github.com/mbaulch/rustifact/issues/4).
#[macro_export]
macro_rules! use_symbols {
    ($($id_name:ident),*) => {
        $(
            include!(concat!(
                env!("OUT_DIR"),
                "/rustifact_",
                env!("CARGO_PKG_NAME"),
                "_",
                stringify!($id_name),
                ".rs"
            ));
        )*
    };
}

/// Export the given symbols (generated by the build script).
///
/// `allow_export!` must be called in the build script for each of the symbols.
///
/// # Example
/// See [`allow_export!`].
///
/// # Limitations
/// Any types referenced by the imported symbols must be manually brought into scope.
/// This may not be necessary in future versions of *Rustifact*.
/// See the relevant [tracking issue](https://github.com/mbaulch/rustifact/issues/4).

#[macro_export]
macro_rules! export_symbols {
    ($($id_name:ident),*) => {
        $(
            include!(concat!(
                env!("OUT_DIR"),
                "/rustifact__pub__",
                env!("CARGO_PKG_NAME"),
                "_",
                stringify!($id_name),
                ".rs"
            ));
        )*
    };
}

#[doc = "Setup the symbol for export from the main crate.

Makes the symbol exportable from the main crate via `export_symbols`. Before calling `allow_export!`
the symbol must be output from the build script with one of the usual `write_`... macros.

## Parameters
* `$id`: The symbol to setup for export.

## Example
build.rs
 ```no_run
use rustifact::ToTokenStream;

fn main() {
    rustifact::write_static!(FOO, &'static str, \"I'm exportable\");
    rustifact::allow_export!(FOO);
}
```

src/lib.rs
```no_run
rustifact::export_symbols!(FOO);

// The above line is equivalent to the declaration:
// pub static FOO: &'static str = \"I'm exportable\";
```"]
#[macro_export]
macro_rules! allow_export {
    ($id_name:ident) => {{
        let private_path_str = rustifact::__path_from_id!($id_name, private);
        let asset_str;
        if let Ok(s) = std::fs::read_to_string(private_path_str) {
            asset_str = s;
        } else {
            panic!(
                "{}",
                rustifact::internal::allow_export_error(stringify!($id_name))
            );
        }
        rustifact::__write_tokens_with_internal!($id_name, public, format!("pub {}", asset_str));
    }};
}

/// Import the given struct initialisation expression (generated by the build script) into scope.
///
/// # Limitations
/// Any types referenced by the imported symbols must be manually brought into scope.
/// This may not be necessary in future versions of *Rustifact*.
/// See the relevant [tracking issue](https://github.com/mbaulch/rustifact/issues/4).
#[macro_export]
macro_rules! init_symbols {
    ($id_struct:ident, $id_vals:ident) => {
        include!(concat!(
            env!("OUT_DIR"),
            "/rustifact_",
            env!("CARGO_PKG_NAME"),
            "_",
            stringify!($id_struct),
            "_",
            stringify!($id_vals),
            ".rs"
        ));
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __array_type_impl {
    (0, $t:ty, $data:expr) => {{
        let len = $data.len();
        rustifact::internal::quote! { [$t; #len] }
    }};
    ($dim:tt, $t:ty, $data:expr) => {{
        let data_next = $data[0];
        let inner = rustifact::__array_type!($dim, $t, data_next);
        let len = $data.len();
        rustifact::internal::quote! { [#inner; #len] }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __vector_type_impl {
    (0, $t:ty, $_:expr) => {
        rustifact::internal::quote! { Vec<$t> }
    };
    ($dim:tt, $t:ty, $data:expr) => {{
        let inner = rustifact::__vector_type!($dim, $t, $data);
        rustifact::internal::quote! { Vec<#inner> }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_tokens_with_internal {
    ($id_name:ident, $visibility:ident, $tokens:expr) => {
        let path_str = rustifact::__path_from_id!($id_name, $visibility);
        let path = std::path::Path::new(&path_str);
        match rustifact::internal::parse_file(&$tokens.to_string()) {
            Ok(syntax_tree) => {
                let formatted = rustifact::internal::unparse(&syntax_tree);
                std::fs::write(&path, formatted).unwrap();
            }
            Err(e) => {
                std::fs::write(&path, &$tokens.to_string()).unwrap();
                panic!(
                    "Failed to pretty-print {} due to parse error: '{}'
This _probably_ indicates in issue with a ToTokenStream implementation. Unformatted output has
been written to {}",
                    stringify!(id_name),
                    e,
                    path.display()
                );
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_tokens_with_internal_raw {
    ($id_name:expr, $tokens:expr) => {
        let path_str = rustifact::__path_from_id!($id_name, private);
        let path = std::path::Path::new(&path_str);
        std::fs::write(&path, &$tokens.to_string()).unwrap();
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_with_internal {
    ($const_static:ident, $id_name:ident, $arr_type:expr, $tokens_data:expr) => {{
        let arr_type = $arr_type;
        let tokens_data = $tokens_data;
        let tokens = rustifact::internal::quote! {
            $const_static $id_name: #arr_type = #tokens_data;
        };
        rustifact::__write_tokens_with_internal!($id_name, private, tokens);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_fn_with_internal {
    ($_:ident, $id_name:ident, $vec_type:expr, $tokens_data:expr) => {{
        let vec_type = $vec_type;
        let tokens_data = $tokens_data;
        let tokens = rustifact::internal::quote! {
            fn $id_name() -> #vec_type { #tokens_data }
        };
        rustifact::__write_tokens_with_internal!($id_name, private, tokens);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __assert_dim_impl {
    (0, $arr:expr) => {};
    ($dim:tt, $arr:expr) => {
        if $arr.len() == 0 {
            panic!("Actual array (or vec) is too shallow");
        }
        let arr_first = $arr[0];
        rustifact::__assert_dim!($dim, arr_first);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_with_impl {
    (
        $dim:tt, $const_static:ident, $id_name:ident, $t:ty, $data:expr,
        $get_tokens:ident, $get_type:ident, $write_internal:ident
    ) => {{
        rustifact::__assert_dim!($dim, $data);
        let tokens_data = rustifact::$get_tokens!($dim, $data);
        let arr_type = rustifact::$get_type!($dim, $t, $data);
        rustifact::$write_internal!($const_static, $id_name, arr_type, tokens_data);
    }};
}

use_symbols!(
    __array_type,
    __assert_dim,
    __get_tokens_array,
    __get_tokens_vector_fn,
    __vector_type,
    __write_with,
    write_array_fn,
    write_const_array,
    write_static_array,
    write_vector_fn
);

#[doc = "Write a static variable.

Makes the variable available for import into the main crate via `use_symbols`.

## Parameters
* `$id`: the name of the static variable. This must be used when importing with `use_symbols`.
* `$t`: the type of the static variable.
* `$data`: the data to assign to the static variable. Must be representable on the stack.

## Example
build.rs
 ```no_run
use rustifact::ToTokenStream;
use std::process::Command;
use std::str;

fn get_cmd_output(cmd: &mut Command) -> Option<String> {
    cmd.output()
        .ok()
        .map(|out| str::from_utf8(&out.stdout).unwrap().trim().to_string())
}

fn main() {
    let uname_output: Option<String> = get_cmd_output(Command::new(\"uname\").arg(\"-a\"));
    let dmesg_output: Option<String> = get_cmd_output(&mut Command::new(\"dmesg\"));
    rustifact::write_static!(UNAME_OUTPUT, Option<&'static str>, uname_output);
    rustifact::write_static!(DMESG_OUTPUT, Option<&'static str>, dmesg_output);
}
```

src/main.rs
```no_run
rustifact::use_symbols!(UNAME_OUTPUT, DMESG_OUTPUT);
// The above line is equivalent to the declarations:
// static UNAME_OUTPUT: Option<&'static str> = Some(/* output of 'uname -a' at build time, if it succeeded */);
// static DMESG_OUTPUT: Option<&'static str> = Some(/* output of 'dmesg' at build time, if it succeeded */);

fn main() {
    for (cmd, cmd_output) in &[(\"uname -a\", UNAME_OUTPUT), (\"dmesg\", DMESG_OUTPUT)] {
        print!(\"At build time, the command '{}' \", cmd);
        if let Some(out) = cmd_output {
            println!(\"produced output: '{}'\", out);
        } else {
            println!(\"failed.\");
        }
    }
}
```"]
#[macro_export]
macro_rules! write_static {
    ($id:ident, $t:ty, $data:expr) => {
        let data = $data;
        rustifact::__write_with_internal!(
            static,
            $id,
            rustifact::internal::quote! { $t },
            data.to_tok_stream()
        );
    };
}

#[doc = "Write a constant variable.

Makes the constant available for import into the main crate via `use_symbols`.

## Parameters
* `$id`: the name of the constant. This must be used when importing with `use_symbols`.
* `$t`: the type of the constant.
* `$data`: the data to assign to the constant. Must be representable on the stack.

## Example
build.rs
 ```no_run
use rustifact::ToTokenStream;

fn main() {
    let meaning_of_life = Some(42);
    rustifact::write_const!(MEANING_OF_LIFE, Option<i32>, meaning_of_life);
}
```

src/main.rs
```no_run
rustifact::use_symbols!(MEANING_OF_LIFE);
// The above line is equivalent to the declaration:
// const MEANING_OF_LIFE: Option<i32> = Some(42);

fn main() {
    if let Some(mean) = MEANING_OF_LIFE {
        println!(\"The meaning of life is {}\", mean);
    } else {
        println!(\"Life has no meaning.\");
    }
}
```"]
#[macro_export]
macro_rules! write_const {
    ($id:ident, $t:ty, $data:expr) => {
        let data = $data;
        rustifact::__write_with_internal!(
            const,
            $id,
            rustifact::internal::quote! { $t },
            data.to_tok_stream()
        );
    };
}

#[doc = "Write a getter function for a heap-allocated variable.

Makes the getter function available for import into the main crate via `use_symbols`.

## Parameters
* `$id`: the name of the getter function. This must be used when importing with `use_symbols`.
* `$t`: the return type of the getter function.
* `$data`: the data to return from the geter function.

## Example
build.rs
 ```no_run
use rustifact::ToTokenStream;

fn main() {
    let vecs = vec![vec![1, 2], vec![1, 2, 3], vec![1, 2, 3, 4]];
    rustifact::write_fn!(get_vecs, Vec<Vec<u32>>, vecs);
}
```

src/main.rs
```no_run
rustifact::use_symbols!(get_vecs);
// The above line is equivalent to the declaration:
// fn get_vecs() -> Vec<Vec<u32>> {
//     vec![vec![1, 2], vec![1, 2, 3], vec![1, 2, 3, 4]]
// }

fn main() {
    println!(\"{:?}\", get_vecs());
}
```"]
#[macro_export]
macro_rules! write_fn {
    ($id:ident, $t:ty, $data:expr) => {
        let data = $data;
        rustifact::__write_fn_with_internal!(
            dummy,
            $id,
            rustifact::internal::quote! { $t },
            data.to_tok_stream()
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_internal {
    ($static_const:ident, $id_group:ident, $t:ty, $public:literal, $ids_data:expr) => {{
        let mut toks = rustifact::internal::TokenStream::new();
        let ids_data = $ids_data;
        for (id_str, data) in ids_data.iter() {
            let data_toks = data.to_tok_stream();
            let id = rustifact::internal::format_ident!("{}", id_str);
            let element = if $public {
                rustifact::internal::quote! { pub $static_const #id: $t = #data_toks; }
            } else {
                rustifact::internal::quote! { $static_const #id: $t = #data_toks; }
            };
            toks.extend(element);
        }
        rustifact::__write_tokens_with_internal!($id_group, private, toks);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_internal_struct {
    ($id_struct:ident, $public:literal, $vis_ids_types:expr) => {{
        let mut toks = rustifact::internal::TokenStream::new();
        let vis_ids_types = $vis_ids_types;
        for (public, id_str, type_str) in vis_ids_types.iter() {
            if let Ok(t) = rustifact::internal::parse_str::<rustifact::internal::Type>(type_str) {
                let id = rustifact::internal::format_ident!("{}", id_str);
                let element = if *public {
                    rustifact::internal::quote! { pub #id: #t, }
                } else {
                    rustifact::internal::quote! { #id: #t, }
                };
                toks.extend(element);
            } else {
                panic!("Couldn't parse the type '{}'", type_str);
            }
        }
        let toks_struct = if $public {
            rustifact::internal::quote! {
                pub struct $id_struct { #toks }
            }
        } else {
            rustifact::internal::quote! {
               struct $id_struct { #toks }
            }
        };
        rustifact::__write_tokens_with_internal!($id_struct, private, toks_struct);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_internal_struct_uniform {
    ($id_struct:ident, $t:ty, $public:literal, $vis_ids:expr) => {{
        let mut toks = rustifact::internal::TokenStream::new();
        let vis_ids = $vis_ids;
        for (public, id_str) in vis_ids.iter() {
            let id = rustifact::internal::format_ident!("{}", id_str);
            let element = if *public {
                rustifact::internal::quote! { pub #id: $t, }
            } else {
                rustifact::internal::quote! { #id: $t, }
            };
            toks.extend(element);
        }
        let toks_struct = if $public {
            rustifact::internal::quote! {
                pub struct $id_struct { #toks }
            }
        } else {
            rustifact::internal::quote! {
               struct $id_struct { #toks }
            }
        };
        rustifact::__write_tokens_with_internal!($id_struct, private, toks_struct);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_internal_struct_uniform_init {
    ($id_struct:ident, $id_exps:ident, $t:ty, $ids_exps:expr) => {{
        let mut toks = rustifact::internal::TokenStream::new();
        let ids_exps = $ids_exps;
        for (id_str, exp) in ids_exps.iter() {
            let id = rustifact::internal::format_ident!("{}", id_str);
            let exp_toks = exp.to_tok_stream();
            toks.extend(rustifact::internal::quote! { #id: #exp_toks, });
        }
        let id_exps = rustifact::internal::format_ident!(
            "{}_{}",
            stringify!($id_struct),
            stringify!($id_exps)
        );
        let toks_init = rustifact::internal::quote! {
            $id_struct { #toks }
        };
        rustifact::__write_tokens_with_internal_raw!(id_exps, toks_init);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_internal_fns {
    ($id_group:ident, $t:ty, $public:literal, $ids_data:expr) => {{
        let mut toks = rustifact::internal::TokenStream::new();
        let ids_data = $ids_data;
        for (id_str, data) in ids_data.iter() {
            let data_toks = data.to_tok_stream();
            let id = rustifact::internal::format_ident!("{}", id_str);
            let element = if $public {
                rustifact::internal::quote! { pub fn #id() -> $t {#data_toks} }
            } else {
                rustifact::internal::quote! { fn #id() -> $t {#data_toks} }
            };
            toks.extend(element);
        }
        rustifact::__write_tokens_with_internal!($id_group, private, toks);
    }};
}

#[doc = "Write a collection of static variables with a common type.

Makes the static variables available for import into the main crate via `use_symbols`.

## Parameters
* `public` or `private`: whether to make the variables publicly visible after import with `use_symbols`.
* `$id_group`: the group alias by which these variables are referred when importing with `use_symbols`.
* `$t`: the (common) type of the static variables.
* `$ids_data`: The list of type `&[(I, $t)]` where $t is as above, and I is a type implementing Display,
though most commonly String or &'static str. This is a list of identifiers for the variables paired with
their values.

## Notes
* Intended for stack-allocated data. For heap-allocated data, use `write_fns` instead.
* Rather than passing identifiers directly, they are passed as string (in fact Display-implementing) types.
It is anticipated that this will be more convenient in the typical use cases of the write_Xs family of macros."]
#[macro_export]
macro_rules! write_statics {
    (public, $id_group:ident, $t:ty, $ids_data:expr) => {
        rustifact::__write_internal!(static, $id_group, $t, true, $ids_data);
    };
    (private, $id_group:ident, $t:ty, $ids_data:expr) => {
        rustifact::__write_internal!(static, $id_group, $t, false, $ids_data);
    };
}

#[doc = "Write a collection of constants with a common type.

Makes the constants available for import into the main crate via `use_symbols`.

## Parameters
* `public` or `private`: whether to make the constants publicly visible after import with `use_symbols`.
* `$id_group`: the group alias by which these variables are referred when importing with `use_symbols`.
* `$t`: the (common) type of the static variables.
* `$ids_data`: The list of type `&[(I, $t)]` where $t is as above, and I is a type implementing Display,
though most commonly String or &'static str. This is a list of identifiers for the constants paired with
their values.

## Notes
* Intended for stack-allocated data. For heap-allocated data, use `write_fns` instead.
* Rather than passing identifiers directly, they are passed as string (in fact Display-implementing) types.
It is anticipated that this will be more convenient in the typical use cases of the write_Xs family of macros."]
#[macro_export]
macro_rules! write_consts {
    (public, $id_group:ident, $t:ty, $ids_data:expr) => {
        rustifact::__write_internal!(const, $id_group, $t, true, $ids_data);
    };
    (private, $id_group:ident, $t:ty, $ids_data:expr) => {
        rustifact::__write_internal!(const, $id_group, $t, false, $ids_data);
    };
}

#[doc = "Write a collection of getter functions returning a common type.

Makes the getter functions available for import into the main crate via `use_symbols`.

## Parameters
* `public` or `private`: whether to make the functions publicly visible after import with `use_symbols`.
* `$id_group`: the group alias by which these functions are referred when importing with `use_symbols`.
* `$t`: the (common) return type of the getter functions.
* `$ids_data`: The list of type `&[(I, $t)]` where $t is as above, and I is a type implementing Display,
though most commonly String or &'static str. This is a list of identifiers for the functions paired with
their values.

## Notes
* Intended for heap-allocated data. For stack-allocated data, consider `write_consts` or `write_static` instead.
* Rather than passing identifiers directly, they are passed as string (in fact Display-implementing) types.
It is anticipated that this will be more convenient in the typical use cases of the write_Xs family of macros."]
#[macro_export]
macro_rules! write_fns {
    (public, $id_group:ident, $t:ty, $ids_data:expr) => {
        rustifact::__write_internal_fns!($id_group, $t, true, $ids_data);
    };
    (private, $id_group:ident, $t:ty, $ids_data:expr) => {
        rustifact::__write_internal_fns!($id_group, $t, false, $ids_data);
    };
}

#[doc = "Write a struct type definition.

Makes the `struct` type available for import into the main crate via `use_symbols`.

## Parameters
* `public` or `private`: whether to make the struct publicly visible after import with `use_symbols`.
* `$id`: the name of the struct type, and the identifier by which it is referred when importing with
`use_symbols`.
* `$vis_ids_types`: The list of type `&[(bool, I, T)]` where the first component indicates visibility
(true = public, false = private) of a field, I is the field's identifier having type String or &str, and T
is the field's type: also having type String or &str.

## Notes
Before using `write_struct!` carefully consider all other approaches. Defining a struct in the usual way
should be preferred when this is possible.

## Some use cases
* Generation of wrapper APIs
* Dependency injection, possibly in combination with `write_statics!`. Suppose that crate A depends on crate B.
The build script of crate A generates certain constants C1, ..., Cn. `write_struct!` is used to create a
type T with fields that can be instantiated with the constants C1, ..., Cn.
Functions (say) in crate B can be called with a parameter of type T, allowing crate B access to the constants
C1, ..., Cn even though crate B is a dependency of crate A. If access to C1, ..., Cn is desired in crate A or
other crates depending on crate A, make a suitable call to `write_statics!` (or `write_consts!`) in crate A's
build script, followed by `use_symbols!`.

## Example
build.rs
 ```no_run
fn main() {
    let foo_fields = vec![
        (true, \"field_a\", \"Vec<u32>\"),
        (true, \"field_b\", \"String\"),
        (false, \"field_c\", \"(bool, Option<f32>)\"),
        (false, \"field_d\", \"i64\"),
    ];
    rustifact::write_struct!(private, Foo, &foo_fields);
}
```

src/main.rs
```no_run
rustifact::use_symbols!(Foo);
// The above line is equivalent to the declaration:
// struct Foo {
//     pub field_a: Vec<u32>,
//     pub field_b: String,
//     field_c: (bool, Option<f32>),
//     field_d: i64,
// }
```"]
#[macro_export]
macro_rules! write_struct {
    (public, $id_struct:ident, $vis_ids_types:expr) => {
        rustifact::__write_internal_struct!($id_struct, true, $vis_ids_types);
    };
    (private, $id_struct:ident, $vis_ids_types:expr) => {
        rustifact::__write_internal_struct!($id_struct, false, $vis_ids_types);
    };
}

#[doc = "Write a struct type definition with a single field type.

Makes the `struct` type available for import into the main crate via `use_symbols`.

## Parameters
* `public` or `private`: whether to make the struct publicly visible after import with `use_symbols`.
* `$id_struct`: the name of the struct type, and the identifier by which it is referred when importing with
`use_symbols`.
* `$t`: the type of *all* fields of this struct
* `$vis_ids`: The list of type `&[(bool, I)]` where the first component indicates visibility
(true = public, false = private) of a field, and I is the field's identifier having type String or &str.

## Notes
Before using `write_struct_uniform!` carefully consider all other approaches.
Defining a struct in the usual way should be preferred when this is possible.

## Some use cases
* Generation of wrapper APIs
* Dependency injection, possibly in combination with `write_statics!`. Suppose that crate A depends on crate B.
The build script of crate A generates certain constants C1, ..., Cn. `write_struct_uniform!` is used to create
a type T with fields that can be instantiated with the constants C1, ..., Cn.
Functions (say) in crate B can be called with a parameter of type T, allowing crate B access to the constants
C1, ..., Cn even though crate B is a dependency of crate A. If access to C1, ..., Cn is desired in crate A or
other crates depending on crate A, make a suitable call to `write_statics!` (or `write_consts!`) in crate A's
build script, followed by `use_symbols!`.

## Example
build.rs
 ```no_run
fn main() {
    let foo_fields = vec![
        (true, \"field_a\"),
        (true, \"field_b\"),
        (false, \"field_c\"),
    ];
    rustifact::write_struct_uniform!(public, Foo, (u32, &'static str), &foo_fields);
}
```

src/main.rs
```no_run
rustifact::use_symbols!(Foo);
// The above line is equivalent to the declaration:
// pub struct Foo {
//     pub field_a: (u32, &'static str>),
//     pub field_b: (u32, &'static str),
//     field_c: (u32, &'static str),
// }
```"]
#[macro_export]
macro_rules! write_struct_uniform {
    (public, $id_struct:ident, $t:ty, $vis_ids_types:expr) => {
        rustifact::__write_internal_struct_uniform!($id_struct, $t, true, $vis_ids_types);
    };
    (private, $id_struct:ident, $t:ty, $vis_ids_types:expr) => {
        rustifact::__write_internal_struct_uniform!($id_struct, $t, false, $vis_ids_types);
    };
}

#[doc = "Write a struct initialisation expression.

Makes the `struct` initialisation expression available for import into the main crate via `use_symbols`.

## Parameters
* `$id_struct`: the name of the struct type, and the identifier by which it is referred when importing with
`use_symbols`.
* `$id_vals`: An identifier alias for this assignment of field values. Can only ever be referenced as
the second parameter to `init_symbols!`.
* `$t`: the type of *all* fields of this struct
* `$ids_vals`: The list of type `&[(I, V)]` where I is the field's identifier having type String or &str,
and V is the value (of type $t) to assign to the field.

## Notes
Before using `write_struct_uniform!` carefully consider all other approaches.
Defining a struct in the usual way should be preferred when this is possible.

## Some use cases
* Generation of wrapper APIs
* Dependency injection, possibly in combination with `write_statics!`. Suppose that crate A depends on crate B.
The build script of crate A generates certain constants C1, ..., Cn. `write_struct_uniform!` is used to create
a type T with fields that can be instantiated with the constants C1, ..., Cn.
Functions (say) in crate B can be called with a parameter of type T, allowing crate B access to the constants
C1, ..., Cn even though crate B is a dependency of crate A. If access to C1, ..., Cn is desired in crate A or
other crates depending on crate A, make a suitable call to `write_statics!` (or `write_consts!`) in crate A's
build script, followed by `use_symbols!`.

## Example
build.rs
 ```no_run
use rustifact::ToTokenStream;

fn main() {
    let foo_fields = vec![(true, \"field_a\"), (true, \"field_b\"), (false, \"field_c\")];
    let foo_vals = vec![
        (\"field_a\", (0u32, \"abc\")),
        (\"field_b\", (1u32, \"def\")),
        (\"field_c\", (2u32, \"ghi\")),
    ];
    rustifact::write_struct_uniform!(public, Foo, (u32, &'static str), &foo_fields);
    rustifact::write_struct_uniform_init!(Foo, Init, (u32, &'static str), &foo_vals);
}
```

src/main.rs
```no_run
rustifact::use_symbols!(Foo);
// Bring the Foo type into scope

static FOO_INIT: Foo = rustifact::init_symbols!(Foo, Init);
// The above line is equivalent to the declaration:
//
// static FOO_INIT: Foo = Foo {
//     field_a: (0, \"abc\"),
//     field_b: (1, \"def\"),
//     field_c: (2, \"ghi\"),
// }

fn main() {
    assert!(FOO_INIT.field_a == (0, \"abc\"));
    assert!(FOO_INIT.field_b == (1, \"def\"));
    assert!(FOO_INIT.field_c == (2, \"ghi\"));
}
```"]
#[macro_export]
macro_rules! write_struct_uniform_init {
    ($id_struct:ident, $id_vals:ident, $t:ty, $ids_vals:expr) => {
        rustifact::__write_internal_struct_uniform_init!($id_struct, $id_vals, $t, $ids_vals);
    };
}

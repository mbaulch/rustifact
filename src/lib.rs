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
//! *Rustifact* has been designed as a streamlined abstraction layer that simplifies the creation of build scripts
//! that produce data for inclusion into the final binary.
//!
//! # Usage steps
//!
//! 1. Generate the required data in your build script.
//!
//! 2. (Optional*#) Implement the [`ToTokenStream`] trait for each of your build script's 'exported' types.
//!
//! 3. Export your data with any combination of the `write_X` macros.
//!
//! 4. In the main part of your crate (within `src/`) import your data with [`use_symbols`].
//!
//! (*) [`ToTokenStream`] is implemented for primitive types ([`u8`], [`i32`], [`char`], [`bool`], ...),
//! [`slice`]s, [`array`], [`Vec`], and [`Option`]. This step is only necessary if you're exporting your
//! own types. We expect to automate this step soon by providing suitable `[#derive(...)]` macros.
//!
//! (#) These types should be implemented in a separate crate, so they're usable from the build script
//! _and_ the main crate.
//!
//! # A simple example
//! build.rs
//! ```no_run
//!use rustifact::ToTokenStream;
//!
//!fn generate_city_data() -> Vec<(String, u32)> {
//!    let mut city_data: Vec<(String, u32)> = Vec::new();
//!    for i in 1..=100 {
//!        let city_name = format!("City{}", i);
//!        let population = i * 1000; // Dummy population data
//!        city_data.push((city_name, population));
//!    }
//!    city_data
//!}
//!
//!fn main() {
//!    let city_data = generate_city_data();
//!    //
//!    // Let's make city_data accessible from the main crate. We'll write it to
//!    // a static array CITY_DATA where the type of each element is (&'static str, u32).
//!    // Note that Strings are converted to static string slices by default.
//!    //
//!    rustifact::write_static_array!(CITY_DATA, (&'static str, u32), &city_data);
//!    //
//!    // We could have specified the dimension like so:
//!    //rustifact::write_static_array!(CITY_DATA, (&'static str, u32) : 1, &city_data);
//!    //
//!    // When the dimension is unspecified (as above) the default is dimension 1.
//!}
//!```
//!
//!src/main.rs
//! ```no_run
//! rustifact::use_symbols!(CITY_DATA);
//! // The above line is equivalent to the declaration:
//! // static CITY_DATA: [(&'static str, u32); 1000] = [/*.. data from build.rs */];
//!
//! fn main() {
//!    for (name, population) in CITY_DATA.iter() {
//!        println!("{} has population {}", name, population)
//!    }
//!}
//! ```
//!
//! # Development status
//! Please note that _Rustifact_ is in an early development stage.  Overall, it is unlikely to
//! cause unpleasant surprises, though there may be edge cases that haven't yet been discovered.
//! Some breaking changes may occur in the future, though we aim to preserve backward compatibility
//! where possible.

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, TokenStreamExt};
/// Provides a flexible interface for converting Rust's data types into their token stream representation.
/// This trait is akin to `quote::ToTokens`, with a similar design, but it serves a distinct purpose.
///
/// Rust's `quote` crate is a fantastic tool for metaprogramming, offering the ability to produce Rust code
/// within Rust itself. It exposes the `ToTokens` trait, which types can implement to define how they can
/// be turned into a token stream, i.e., a sequence of Rust's syntactic tokens.
///
/// However, `ToTokens` in `quote` does not provide out-of-the-box implementations for certain common
/// data types like tuples. Furthermore, due to Rust's orphan rule, we can't implement `ToTokens` for
/// these types outside of the `quote` crate. This limitation is unacceptable given the expected use cases of _Rustifact_.
///
/// # Design
///
/// The trait exposes three primary methods:
///
/// - `to_toks(&self, toks: &mut TokenStream)`: Defines how the type is converted into a token stream. This is the primary method implementers should focus on.
///
/// - `to_tok_stream(&self) -> TokenStream`: A helper method which leverages `to_toks` to generate a new token stream.
///
/// - `to_tokens(&self, toks: &mut TokenStream)`: This method mirrors `to_toks` and is included for compatibility with `quote::ToTokens`.
///
/// This crate also provides implementations for a range of primitive types, booleans, references, arrays, and vectors.
///
pub trait ToTokenStream {
    fn to_toks(&self, toks: &mut TokenStream);

    fn to_tok_stream(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        self.to_toks(&mut tokens);
        tokens
    }

    fn to_tokens(&self, toks: &mut TokenStream) {
        self.to_toks(toks);
    }
}

macro_rules! primitive {
    ($($t:ty => $name:ident)*) => {
        $(
            impl ToTokenStream for $t {
                fn to_toks(&self, tokens: &mut TokenStream) {
                    tokens.append(Literal::$name(*self));
                }
            }
        )*
    };
}

primitive! {
    i8 => i8_suffixed
    i16 => i16_suffixed
    i32 => i32_suffixed
    i64 => i64_suffixed
    i128 => i128_suffixed
    isize => isize_suffixed

    u8 => u8_suffixed
    u16 => u16_suffixed
    u32 => u32_suffixed
    u64 => u64_suffixed
    u128 => u128_suffixed
    usize => usize_suffixed

    f32 => f32_suffixed
    f64 => f64_suffixed

    char => character
    &str => string
}

impl ToTokenStream for bool {
    fn to_toks(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new(&self.to_string(), Span::call_site()));
    }
}

impl<'a, T: ?Sized + ToTokenStream> ToTokenStream for &'a T {
    fn to_toks(&self, tokens: &mut TokenStream) {
        (**self).to_toks(tokens);
    }
}

impl<'a, T: ?Sized + ToTokenStream> ToTokenStream for &'a mut T {
    fn to_toks(&self, tokens: &mut TokenStream) {
        (**self).to_toks(tokens);
    }
}

fn to_toks_slice<T>(sl: &[T], tokens: &mut TokenStream)
where
    T: ToTokenStream,
{
    let mut arr_toks = TokenStream::new();
    for a in sl.iter() {
        let a_toks = a.to_tok_stream();
        let element = quote! { #a_toks, };
        arr_toks.extend(element);
    }
    let element = quote! { [#arr_toks] };
    tokens.extend(element);
}

impl<T> ToTokenStream for &[T]
where
    T: ToTokenStream,
{
    fn to_toks(&self, tokens: &mut TokenStream) {
        to_toks_slice(self, tokens);
    }
}

impl<T, const N: usize> ToTokenStream for [T; N]
where
    T: ToTokenStream,
{
    fn to_toks(&self, tokens: &mut TokenStream) {
        to_toks_slice(self, tokens);
    }
}

impl ToTokenStream for String {
    fn to_toks(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! { #self });
    }
}

impl<T> ToTokenStream for Vec<T>
where
    T: ToTokenStream,
{
    fn to_toks(&self, tokens: &mut TokenStream) {
        let mut arr_toks = TokenStream::new();
        for a in self {
            let a_toks = a.to_tok_stream();
            let element = quote! { #a_toks, };
            arr_toks.extend(element);
        }
        let element = quote! { vec![#arr_toks] };
        tokens.extend(element);
    }
}

impl<T> ToTokenStream for Option<T>
where
    T: ToTokenStream,
{
    fn to_toks(&self, tokens: &mut TokenStream) {
        let element;
        match self {
            Some(a) => {
                let a_toks = a.to_tok_stream();
                element = quote! {
                    Some(#a_toks)
                };
            }
            None => {
                element = quote! { None };
            }
        }
        tokens.extend(element);
    }
}

macro_rules! build_tuple_trait {
    ($($id:ident),+;$($index:literal),+) => {
        fn to_toks(&self, tokens: &mut TokenStream) {
            // As of Rust 1.69, limitations in the macro system mean
            // we can't use tuple indexing with the form self.$index,
            // so we destructure and use shadowing instead.
            let ($($id),+) = self;
            $(let $id = $id.to_tok_stream();)+
            let element = quote! { ($(#$id),+) };
            tokens.extend(element);
        }
    };
}

impl<T1, T2> ToTokenStream for (T1, T2)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
{
    build_tuple_trait!(t1, t2; 0, 1);
}

impl<T1, T2, T3> ToTokenStream for (T1, T2, T3)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
    T3: ToTokenStream,
{
    build_tuple_trait!(t1, t2, t3; 0, 1, 2);
}

impl<T1, T2, T3, T4> ToTokenStream for (T1, T2, T3, T4)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
    T3: ToTokenStream,
    T4: ToTokenStream,
{
    build_tuple_trait!(t1, t2, t3, t4; 0, 1, 2, 3);
}

impl<T1, T2, T3, T4, T5> ToTokenStream for (T1, T2, T3, T4, T5)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
    T3: ToTokenStream,
    T4: ToTokenStream,
    T5: ToTokenStream,
{
    build_tuple_trait!(t1, t2, t3, t4, t5; 0, 1, 2, 3, 4);
}

impl<T1, T2, T3, T4, T5, T6> ToTokenStream for (T1, T2, T3, T4, T5, T6)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
    T3: ToTokenStream,
    T4: ToTokenStream,
    T5: ToTokenStream,
    T6: ToTokenStream,
{
    build_tuple_trait!(t1, t2, t3, t4, t5, t6; 0, 1, 2, 3, 4, 5);
}

impl<T1, T2, T3, T4, T5, T6, T7> ToTokenStream for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
    T3: ToTokenStream,
    T4: ToTokenStream,
    T5: ToTokenStream,
    T6: ToTokenStream,
    T7: ToTokenStream,
{
    build_tuple_trait!(t1, t2, t3, t4, t5, t6, t7; 0, 1, 2, 3, 4, 5, 6);
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> ToTokenStream for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
    T3: ToTokenStream,
    T4: ToTokenStream,
    T5: ToTokenStream,
    T6: ToTokenStream,
    T7: ToTokenStream,
    T8: ToTokenStream,
{
    build_tuple_trait!(t1, t2, t3, t4, t5, t6, t7, t8; 0, 1, 2, 3, 4, 5, 6, 7);
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9> ToTokenStream for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
    T3: ToTokenStream,
    T4: ToTokenStream,
    T5: ToTokenStream,
    T6: ToTokenStream,
    T7: ToTokenStream,
    T8: ToTokenStream,
    T9: ToTokenStream,
{
    build_tuple_trait!(t1, t2, t3, t4, t5, t6, t7, t8, t9; 0, 1, 2, 3, 4, 5, 6, 7, 8);
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> ToTokenStream
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
    T3: ToTokenStream,
    T4: ToTokenStream,
    T5: ToTokenStream,
    T6: ToTokenStream,
    T7: ToTokenStream,
    T8: ToTokenStream,
    T9: ToTokenStream,
    T10: ToTokenStream,
{
    build_tuple_trait!(t1, t2, t3, t4, t5, t6, t7, t8, t9, t10; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> ToTokenStream
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
    T3: ToTokenStream,
    T4: ToTokenStream,
    T5: ToTokenStream,
    T6: ToTokenStream,
    T7: ToTokenStream,
    T8: ToTokenStream,
    T9: ToTokenStream,
    T10: ToTokenStream,
    T11: ToTokenStream,
{
    build_tuple_trait!(t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> ToTokenStream
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12)
where
    T1: ToTokenStream,
    T2: ToTokenStream,
    T3: ToTokenStream,
    T4: ToTokenStream,
    T5: ToTokenStream,
    T6: ToTokenStream,
    T7: ToTokenStream,
    T8: ToTokenStream,
    T9: ToTokenStream,
    T10: ToTokenStream,
    T11: ToTokenStream,
    T12: ToTokenStream,
{
    build_tuple_trait!(t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
}

/// An implementation detail, exposing parts of external crates used by `rustifact`.
///
/// API stability is not guaranteed here.
pub mod internal {
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
}

#[doc(hidden)]
#[macro_export]
macro_rules! __get_tokens_array_multi {
    ($data:expr, $get_inner:expr) => {{
        let mut tokens = rustifact::internal::TokenStream::new();
        for element in $data.iter().map($get_inner) {
            tokens.extend(rustifact::internal::quote! { #element, });
        }
        rustifact::internal::quote! { [#tokens] }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __get_tokens_vector_fn_multi {
    ($data:expr, $get_inner:expr) => {{
        let mut tokens = rustifact::internal::TokenStream::new();
        for element in $data.iter().map($get_inner) {
            tokens.extend(rustifact::internal::quote! { #element, });
        }
        rustifact::internal::quote! { vec![#tokens] }
    }};
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
    ($id_name:ident) => {{
        format!(
            "{}/rustifact_{}_{}.rs",
            std::env::var("OUT_DIR").unwrap(),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            stringify!($id_name),
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
    ($id_name:ident, $tokens:expr) => {
        let path_str = rustifact::__path_from_id!($id_name);
        let path = std::path::Path::new(&path_str);
        let syntax_tree = rustifact::internal::parse_file(&$tokens.to_string()).unwrap();
        let formatted = rustifact::internal::unparse(&syntax_tree);
        std::fs::write(&path, formatted).unwrap();
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
        rustifact::__write_tokens_with_internal!($id_name, tokens);
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
        rustifact::__write_tokens_with_internal!($id_name, tokens);
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
        rustifact::__write_tokens_with_internal!($id_group, toks);
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
        rustifact::__write_tokens_with_internal!($id_struct, toks_struct);
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
        rustifact::__write_tokens_with_internal!($id_group, toks);
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

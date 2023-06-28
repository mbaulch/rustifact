// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! # Rustifact
//!
//! _A seamless bridge between a build script and the main crate._
//!
//! # Usage steps
//!
//! 1. (Optional*) Implement the [`ToTokenStream`] trait for each of your build script's 'exported' types.
//! * This is predefined for primitive types ([`u8`], [`i32`], [`char`], [`bool`], ...), [`slice`]s,
//! [`array`]s, and [`Vec`]s. This step is only necessary if you're exporting your own types.
//! * These types should be implemented in a separate crate, so they're usable from the build script
//! _and_ the main crate.
//!
//! 2. Generate the required data in your build script.
//!
//! 3. Export your data with any combination of [`write_array_fn`], [`write_const_array`],
//! [`write_static_array`], and [`write_vector_fn`].
//!
//! 4. In the main part of your crate (within `src/`) import your data with [`use_symbols`].
//!
//! (*) We expect to automate this step soon by providing suitable `[#derive(...)]` macros.
//!
//! # Motivation
//! When it comes to generating computationally intensive artifacts at compile time, we have
//! many tools at our disposal: build scripts (build.rs), declarative macros (macro_rules!),
//! procedural macros, and increasingly, const functions. Each of these methods, however,
//! brings its own set of challenges.
//!
//! Issues with namespaces and types can arise from using build scripts and macros. Const functions,
//! while useful, may cause performance issues during compilation. Build scripts can make file management
//! complex. The number of library functions available to macros and const functions is limited.
//! Declarative macros suffer from a lack of expressiveness, and both macros and const functions can
//! encounter problems with environmental isolation.
//!
//! Rustifact has been designed as a streamlined abstraction layer that simplifies the use of build scripts.
//! By mitigating these complexities, Rustifact offers a more efficient approach for handling
//! compile-time computations in Rust.
//!
//! # A simple example
//! build.rs
//! ```no_run
//! use rustifact::ToTokenStream;
//!
//!fn main() {
//!    let mut city_data: Vec<(String, u32)> = Vec::new();
//!    for i in 1..=1000 {
//!        let city_name = format!("City{}", i);
//!        let population = i * 1000; // Dummy population data
//!        city_data.push((city_name, population));
//!    }
//!    // Let's make city_data accessible from the main crate. We'll write it to
//!    // a static array CITY_DATA where the type of each element is (&'static str, u32).
//!    // Note that Strings are converted to static string slices by default.
//!    //
//!    rustifact::write_static_array!(CITY_DATA, (&'static str, u32), &city_data);
//!    //
//!    // Alternatively, this could be written:
//!    // rustifact::write_static_array!(CITY_DATA, (&'static str, u32) : 1, &city_data);
//!    //
//!    // When the dimension is unspecified, the default is dimension 1.
//!    //
//!    // Passing city_data as a slice allows it to be treated as an
//!    // array. Note that this would not have been possible if its elements were heap allocated.
//!    // In that case, write_array_fn or write_vector_fn would need to be used.
//!}
//! ```
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
//! Please note that _Rustifact_ is in an early development stage. While it is utilised in at least one
//! commercial project, it lacks extensive testing and polishing. Overall, it is unlikely to cause unpleasant
//! surprises, though there may be edge cases that haven't yet been discovered.
//! As the API surface is minimal, it's unlikely that API changes would cause major headaches,
//! though be warned that some breaking changes may occur in the future.
//!

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
    /// A re-export of the `prettyplease` crate's `unparse` function.
    pub use prettyplease::unparse;
    /// A re-export of the `proc_macro2` crate's `TokenStream` type.
    pub use proc_macro2::TokenStream;
    /// A re-export of the `quote` crate's `quote!` macro.
    pub use quote::quote;
    /// A re-export of the `syn` quote's `parse_file` function.
    pub use syn::parse_file;
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
macro_rules! __write_array_with_internal {
    ($const_static:ident, $id_name:ident, $arr_type:expr, $tokens_data:expr) => {
        let arr_type = $arr_type;
        let tokens_data = $tokens_data;
        let tokens = rustifact::internal::quote! {
            $const_static $id_name: #arr_type = #tokens_data;
        };
        rustifact::__write_tokens_with_internal!($id_name, tokens);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __write_fn_with_internal {
    ($_:ident, $id_name:ident, $vec_type:expr, $tokens_data:expr) => {
        let vec_type = $vec_type;
        let tokens_data = $tokens_data;
        let tokens = rustifact::internal::quote! {
            fn $id_name() -> #vec_type { #tokens_data }
        };
        rustifact::__write_tokens_with_internal!($id_name, tokens);
    };
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
    ) => {
        rustifact::__assert_dim!($dim, $data);
        let tokens_data = rustifact::$get_tokens!($dim, $data);
        let arr_type = rustifact::$get_type!($dim, $t, $data);
        rustifact::$write_internal!($const_static, $id_name, arr_type, tokens_data);
    };
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

#[macro_export]
macro_rules! write_static {
    ($id:ident, $t:ty, $data:expr) => {
        let data: $t = $data;
        rustifact::__write_array_with_internal!(
            static,
            $id,
            rustifact::internal::quote! { $t },
            data.to_tok_stream()
        );
    };
}

#[macro_export]
macro_rules! write_const {
    ($id:ident, $t:ty, $data:expr) => {
        let data: $t = $data;
        rustifact::__write_array_with_internal!(
            const,
            $id,
            rustifact::internal::quote! { $t },
            data.to_tok_stream()
        );
    };
}

#[macro_export]
macro_rules! write_fn {
    ($id:ident, $t:ty, $data:expr) => {
        let data: $t = $data;
        rustifact::__write_fn_with_internal!(
            const,
            $id,
            rustifact::internal::quote! { $t },
            data.to_tok_stream()
        );
    };
}

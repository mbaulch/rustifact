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

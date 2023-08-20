use crate::tokens::ToTokenStream;
use proc_macro2::TokenStream;
use quote::quote;

/// A compile time builder for an immutable set.
///
/// Produces a highly optimised `Set` when output (for example, by `write_static!`) from the build script.
/// Internally, this is a wrapper for `phf_codegen::Set` from the excellent
/// [phf_codegen](https://crates.io/crates/phf_codegen) crate.
///
/// *This API requires the following crate feature to be activated: `set`*

pub struct SetBuilder<T>(phf_codegen::Set<T>);

/// An immutable set with lookup via a perfect hash function.
///
/// Constructable at compile time with a `BuildSet`. Unlike an `OrderedSet`, no iteration order is specified.
/// Internally, this is a wrapper for `phf::Set` from the excellent
/// [phf](https://crates.io/crates/phf) crate.
///
/// *This API requires the following crate feature to be activated: `set`*
pub struct Set<T: 'static>(phf::Set<T>);

impl<T> SetBuilder<T>
where
    T: ToTokenStream + std::hash::Hash + phf_shared::PhfHash + Eq + phf_shared::FmtConst,
{
    pub fn new() -> SetBuilder<T> {
        let mut internal = phf_codegen::Set::new();
        internal.phf_path("rustifact::internal::phf");
        SetBuilder(internal)
    }

    #[inline]
    pub fn entry(&mut self, value: T) {
        self.0.entry(value);
    }
}

impl<T> Set<T> {
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn contains<U>(&self, value: &U) -> bool
    where
        U: phf_shared::PhfHash + Eq + ?Sized,
        T: phf_shared::PhfBorrow<U>,
    {
        self.0.contains(value)
    }

    #[inline]
    pub fn get_key<U>(&self, value: &U) -> Option<&T>
    where
        U: phf_shared::PhfHash + Eq + ?Sized,
        T: phf_shared::PhfBorrow<U>,
    {
        self.0.get_key(value)
    }

    #[inline]
    pub fn iter(&self) -> phf::set::Iter<'_, T> {
        self.0.iter()
    }

    /// An implementation detail. You shouldn't need to call this function.
    #[inline]
    pub const fn init_raw(set: phf::Set<T>) -> Set<T> {
        Set(set)
    }
}

impl<T> ToTokenStream for SetBuilder<T>
where
    T: ToTokenStream + std::hash::Hash + phf_shared::PhfHash + Eq + phf_shared::FmtConst,
{
    fn to_toks(&self, tokens: &mut TokenStream) {
        let set_toks_str = self.0.build().to_string();
        if let Ok(t) = crate::internal::parse_str::<syn::Expr>(&set_toks_str) {
            tokens.extend(quote! { rustifact::Set::init_raw(#t) });
        } else {
            panic!("Couldn't parse the expression '{}'", set_toks_str);
        }
    }
}

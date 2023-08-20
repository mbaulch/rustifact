use crate::tokens::ToTokenStream;
use proc_macro2::TokenStream;
use quote::quote;

/// A compile time builder for an order-preserving immutable map.
///
/// Produces a highly optimised `OrderedMap` when output (for example, by `write_static!`) from the build script.
///
/// Internally, this is a wrapper for `phf_codegen::OrderedMap` from the excellent
/// [phf_codegen](https://crates.io/crates/phf_codegen) crate.
///
/// *This API requires the following crate feature to be activated: `map`*
pub struct OrderedMapBuilder<K, V>(phf_codegen::OrderedMap<K>, std::marker::PhantomData<V>);

/// An order-preserving immutable map with lookup via a perfect hash function.
///
/// Constructable at compile time with a `BuildOrderedMap`. Unlike a `Map`, iteration order is guaranteed to
/// match the definition order.
///
/// Internally, this is a wrapper for `phf::OrderedMap` from the excellent
/// [phf](https://crates.io/crates/phf) crate.
///
/// *This API requires the following crate feature to be activated: `map`*
pub struct OrderedMap<K: 'static, V: 'static>(phf::OrderedMap<K, V>);

impl<K, V> OrderedMapBuilder<K, V>
where
    K: ToTokenStream + std::hash::Hash + phf_shared::PhfHash + Eq + phf_shared::FmtConst,
    V: ToTokenStream,
{
    pub fn new() -> OrderedMapBuilder<K, V> {
        let mut internal = phf_codegen::OrderedMap::new();
        internal.phf_path("rustifact::internal::phf");
        OrderedMapBuilder(internal, std::marker::PhantomData)
    }

    #[inline]
    pub fn entry(&mut self, key: K, value: V) {
        self.0.entry(key, &value.to_tok_stream().to_string());
    }
}

impl<K, V> OrderedMap<K, V> {
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn contains_key<T>(&self, key: &T) -> bool
    where
        T: phf_shared::PhfHash + Eq + ?Sized,
        K: phf_shared::PhfBorrow<T>,
    {
        self.0.contains_key(key)
    }

    #[inline]
    pub fn get<T>(&self, key: &T) -> Option<&V>
    where
        T: phf_shared::PhfHash + Eq + ?Sized,
        K: phf_shared::PhfBorrow<T>,
    {
        self.0.get(key)
    }

    #[inline]
    pub fn get_key<T>(&self, key: &T) -> Option<&K>
    where
        T: phf_shared::PhfHash + Eq + ?Sized,
        K: phf_shared::PhfBorrow<T>,
    {
        self.0.get_key(key)
    }

    #[inline]
    pub fn get_entry<T>(&self, key: &T) -> Option<(&K, &V)>
    where
        T: phf_shared::PhfHash + Eq + ?Sized,
        K: phf_shared::PhfBorrow<T>,
    {
        self.0.get_entry(key)
    }

    #[inline]
    pub fn get_index<T>(&self, key: &T) -> Option<usize>
    where
        T: phf_shared::PhfHash + Eq + ?Sized,
        K: phf_shared::PhfBorrow<T>,
    {
        self.0.get_index(key)
    }

    #[inline]
    pub fn index<T>(&self, index: usize) -> Option<(&K, &V)> {
        self.0.index(index)
    }

    #[inline]
    pub fn entries(&self) -> phf::ordered_map::Entries<'_, K, V> {
        self.0.entries()
    }

    #[inline]
    pub fn keys(&self) -> phf::ordered_map::Keys<'_, K, V> {
        self.0.keys()
    }

    #[inline]
    pub fn values(&self) -> phf::ordered_map::Values<'_, K, V> {
        self.0.values()
    }

    /// An implementation detail. You shouldn't need to call this function.
    #[inline]
    pub const fn init_raw(map: phf::OrderedMap<K, V>) -> OrderedMap<K, V> {
        OrderedMap(map)
    }
}

impl<K, V> ToTokenStream for OrderedMapBuilder<K, V>
where
    K: ToTokenStream + std::hash::Hash + phf_shared::PhfHash + Eq + phf_shared::FmtConst,
{
    fn to_toks(&self, tokens: &mut TokenStream) {
        let map_toks_str = self.0.build().to_string();
        if let Ok(t) = crate::internal::parse_str::<syn::Expr>(&map_toks_str) {
            tokens.extend(quote! { rustifact::OrderedMap::init_raw(#t) });
        } else {
            panic!("Couldn't parse the expression '{}'", map_toks_str);
        }
    }
}

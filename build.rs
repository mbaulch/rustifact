// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Implemented for compatibility with use_symbols in the main crate
macro_rules! path_from_id {
    ($id_name:ident) => {{
        format!(
            "{}/rustifact_{}_{}.rs",
            std::env::var("OUT_DIR").unwrap(),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            stringify!($id_name),
        )
    }};
}

const DOC_HIDDEN: &'static str = "#[doc(hidden)]";
const MACRO_HEADER: &'static str = r#"
#[macro_export]
macro_rules! "#;

fn counting_entry_for(delta: i32, impl_id: &str, dim: usize) -> String {
    format!(
        "    ({}, $($args:tt),+) => {{ rustifact::{}!({}, $($args),+) }};",
        dim,
        impl_id,
        dim as i32 + delta,
    )
}

macro_rules! write_counting {
    ($delta:expr, $id:ident, $id_impl:expr) => {
        let path_str = path_from_id!($id);
        let path = std::path::Path::new(&path_str);
        let id = stringify!($id);
        let id_impl = stringify!($id_impl);
        let s = format!(
            "{}\n{} {} {{\n{}\n}}",
            DOC_HIDDEN,
            MACRO_HEADER,
            id,
            (1..=NUM_DIMS)
                .into_iter()
                .map(|d| counting_entry_for($delta, id_impl, d))
                .collect::<Vec<String>>()
                .join("\n")
        );
        std::fs::write(&path, s).unwrap();
    };
}

fn public_base_entry_for(id: &str) -> String {
    format!(
        "($id:ident, $t:ty, $data:expr) => {{ rustifact::{}!($id, $t : 1, $data); }};",
        id
    )
}

fn public_entry_for(dim: usize, const_static: &str, params_extra: &str) -> String {
    format!(
        "    ($id:ident, $t:ty : {}, $data:expr) => {{ rustifact::__write_with!({}, {}, $id, $t, $data, {}) }};",
        dim,
        dim,
        const_static,
        params_extra
    )
}

macro_rules! write_public {
    ($id:ident, $const_static:ident, $params_extra:expr, $doc:expr) => {
        let path_str = path_from_id!($id);
        let path = std::path::Path::new(&path_str);
        let id = stringify!($id);
        let const_static = stringify!($const_static);
        let s = format!(
            "#[doc = \"{}\"]\n{} {} {{\n{}\n{}\n}}",
            $doc,
            MACRO_HEADER,
            id,
            public_base_entry_for(id),
            (1..=NUM_DIMS)
                .into_iter()
                .map(|d| public_entry_for(d, const_static, $params_extra))
                .collect::<Vec<String>>()
                .join("\n")
        );
        std::fs::write(&path, s).unwrap();
    };
}

// The number of dimensions supported by Rustifact. Adjustable via NUM_DIMS.
// The only reason we don't support more is that limitations in Rust's macro system (as of Rust 1.69)
// require this code generation for each dimension, and additionally, we wish to minimise code bloat.
// It seems very unlikely that arrays or vectors are likely to be nested beyond depth 16.
const NUM_DIMS: usize = 16;

fn main() {
    write_counting!(-1, __vector_type, __vector_type_impl);
    write_counting!(-1, __array_type, __array_type_impl);
    write_counting!(-1, __assert_dim, __assert_dim_impl);
    write_counting!(-1, __get_tokens_vector_fn, __get_tokens_vector_fn_impl);
    write_counting!(-1, __get_tokens_array, __get_tokens_array_impl);
    write_counting!(0, __write_with, __write_with_impl);
    write_public!(
        write_static_array,
        static,
        "__get_tokens_array, __array_type, __write_with_internal",
        r#"Write an array to a static context.

Makes the array, array reference, or array slice available for import into the main crate via [`use_symbols`].

## Parameters
* `$id`: the name/identifier to give the exported array
* `$t`: the type of elements of the exported array will contain. Optionally followed by `: DIM`
where `DIM` is the dimension (1, 2, 3, ...) of the array. The dimension defaults to 1 when unspecified.
* `$data`: the contents of the array. May be an array, an array reference, or array slice.

## Further notes
* Must be called from a build script (build.rs) only.
* If the array elements are heap allocated, use [`write_array_fn`] instead."#
    );
    write_public!(
        write_const_array,
        const,
        "__get_tokens_array, __array_type, __write_with_internal",
        r#"Write an array to a const context.

Makes the array, array reference, or array slice available for import into the main crate via [`use_symbols`].
Stack allocated types (such as [`slice`]s and [`array`]s) may be returned.

## Parameters
* `$id`: the name/identifier to give the exported array
* `$t`: the type of elements of the exported array will contain. Optionally followed by `: DIM`
where `DIM` is the dimension (1, 2, 3, ...) of the array. The dimension defaults to 1 when unspecified.
* `$data`: the contents of the array. May be an array, an array reference, or array slice.

## Further notes
* Must be called from a build script (build.rs) only.
* If the array is large and referenced many times, this will lead to code bloat. In such a case,
consider carefully whether [`write_static_array`] would be more appropriate instead.
* If the array elements are heap allocated, use [`write_array_fn`] instead."#
    );
    write_public!(
        write_array_fn,
        dummy,
        "__get_tokens_array, __array_type, __write_fn_with_internal",
        r#"Write an array or vector to an array getter function.

Generates an array-returning function available for import into the main crate via [`use_symbols`].

## Parameters
* `$id`: the name/identifier to give the exported array-returning function.
* `$t`: the type of elements of the exported array will contain. Optionally followed by `: DIM`
where `DIM` is the dimension (1, 2, 3, ...) of the array. The dimension defaults to 1 when unspecified.
* `$data`: the contents of the array to be returned. May be an array, an array reference, or array slice.

## Further notes
* Must be called from a build script (build.rs) only.
* If the array elements are not heap allocated, consider using [`write_static_array`] or [`write_const_array`] instead."#
    );
    write_public!(
        write_vector_fn,
        dummy,
        "__get_tokens_vector_fn, __vector_type, __write_fn_with_internal",
        r#"Write an array or vector to a vector getter function.

Generates an array-returning function available for import into the main crate via [`use_symbols`].

## Parameters
* `$id`: the name/identifier to give the exported array-returning function.
* `$t`: the type of elements of the exported array will contain. Optionally followed by `: DIM`
where `DIM` is the dimension (1, 2, 3, ...) of the array. The dimension defaults to 1 when unspecified.
* `$data`: the contents of the vector to be returned. May be a `Vec`, an array, an array reference,
or array slice.

## Further notes
* Must be called from a build script (build.rs) only.
* If the output won't be mutated, consider using [`write_array_fn`] instead."#
    );
}

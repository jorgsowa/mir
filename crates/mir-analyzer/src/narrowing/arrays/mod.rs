//! The array-narrowing bounded context: array-shape path tracking
//! (`isset()`/`empty()`/`array_key_exists()` on `$var['key']`-style
//! accesses), `in_array()`/`array_search()` haystack-literal narrowing, and
//! `count()`/`array_key_first()`/`array_key_last()` narrowing.
//!
//! Nested (unlike this crate's other narrowing submodules) to visually mark
//! it as its own bounded context — this cluster is large enough, and
//! internally cross-referential enough, to warrant a subdirectory of its own
//! rather than another flat file alongside `core.rs`/`literals.rs`/etc.

mod count;
mod in_array;
mod key_exists;
mod shapes;

pub(super) use count::{
    extract_array_key_first_or_last_arg, extract_array_key_first_or_last_static_prop_arg,
    extract_count_arg, extract_count_static_prop_arg, narrow_array_count_comparison,
    narrow_array_count_condition, narrow_array_key_first_or_last_null,
    narrow_prop_array_count_comparison, narrow_prop_array_key_first_or_last_null,
    narrow_static_prop_array_count_comparison, narrow_static_prop_array_key_first_or_last_null,
};
pub(super) use in_array::{
    extract_haystack_type, in_array_loose_narrowing_is_safe, narrow_in_array_condition,
    narrow_to_haystack_values,
};
pub(super) use key_exists::narrow_array_key_exists_condition;
pub(super) use shapes::{
    array_access_base_target, narrow_array_emptiness_condition,
    narrow_container_non_null_non_false, narrow_empty_shape_key, narrow_isset_shape_key,
    narrow_isset_shape_key_false,
};

mod arrays;
#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "url")]
mod url;
mod field_rename;
mod flatten;
mod generic_fields;
mod generics;
mod indexmap;
mod list;
mod nested;
mod optional_field;
mod raw_idents;
mod simple;
mod skip;
mod struct_rename;
mod struct_tag;
mod tuple;
mod type_override;
mod union;
mod union_rename;
mod union_serde;
mod union_with_data;
mod union_with_internal_tag;
mod unit;

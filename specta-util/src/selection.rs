/// Specta compatible selection of struct fields.
///
/// ```rust
/// use specta::{selection, ts::inline_ref};
///
/// #[derive(Clone)]
/// struct MyStruct {
///     name: String,
///     age: i32,
///     is_verified: bool,
///     password: String,
/// }
///
/// let person = MyStruct {
///     name: "Monty".into(),
///     age: 7,
///     is_verified: true,
///     password: "password".into(),
/// };
/// let people = vec![person.clone(), person.clone()];
///
/// // Selection creates an anonymous struct with the subset of fields you want.
/// assert_eq!(inline_ref(&selection!(person, {
///     name,
///     age
/// }), &Default::default()).unwrap(), "{ name: string; age: number }");
///
/// // You can apply the selection to an array.
/// assert_eq!(inline_ref(&selection!(people, [{
///     name,
///     age
/// }]), &Default::default()).unwrap(), "{ name: string; age: number }[]");
/// ```
// TODO: better docs w/ example
#[cfg(feature = "serde")]
#[cfg_attr(docsrs2, doc(cfg(feature = "serde")))]
#[macro_export]
macro_rules! selection {
    ( $s:expr, { $($n:ident),+ $(,)? } ) => {{
        #[allow(non_camel_case_types)]
        mod selection {
            #[derive(serde::Serialize, $crate::Type)]
            #[specta(inline)]
            pub struct Selection<$($n,)*> {
                $(pub $n: $n),*
            }
        }
        use selection::Selection;
        #[allow(non_camel_case_types)]
        Selection { $($n: $s.$n,)* }
    }};
    ( $s:expr, [{ $($n:ident),+ $(,)? }] ) => {{
        #[allow(non_camel_case_types)]
        mod selection {
            #[derive(serde::Serialize, $crate::Type)]
            #[specta(inline)]
            pub struct Selection<$($n,)*> {
                $(pub $n: $n,)*
            }
        }
        use selection::Selection;
        #[allow(non_camel_case_types)]
        $s.into_iter().map(|v| Selection { $($n: v.$n,)* }).collect::<Vec<_>>()
    }};
}

// Tests in `src/tests/selection.rs` due to `$crate` issues

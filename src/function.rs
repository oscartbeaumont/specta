//! Support for exporting Rust function.
//!
//! TODO: Docs. Talk about how Specta doesn't export functions but it helps you to.

use crate::{FunctionType, TypeMap};

pub(crate) type ExportFn = &'static dyn Fn(&mut TypeMap) -> FunctionType;

pub struct Function {
    pub(crate) export_fn: ExportFn,
}

/// Returns a [`FunctionDataType`] for a given function that has been annotated with
/// [`specta`](macro@crate::specta).
///
/// # Examples
///
/// ```rust
/// use specta::{specta, func};
///
/// #[specta]
/// fn some_function(name: String, age: i32) -> bool {
///     true
/// }
///
/// fn main() {
///     let typ = func!(some_function);
///
///     assert_eq!(typ.name, "some_function");
///     assert_eq!(typ.args.len(), 2);
///     assert_eq!(typ.result, DataType::Primitive(PrimitiveType::bool));
/// }
/// ```
#[macro_export]
macro_rules! func {
    ($($function:path),* $(,)?) => {{
        // TODO: This caps out at `12` variants. Can we fix it?
        fn infer_array<const N: usize>(funcs: impl Into<[$crate::Function; N]>) -> [$crate::Function; N] {
            funcs.into()
        }

        infer_array((

            $(
                $crate::internal::__specta_paste! { [< __specta__fn__ $function>]!(@internal) },
            )*
        ))
    }};
}

use std::future::Future;

use crate::{DataType, DefOpts, Type};

/// Implemented by types that can be returned from a function annotated with
/// [`specta`](crate::specta).
pub trait FunctionOutput<TMarker> {
    /// Gets the type of the result as a [`DataType`].
    fn to_datatype(opts: DefOpts) -> DataType;
}

pub enum FunctionOutputMarker {}
impl<T: Type> FunctionOutput<FunctionOutputMarker> for T {
    fn to_datatype(opts: DefOpts) -> DataType {
        T::reference(opts, &[]).inner
    }
}

pub enum FunctionOutputFutureMarker {}
impl<F> FunctionOutput<FunctionOutputFutureMarker> for F
where
    F: Future,
    F::Output: Type,
{
    fn to_datatype(opts: DefOpts) -> DataType {
        F::Output::reference(opts, &[]).inner
    }
}

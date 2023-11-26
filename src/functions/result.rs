mod private {
    use std::future::Future;

    use crate::{DataType, Type, TypeMap};

    /// Implemented by types that can be returned from a function annotated with
    /// [`specta`](crate::specta).
    pub trait SpectaFunctionResult<TMarker> {
        /// Gets the type of the result as a [`DataType`].
        fn to_datatype(type_map: &mut TypeMap) -> DataType;
    }

    pub enum SpectaFunctionResultMarker {}
    impl<T: Type> SpectaFunctionResult<SpectaFunctionResultMarker> for T {
        fn to_datatype(type_map: &mut TypeMap) -> DataType {
            T::reference(type_map).inner
        }
    }

    pub enum SpectaFunctionResultFutureMarker {}
    impl<F> SpectaFunctionResult<SpectaFunctionResultFutureMarker> for F
    where
        F: Future,
        F::Output: Type,
    {
        fn to_datatype(type_map: &mut TypeMap) -> DataType {
            F::Output::reference(type_map).inner
        }
    }
}

pub(crate) use private::SpectaFunctionResult;

//! The magic that makes this crate work. This module does *NOT* follow semver at all.

// This macro must *NEVER* change signature, even in major releases!!!!!
// This is called by `specta` and `specta` depends on *any* version of this crate.
//
// This is a little hack to avoid the orphan rule. The code inside this is expanded inside `specta` so it can do `impl specta::Type for ...`.
#[macro_export]
macro_rules! impls {
    // This runs in the context of the `specta` crate so the `cfg`'s won't work as expected.
    () => {
        use crate::{DataType, Type, TypeMap};

        $crate::_feature_testing!();
    };
}

// TODO: Make a nicer abstraction for this

#[macro_export]
#[cfg(any(feature = "testing", docsrs))]
macro_rules! _feature_testing {
    () => {
        impl Type for specta_impls::Testing {
            fn inline(_: &mut TypeMap, _: &[DataType]) -> DataType {
                DataType::Any
            }
        }
    };
}

#[macro_export]
#[cfg(not(any(feature = "testing", docsrs)))]
macro_rules! _feature_testing {
    () => {};
}

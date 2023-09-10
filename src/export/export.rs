use crate::*;
use once_cell::sync::Lazy;
use std::sync::{PoisonError, RwLock, RwLockReadGuard};

// Global type store for collecting custom types to export.
pub(crate) static TYPES: Lazy<RwLock<TypeMap>> = Lazy::new(Default::default);

/// A lock type for iterating over the internal type map.
///
/// Holding this type will prevent any new types from being registered until it is dropped.
///
pub struct TypesIter {
    index: usize,
    lock: RwLockReadGuard<'static, TypeMap>,
}

impl Iterator for TypesIter {
    type Item = (SpectaID, Option<NamedDataType>);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = self.lock.iter().nth(self.index)?;
        self.index += 1;
        // We have to clone, because we can't invent a lifetime
        Some((*k, v.clone()))
    }
}

/// Get the global type store for collecting custom types to export.
pub fn get_types() -> TypesIter {
    let types = TYPES.read().unwrap_or_else(PoisonError::into_inner);

    TypesIter {
        index: 0,
        lock: types,
    }
}

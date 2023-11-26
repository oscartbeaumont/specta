use crate::*;
use once_cell::sync::Lazy;
use std::sync::{PoisonError, RwLock, RwLockReadGuard};

// Global type store for collecting custom types to export.
static TYPES: Lazy<RwLock<TypeMap>> = Lazy::new(Default::default);

/// A lock type for iterating over the internal type map.
///
/// Holding this type will prevent any new types from being registered until it is dropped.
///
pub struct TypesIter {
    index: usize,
    lock: RwLockReadGuard<'static, TypeMap>,
}

impl Iterator for TypesIter {
    type Item = (SpectaID, NamedDataType);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = self.lock.map.iter().nth(self.index)?;
        self.index += 1;
        // We have to clone, because we can't invent a lifetime
        Some((
            *k,
            v.clone()
                .expect("specta: `TypesIter` found a type placeholder!"),
        ))
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

// Called within ctor functions to register a type.
#[doc(hidden)]
pub fn register_ty<T: Type>() {
    let type_map = &mut *TYPES.write().unwrap_or_else(PoisonError::into_inner);

    // We call this for it's side effects on the `type_map`
    T::reference(type_map);
}

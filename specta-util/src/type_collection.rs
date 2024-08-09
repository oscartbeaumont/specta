use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    path::Path,
};

use specta::{datatype::NamedDataType, Language, NamedType, SpectaID, TypeMap};

/// Define a set of types which can be exported together
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCollection {
    // TODO: Make into a single `Vec<Enum>` to maintain global ordering?
    types: HashMap<SpectaID, fn(&mut TypeMap) -> NamedDataType>,
    constants: HashMap<Cow<'static, str>, serde_json::Value>, // TODO: Can we make this format agnostic?
}

impl Default for TypeCollection {
    fn default() -> Self {
        Self {
            types: Default::default(),
            constants: Default::default(),
        }
    }
}

impl TypeCollection {
    #[allow(unused)]
    pub(crate) fn from_raw(types: HashMap<SpectaID, fn(&mut TypeMap) -> NamedDataType>) -> Self {
        Self {
            types,
            constants: Default::default(),
        }
    }

    // TODO: Maybe register framework info for default header

    /// Join another type collection into this one.
    pub fn extend(&mut self, collection: impl Borrow<TypeCollection>) -> &mut Self {
        let collection = collection.borrow();
        self.types.extend(collection.types.iter());
        self.constants.extend(
            collection
                .constants
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );
        self
    }

    // TODO: Should you be allowed to merge in a `TypeMap`???

    // TODO: Should you be able to output a type_map from our internal registry

    /// Register a type with the collection.
    pub fn register<T: NamedType>(&mut self) -> &mut Self {
        self.types
            .insert(T::sid(), |type_map| T::definition_named_data_type(type_map));
        self
    }

    /// TODO
    // #[cfg(feature = "serde")] // TODO
    pub fn constant<T: serde::Serialize>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        value: T,
    ) -> &mut Self {
        self.constants
            .insert(name.into(), serde_json::to_value(value).unwrap()); // TODO: Error handling
        self
    }

    /// Export all the types in the collection into the given type map.
    pub fn collect(&self, mut type_map: &mut TypeMap) {
        for (sid, export) in self.types.iter() {
            let dt = export(&mut type_map);
            type_map.insert(*sid, dt);
        }
    }

    /// TODO
    pub fn export<L: Language>(&self, language: L) -> Result<String, L::Error> {
        let mut type_map = TypeMap::default();
        self.collect(&mut type_map);
        language.export(type_map) // TODO: &self.constants
    }

    // TODO: Maybe we could put `path` on `Language` and remove this?
    /// TODO
    pub fn export_to<L: Language>(
        &self,
        language: L,
        path: impl AsRef<Path>,
    ) -> Result<(), L::Error> {
        std::fs::write(path, self.export(language)?).map_err(Into::into)
    }
}

use std::borrow::Cow;

use crate::{NamedDataType, NamedType, Type};

#[derive(Debug, Default)]
pub struct Registry {
    types: Vec<NamedDataType>,
}

impl Registry {
    fn insert<T: NamedType>(&mut self) {
        todo!();
    }

    fn insert_with_name<T: Type>(&mut self, name: impl Into<Cow<'static, str>>) {
        todo!();
    }

    // TODO: Export

    // TODO: Iterator
}

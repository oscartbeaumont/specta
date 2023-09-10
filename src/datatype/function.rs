use std::borrow::Cow;

use crate::{DataType, Function, TypeMap};

/// Contains type information about a function annotated with [`specta`](macro@crate::specta).
/// Returned by [`func`].
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub(crate) asyncness: bool,
    pub(crate) name: Cow<'static, str>,
    pub(crate) args: Vec<(Cow<'static, str>, DataType)>,
    pub(crate) result: DataType,
    pub(crate) docs: Vec<Cow<'static, str>>,
}

impl FunctionType {
    /// Constructs a [`FunctionType`] from a [`Function`].
    pub fn new<const N: usize>(
        function: [Function; N],
        type_map: &mut TypeMap,
    ) -> [FunctionType; N] {
        todo!();
    }

    /// Returns whether the function is async.
    pub fn asyncness(&self) -> bool {
        self.asyncness
    }

    /// Returns the function's name.
    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    /// Returns the name and type of each of the function's arguments.
    pub fn args(&self) -> &Vec<(Cow<'static, str>, DataType)> {
        &self.args
    }

    /// Returns the return type of the function.
    pub fn result(&self) -> &DataType {
        &self.result
    }

    /// Returns the function's documentation. Detects both `///` and `#[doc = ...]` style documentation.
    pub fn docs(&self) -> &Vec<Cow<'static, str>> {
        &self.docs
    }
}

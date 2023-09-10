use std::collections::HashMap;

use specta::{advanced::SerdeError, Type};

use crate::ts::assert_ts;

// This type won't even compile with Serde macros
#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum A {
    // For internal tagging all variants must be a unit, named or *unnamed with a single variant*.
    A(String, u32),
}

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum B {
    // Is not a map-type so invalid.
    A(String),
}

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum C {
    // Is not a map-type so invalid.
    A(Vec<String>),
}

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum D {
    // Is a map type so valid.
    A(HashMap<String, String>),
}

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum E {
    // Null is valid (although it's not a map-type)
    A(()),
}

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum F {
    // `FInner` is untagged so this is *only* valid if it is (which it is)
    A(FInner),
}

#[derive(Type)]
#[serde(export = false, untagged)]
pub enum FInner {
    A(()),
}

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum G {
    // `GInner` is untagged so this is *only* valid if it is (which it is not)
    A(GInner),
}

#[derive(Type)]
#[serde(export = false, untagged)]
pub enum GInner {
    A(String),
}

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum H {
    // `HInner` is transparent so this is *only* valid if it is (which it is)
    A(HInner),
}

#[derive(Type)]
#[serde(export = false, transparent)]
pub struct HInner(());

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum I {
    // `IInner` is transparent so this is *only* valid if it is (which it is not)
    A(IInner),
}

#[derive(Type)]
#[serde(export = false, transparent)]
pub struct IInner(String);

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum L {
    // Internally tag enum with inlined field that is itself internally tagged
    #[specta(inline)]
    A(LInner),
}

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum LInner {
    A,
    B,
}

#[derive(Type)]
#[serde(export = false, tag = "type")]
pub enum M {
    // Internally tag enum with inlined field that is untagged
    // `MInner` is `null` - Test `B` in `untagged.rs`
    #[specta(inline)]
    A(MInner),
}

#[derive(Type)]
#[serde(export = false, untagged)]
pub enum MInner {
    A,
    B,
}

#[test]
fn internally_tagged() {
    assert_ts!(error; A, SerdeError::InvalidInternallyTaggedEnum);
    assert_ts!(error; B, SerdeError::InvalidInternallyTaggedEnum);
    assert_ts!(error; C, SerdeError::InvalidInternallyTaggedEnum);
    assert_ts!(D, "({ type: \"A\" } & { [key in string]: string })");
    assert_ts!(E, "({ type: \"A\" })");
    assert_ts!(F, "({ type: \"A\" } & FInner)");
    assert_ts!(error; G, SerdeError::InvalidInternallyTaggedEnum);
    assert_ts!(H, "({ type: \"A\" } & HInner)");
    assert_ts!(error; I, SerdeError::InvalidInternallyTaggedEnum);
    assert_ts!(L, "({ type: \"A\" } & ({ type: \"A\" } | { type: \"B\" }))");
    assert_ts!(M, "({ type: \"A\" })");
}

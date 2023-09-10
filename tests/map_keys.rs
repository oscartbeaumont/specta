use std::{collections::HashMap, convert::Infallible};

use specta::{advanced::SerdeError, Any, Type};

use crate::ts::{assert_ts, assert_ts_export};

// Export needs a `NamedDataType` but uses `Type::reference` instead of `Type::inline` so we test it.
#[derive(Type)]
#[specta(export = false)]
struct Regular(HashMap<String, ()>);

#[derive(Type)]
#[specta(export = false)]
struct RegularStruct {
    a: String,
}

#[derive(Type)]
#[specta(export = false, transparent)]
struct TransparentStruct(String);

#[derive(Type)]
#[specta(export = false)]
enum UnitVariants {
    A,
    B,
    C,
}

#[derive(Type)]
#[specta(export = false, untagged)]
enum UntaggedVariants {
    A(String),
    B(i32),
    C(u8),
}

#[derive(Type)]
#[specta(export = false, untagged)]
enum InvalidUntaggedVariants {
    A(String),
    B(i32, String),
    C(u8),
}

#[derive(Type)]
#[specta(export = false)]
enum Variants {
    A(String),
    B(i32),
    C(u8),
}

#[derive(Type)]
#[specta(export = false, transparent)]
pub struct MaybeValidKey<T>(T);

#[derive(Type)]
#[specta(export = false, transparent)]
pub struct ValidMaybeValidKey(HashMap<MaybeValidKey<String>, ()>);

#[derive(Type)]
#[specta(export = false, transparent)]
pub struct InvalidMaybeValidKey(HashMap<MaybeValidKey<()>, ()>);

#[test]
fn map_keys() {
    assert_ts!(HashMap<String, ()>, "{ [key in string]: null }");
    assert_ts_export!(Regular, "export type Regular = { [key in string]: null }");
    assert_ts!(HashMap<Infallible, ()>, "{ [key in never]: null }");
    assert_ts!(HashMap<Any, ()>, "{ [key in any]: null }");
    assert_ts!(HashMap<TransparentStruct, ()>, "{ [key in string]: null }");
    assert_ts!(HashMap<UnitVariants, ()>, "{ [key in \"A\" | \"B\" | \"C\"]: null }");
    assert_ts!(HashMap<UntaggedVariants, ()>, "{ [key in string | number]: null }");
    assert_ts!(
        ValidMaybeValidKey,
        "{ [key in MaybeValidKey<string>]: null }"
    );
    assert_ts_export!(
        ValidMaybeValidKey,
        "export type ValidMaybeValidKey = { [key in MaybeValidKey<string>]: null }"
    );

    assert_ts!(error; HashMap<() /* `null` */, ()>, SerdeError::InvalidMapKey);
    assert_ts!(error; HashMap<RegularStruct, ()>, SerdeError::InvalidMapKey);
    assert_ts!(error; HashMap<Variants, ()>, SerdeError::InvalidMapKey);
    assert_ts!(error; InvalidMaybeValidKey, SerdeError::InvalidMapKey);
    assert_ts_export!(error; InvalidMaybeValidKey, SerdeError::InvalidMapKey);
}

#![allow(deprecated)]

use std::{
    cell::RefCell,
    collections::HashMap,
    convert::Infallible,
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    path::PathBuf,
};

use serde::Serialize;
use specta::{
    ts::{BigIntExportBehavior, ExportConfig},
    Any, Type,
};

macro_rules! assert_ts {
    ($t:ty, $e:expr) => {
        assert_eq!(specta::ts::inline::<$t>(&Default::default()), Ok($e.into()))
    };

    (() => $expr:expr, $e:expr) => {
        let _: () = {
            fn assert_ty_eq<T: specta::Type>(_t: T) {
                assert_eq!(specta::ts::inline::<T>(&Default::default()), Ok($e.into()));
            }
            assert_ty_eq($expr);
        };
    };
}
pub(crate) use assert_ts;

macro_rules! assert_ts_export {
    ($t:ty, $e:expr) => {
        assert_eq!(specta::ts::export::<$t>(&Default::default()), Ok($e.into()))
    };
    ($t:ty, $e:expr; $cfg:expr) => {
        assert_eq!(specta::ts::export::<$t>($cfg), Ok($e.into()))
    };
}
pub(crate) use assert_ts_export;

// TODO: Unit test other `specta::Type` methods such as `::reference(...)`

#[test]
fn typescript_types() {
    assert_ts!(i8, "number");
    assert_ts!(u8, "number");
    assert_ts!(i16, "number");
    assert_ts!(u16, "number");
    assert_ts!(i32, "number");
    assert_ts!(u32, "number");
    assert_ts!(f32, "number");
    assert_ts!(f64, "number");

    assert_ts!(bool, "boolean");

    assert_ts!((), "null");
    assert_ts!((String, i32), "[string, number]");
    assert_ts!((String, i32, bool), "[string, number, boolean]");
    assert_ts!(
        (bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool),
        "[boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean]"
    );

    assert_ts!(String, "string");
    // impossible since Path as a generic is unsized lol
    // assert_ts!(Path, "string");
    assert_ts!(PathBuf, "string");
    assert_ts!(IpAddr, "string");
    assert_ts!(Ipv4Addr, "string");
    assert_ts!(Ipv6Addr, "string");
    assert_ts!(SocketAddr, "string");
    assert_ts!(SocketAddrV4, "string");
    assert_ts!(SocketAddrV6, "string");
    assert_ts!(char, "string");
    assert_ts!(&'static str, "string");

    assert_ts!(&'static bool, "boolean");
    assert_ts!(&'static i32, "number");

    assert_ts!(Vec<i32>, "number[]");
    assert_ts!(&[i32], "number[]");
    assert_ts!(&[i32; 5], "number[]");

    assert_ts!(Option<i32>, "number | null");

    // https://github.com/oscartbeaumont/specta/issues/88
    assert_ts!(Unit1, "null");
    assert_ts!(Unit2, "Record<string, never>");
    assert_ts!(Unit3, "[]");
    assert_ts!(Unit4, "null");
    assert_ts!(Unit5, r#""A""#);
    assert_ts!(Unit6, "{ A: [] }");
    assert_ts!(Unit7, "{ A: Record<string, never> }");

    assert_ts!(
        SimpleStruct,
        "{ a: number; b: string; c: [number, string, number]; d: string[]; e: string | null }"
    );
    assert_ts!(TupleStruct1, "number");
    assert_ts!(TupleStruct3, "[number, boolean, string]");

    assert_ts!(
        TestEnum,
        r#""Unit" | { Single: number } | { Multiple: [number, number] } | { Struct: { a: number } }"#
    );
    assert_ts!(RefStruct, "TestEnum");

    assert_ts!(
        InlinerStruct,
        "{ inline_this: { ref_struct: SimpleStruct; val: number }; dont_inline_this: RefStruct }"
    );

    assert_ts!(GenericStruct<i32>, "{ arg: number }");
    assert_ts!(GenericStruct<String>, "{ arg: string }");

    assert_ts!(
        FlattenEnumStruct,
        r#"({ tag: "One" } | { tag: "Two" } | { tag: "Three" }) & { outer: string }"#
    );

    assert_ts!(OverridenStruct, "{ overriden_field: string }");
    assert_ts!(HasGenericAlias, r#"{ [key: number]: string }"#);

    assert_ts!(SkipVariant, "{ A: string }");
    assert_ts!(SkipVariant2, r#"{ tag: "A"; data: string }"#);
    assert_ts!(SkipVariant3, "{ A: { a: string } }");

    assert_ts!(
        EnumMacroAttributes,
        "{ A: string } | { bbb: number } | { cccc: number } | { D: { a: string; bbbbbb: number } }"
    );

    assert_ts_export!(
        DocComments,
        "/**\n * Type level doc comment\n */\nexport type DocComments = { a: string }"
    );
    assert_ts_export!(DocComments, "export type DocComments = { a: string }"; &ExportConfig::new().comment_style(None));

    assert_ts!(Recursive, "{ a: number; children: Recursive[] }");

    assert_ts!(InlineEnumField, "{ A: { a: string } }");

    assert_ts!(InlineOptionalType, "{ optional_field: DocComments | null }");

    assert_ts_export!(
        RenameToValue,
        "export type RenameToValueNewName = { demo_new_name: number }"
    );

    assert_ts!(Rename, r#""OneWord" | "Two words""#);

    assert_ts!(TransparentType, r#"TransparentTypeInner"#);
    assert_ts!(TransparentType2, r#"null"#);
    assert_ts!(TransparentTypeWithOverride, r#"string"#);

    // I love serde but this is so mega cringe. Lack of support and the fact that `0..5` == `0..=5` is so dumb.
    assert_ts!(() => 0..5, r#"{ start: number; end: number }"#);
    // assert_ts!(() => 0.., r#"{ start: 0 }"#);
    // assert_ts!(() => .., r#""#);
    assert_ts!(() => 0..=5, r#"{ start: number; end: number }"#);
    // assert_ts!(() => ..5, r#"{ end: 5 }"#);
    // assert_ts!(() => ..=5, r#"{ end: 5 }"#);

    // https://github.com/oscartbeaumont/specta/issues/66
    assert_ts!([Option<u8>; 16], r#"(number | null)[]"#);

    // https://github.com/oscartbeaumont/specta/issues/65
    assert_ts!(HashMap<BasicEnum, ()>, r#"{ [key in "A" | "B"]: null }"#);

    // https://github.com/oscartbeaumont/specta/issues/60
    assert_ts!(Option<Option<Option<Option<i32>>>>, r#"number | null"#);

    // https://github.com/oscartbeaumont/specta/issues/71
    assert_ts!(Vec<DocComments>, r#"{ a: string }[]"#);

    // https://github.com/oscartbeaumont/specta/issues/77
    assert_eq!(
        specta::ts::inline::<std::time::SystemTime>(
            &ExportConfig::new().bigint(BigIntExportBehavior::Number)
        ),
        Ok(r#"{ duration_since_epoch: number; duration_since_unix_epoch: number }"#.into())
    );
    assert_eq!(
        specta::ts::inline::<std::time::SystemTime>(
            &ExportConfig::new().bigint(BigIntExportBehavior::String)
        ),
        Ok(r#"{ duration_since_epoch: string; duration_since_unix_epoch: number }"#.into())
    );

    assert_eq!(
        specta::ts::inline::<std::time::Duration>(
            &ExportConfig::new().bigint(BigIntExportBehavior::Number)
        ),
        Ok(r#"{ secs: number; nanos: number }"#.into())
    );
    assert_eq!(
        specta::ts::inline::<std::time::Duration>(
            &ExportConfig::new().bigint(BigIntExportBehavior::String)
        ),
        Ok(r#"{ secs: string; nanos: number }"#.into())
    );

    assert_ts!(HashMap<BasicEnum, i32>, r#"{ [key in "A" | "B"]: number }"#);
    assert_ts_export!(
        EnumReferenceRecordKey,
        "export type EnumReferenceRecordKey = { a: { [key in BasicEnum]: number } }"
    );

    assert_ts!(
        FlattenOnNestedEnum,
        r#"({ type: "a"; value: string } | { type: "b"; value: number }) & { id: string }"#
    );

    assert_ts!(PhantomData<()>, r#"null"#);
    assert_ts!(PhantomData<String>, r#"null"#);
    assert_ts!(Infallible, r#"never"#);

    assert_ts!(Result<String, i32>, r#"string | number"#);
    assert_ts!(Result<i16, i32>, r#"number"#);

    #[cfg(feature = "either")]
    {
        assert_ts!(either::Either<String, i32>, r#"string | number"#);
        assert_ts!(either::Either<i16, i32>, r#"number"#);
    }

    assert_ts!(Any, r#"any"#);

    assert_ts!(MyEmptyInput, "Record<string, never>");
    assert_ts_export!(
        MyEmptyInput,
        "export type MyEmptyInput = Record<string, never>"
    );

    // assert_ts_export!(DeprecatedType, "");
    // assert_ts_export!(DeprecatedTypeWithMsg, "");
    // assert_ts_export!(DeprecatedFields, "");
}

#[derive(Type)]
#[specta(export = false)]
struct Unit1;

#[derive(Type)]
#[specta(export = false)]
struct Unit2 {}

#[derive(Type)]
#[specta(export = false)]
struct Unit3();

#[derive(Type)]
#[specta(export = false)]
struct Unit4(());

#[derive(Type)]
#[specta(export = false)]
enum Unit5 {
    A,
}

#[derive(Type)]
#[specta(export = false)]
enum Unit6 {
    A(),
}

#[derive(Type)]
#[specta(export = false)]
enum Unit7 {
    A {},
}

#[derive(Type)]
#[specta(export = false)]
struct SimpleStruct {
    a: i32,
    b: String,
    c: (i32, String, RefCell<i32>),
    d: Vec<String>,
    e: Option<String>,
}

#[derive(Type)]
#[specta(export = false)]
struct TupleStruct1(i32);

#[derive(Type)]
#[specta(export = false)]
struct TupleStruct3(i32, bool, String);

#[derive(Type)]
#[specta(export = false)]
#[specta(rename = "HasBeenRenamed")]
struct RenamedStruct;

#[derive(Type)]
#[specta(export = false)]
enum TestEnum {
    Unit,
    Single(i32),
    Multiple(i32, i32),
    Struct { a: i32 },
}

#[derive(Type)]
#[specta(export = false)]
struct RefStruct(TestEnum);

#[derive(Type)]
#[specta(export = false)]
struct InlineStruct {
    ref_struct: SimpleStruct,
    val: i32,
}

#[derive(Type)]
#[specta(export = false)]
struct InlinerStruct {
    #[specta(inline)]
    inline_this: InlineStruct,
    dont_inline_this: RefStruct,
}

#[derive(Type)]
#[specta(export = false)]
struct GenericStruct<T> {
    arg: T,
}

#[derive(Serialize, Type)]
#[specta(export = false)]
struct FlattenEnumStruct {
    outer: String,
    #[serde(flatten)]
    inner: FlattenEnum,
}

#[derive(Serialize, Type)]
#[specta(export = false)]
#[serde(tag = "tag", content = "test")]
enum FlattenEnum {
    One,
    Two,
    Three,
}

#[derive(Serialize, Type)]
#[specta(export = false)]
struct OverridenStruct {
    #[specta(type = String)]
    overriden_field: i32,
}

#[derive(Type)]
#[specta(export = false)]
struct HasGenericAlias(GenericAlias<i32>);

type GenericAlias<T> = std::collections::HashMap<T, String>;

#[derive(Serialize, Type)]
#[specta(export = false)]
enum SkipVariant {
    A(String),
    #[serde(skip)]
    B(i32),
    #[specta(skip)]
    C(i32),
}

#[derive(Serialize, Type)]
#[specta(export = false)]
#[serde(tag = "tag", content = "data")]
enum SkipVariant2 {
    A(String),
    #[serde(skip)]
    B(i32),
    #[specta(skip)]
    C(i32),
}

#[derive(Serialize, Type)]
#[specta(export = false)]
enum SkipVariant3 {
    A {
        a: String,
    },
    #[serde(skip)]
    B {
        b: i32,
    },
    #[specta(skip)]
    C {
        b: i32,
    },
}

#[derive(Type)]
#[specta(export = false)]
pub enum EnumMacroAttributes {
    A(#[specta(type = String)] i32),
    #[specta(rename = "bbb")]
    B(i32),
    #[specta(rename = "cccc")]
    C(#[specta(type = i32)] String),
    D {
        #[specta(type = String)]
        a: i32,
        #[specta(rename = "bbbbbb")]
        b: i32,
    },
}

/// Type level doc comment
#[derive(Type)]
#[specta(export = false)]
pub struct DocComments {
    /// Field level doc comment
    a: String,
}

#[derive(Type)]
#[specta(export = false)]
pub struct Recursive {
    a: i32,
    children: Vec<Recursive>,
}

#[derive(Type)]
#[specta(export = false)]

pub enum InlineEnumField {
    #[specta(inline)]
    A(DocComments),
}

#[derive(Type)]
#[specta(export = false)]
pub struct InlineOptionalType {
    #[specta(inline)]
    pub optional_field: Option<DocComments>,
}

const CONTAINER_NAME: &str = "RenameToValueNewName";
const FIELD_NAME: &str = "demo_new_name";

// This is very much an advanced API. It is not recommended to use this unless you know what your doing.
// For personal reference: Is used in PCR to apply an inflection to the dynamic name of the include/select macro.
#[derive(Type)]
#[specta(export = false, rename_from_path = CONTAINER_NAME)]
pub struct RenameToValue {
    #[specta(rename_from_path = FIELD_NAME)]
    pub demo: i32,
}

// Regression test for https://github.com/oscartbeaumont/specta/issues/56
#[derive(Type, serde::Serialize)]
#[specta(export = false)]
enum Rename {
    OneWord,
    #[serde(rename = "Two words")]
    TwoWords,
}

#[derive(Type, serde::Serialize)]
#[specta(export = false)]
pub struct TransparentTypeInner {
    inner: String,
}

#[derive(Type, serde::Serialize)]
#[specta(export = false)]
#[serde(transparent)]
pub struct TransparentType(pub(crate) TransparentTypeInner);

#[derive(Type, serde::Serialize)]
#[specta(export = false)]
#[serde(transparent)]
pub struct TransparentType2(pub(crate) ());

#[derive(serde::Serialize)]
pub struct NonTypeType;

#[derive(Type, serde::Serialize)]
#[specta(export = false)]
#[serde(transparent)]
pub struct TransparentTypeWithOverride(#[specta(type = String)] NonTypeType);

#[derive(Type, serde::Serialize)]
#[specta(export = false)]
pub enum BasicEnum {
    A,
    B,
}

#[derive(Type)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum NestedEnum {
    A(String),
    B(i32),
}

#[derive(Type)]
#[serde(rename_all = "camelCase")]
pub struct FlattenOnNestedEnum {
    id: String,
    #[serde(flatten)]
    result: NestedEnum,
}

#[derive(Type)]
#[specta(export = false)]
pub struct EnumReferenceRecordKey {
    a: HashMap<BasicEnum, i32>,
}

// https://github.com/oscartbeaumont/specta/issues/88
#[derive(Type)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub(super) struct MyEmptyInput {}

// #[derive(Type)]
// #[specta(export = false)]
// #[deprecated]
// struct DeprecatedType {
//     a: i32,
// }

// #[derive(Type)]
// #[specta(export = false)]
// #[deprecated = "Look at you big man using a deprecation message"]
// struct DeprecatedTypeWithMsg {
//     a: i32,
// }

// #[derive(Type)]
// #[specta(export = false)]
// #[deprecated(note = "Look at you big man using a deprecation message")]
// struct DeprecatedTypeWithMsg2 {
//     a: i32,
// }

// #[derive(Type)]
// #[specta(export = false)]
// struct DeprecatedFields {
//     a: i32,
//     // #[deprecated]
//     b: String,
//     #[deprecated = "This field is cringe!"]
//     c: String,
//     #[deprecated(note = "This field is cringe!")]
//     d: String,
// }

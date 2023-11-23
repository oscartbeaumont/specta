use crate::{reference::Reference, *};

use std::borrow::Cow;

impl_primitives!(
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
    f32 f64
    bool char
    String
);

impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13); // Technically we only support 12-tuples but the `T13` is required due to how the macro works

const _: () = {
    use std::{cell::*, rc::Rc, sync::*};
    impl_containers!(Box Rc Arc Cell RefCell Mutex RwLock);
};

#[cfg(feature = "tokio")]
const _: () = {
    use tokio::sync::{Mutex, RwLock};
    impl_containers!(Mutex RwLock);
};

impl<'a> Type for &'a str {
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        String::inline(opts, generics)
    }
}

impl<'a, T: Type + 'static> Type for &'a T {
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        T::inline(opts, generics)
    }
}

impl<T: Type> Type for [T] {
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        T::inline(opts, generics)
    }
}

impl<'a, T: ?Sized + ToOwned + Type + 'static> Type for std::borrow::Cow<'a, T> {
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        T::inline(opts, generics)
    }
}

use std::ffi::*;
impl_as!(
    str as String
    CString as String
    CStr as String
    OsString as String
    OsStr as String
);

use std::path::*;
impl_as!(
    Path as String
    PathBuf as String
);

use std::net::*;
impl_as!(
    IpAddr as String
    Ipv4Addr as String
    Ipv6Addr as String

    SocketAddr as String
    SocketAddrV4 as String
    SocketAddrV6 as String
);

use std::sync::atomic::*;
impl_as!(
    AtomicBool as bool
    AtomicI8 as i8
    AtomicI16 as i16
    AtomicI32 as i32
    AtomicIsize as isize
    AtomicU8 as u8
    AtomicU16 as u16
    AtomicU32 as u32
    AtomicUsize as usize
    AtomicI64 as i64
    AtomicU64 as u64
);

use std::num::*;
impl_as!(
    NonZeroU8 as u8
    NonZeroU16 as u16
    NonZeroU32 as u32
    NonZeroU64 as u64
    NonZeroUsize as usize
    NonZeroI8 as i8
    NonZeroI16 as i16
    NonZeroI32 as i32
    NonZeroI64 as i64
    NonZeroIsize as isize
    NonZeroU128 as u128
    NonZeroI128 as i128
);

use std::collections::*;
impl_for_list!(
    Vec<T> as "Vec"
    VecDeque<T> as "VecDeque"
    BinaryHeap<T> as "BinaryHeap"
    LinkedList<T> as "LinkedList"
    HashSet<T> as "HashSet"
    BTreeSet<T> as "BTreeSet"
);

impl<'a, T: Type> Type for &'a [T] {
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        <Vec<T>>::inline(opts, generics)
    }
}

impl<const N: usize, T: Type> Type for [T; N] {
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        DataType::List(List {
            ty: Box::new(
                // TODO: This is cursed. Fix it properly!!!
                match Vec::<T>::inline(
                    DefOpts {
                        parent_inline: opts.parent_inline,
                        type_map: opts.type_map,
                    },
                    generics,
                ) {
                    DataType::List(List { ty, .. }) => *ty,
                    _ => unreachable!(),
                },
            ),
            length: Some(N),
        })
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> Reference {
        Reference {
            inner: DataType::List(List {
                ty: Box::new(
                    // TODO: This is cursed. Fix it properly!!!
                    match Vec::<T>::reference(
                        DefOpts {
                            parent_inline: opts.parent_inline,
                            type_map: opts.type_map,
                        },
                        generics,
                    )
                    .inner
                    {
                        DataType::List(List { ty, .. }) => *ty,
                        _ => unreachable!(),
                    },
                ),
                length: Some(N),
            }),
        }
    }
}

impl<T: Type> Type for Option<T> {
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        DataType::Nullable(Box::new(
            generics
                .get(0)
                .cloned()
                .unwrap_or_else(|| T::inline(opts, generics)),
        ))
    }
}

impl<T: Type, E: Type> Type for std::result::Result<T, E> {
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        DataType::Result(Box::new((
            T::inline(
                DefOpts {
                    parent_inline: opts.parent_inline,
                    type_map: opts.type_map,
                },
                generics,
            ),
            E::inline(
                DefOpts {
                    parent_inline: opts.parent_inline,
                    type_map: opts.type_map,
                },
                generics,
            ),
        )))
    }
}

impl<T> Type for std::marker::PhantomData<T> {
    fn inline(_: DefOpts, _: &[DataType]) -> DataType {
        DataType::Literal(LiteralType::None)
    }
}

// Serde does no support `Infallible` as it can't be constructed so a `&self` method is uncallable on it.
#[allow(unused)]
#[derive(Type)]
#[specta(remote = std::convert::Infallible, crate = crate, export = false)]
pub enum Infallible {}

impl<T: Type> Type for std::ops::Range<T> {
    fn inline(opts: DefOpts, _generics: &[DataType]) -> DataType {
        let ty = Some(T::definition(opts));
        DataType::Struct(StructType {
            name: "Range".into(),
            generics: vec![],
            fields: StructFields::Named(NamedFields {
                fields: vec![
                    (
                        "start".into(),
                        Field {
                            optional: false,
                            flatten: false,
                            deprecated: None,
                            docs: Cow::Borrowed(""),
                            ty: ty.clone(),
                        },
                    ),
                    (
                        "end".into(),
                        Field {
                            optional: false,
                            flatten: false,
                            deprecated: None,
                            docs: Cow::Borrowed(""),
                            ty,
                        },
                    ),
                ],
                tag: None,
            }),
        })
    }
}

impl<T: Type> Type for std::ops::RangeInclusive<T> {
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        std::ops::Range::<T>::inline(opts, generics) // Yeah Serde are cringe
    }
}

impl_for_map!(HashMap<K, V> as "HashMap");
impl_for_map!(BTreeMap<K, V> as "BTreeMap");
impl<K: Type, V: Type> Flatten for std::collections::HashMap<K, V> {}
impl<K: Type, V: Type> Flatten for std::collections::BTreeMap<K, V> {}

#[derive(Type)]
#[specta(remote = std::time::SystemTime, crate = crate, export = false)]
#[allow(dead_code)]
struct SystemTime {
    duration_since_epoch: i64,
    duration_since_unix_epoch: u32,
}

#[derive(Type)]
#[specta(remote = std::time::Duration, crate = crate, export = false)]
#[allow(dead_code)]
struct Duration {
    secs: u64,
    nanos: u32,
}

#[cfg(feature = "indexmap")]
const _: () = {
    impl_for_list!(indexmap::IndexSet<T> as "IndexSet");
    impl_for_map!(indexmap::IndexMap<K, V> as "IndexMap");
    impl<K: Type, V: Type> Flatten for indexmap::IndexMap<K, V> {}
};

#[cfg(feature = "serde_json")]
const _: () = {
    use serde_json::{Map, Number, Value};

    impl_for_map!(Map<K, V> as "Map");
    impl<K: Type, V: Type> Flatten for Map<K, V> {}

    #[derive(Type)]
    #[specta(rename = "JsonValue", untagged, remote = Value, crate = crate, export = false)]
    pub enum JsonValue {
        Null,
        Bool(bool),
        Number(Number),
        String(String),
        Array(Vec<Value>),
        Object(Map<String, Value>),
    }

    impl Type for Number {
        fn inline(_: DefOpts, _: &[DataType]) -> DataType {
            DataType::Enum(EnumType {
                name: "Number".into(),
                repr: EnumRepr::Untagged,
                skip_bigint_checks: true,
                variants: vec![
                    (
                        "f64".into(),
                        EnumVariant {
                            skip: false,
                            docs: Cow::Borrowed(""),
                            deprecated: None,
                            inner: EnumVariants::Unnamed(UnnamedFields {
                                fields: vec![Field {
                                    optional: false,
                                    flatten: false,
                                    deprecated: None,
                                    docs: Cow::Borrowed(""),
                                    ty: Some(DataType::Primitive(PrimitiveType::f64)),
                                }],
                            }),
                        },
                    ),
                    (
                        "i64".into(),
                        EnumVariant {
                            skip: false,
                            docs: Cow::Borrowed(""),
                            deprecated: None,
                            inner: EnumVariants::Unnamed(UnnamedFields {
                                fields: vec![Field {
                                    optional: false,
                                    flatten: false,
                                    deprecated: None,
                                    docs: Cow::Borrowed(""),
                                    ty: Some(DataType::Primitive(PrimitiveType::i64)),
                                }],
                            }),
                        },
                    ),
                    (
                        "u64".into(),
                        EnumVariant {
                            skip: false,
                            docs: Cow::Borrowed(""),
                            deprecated: None,
                            inner: EnumVariants::Unnamed(UnnamedFields {
                                fields: vec![Field {
                                    optional: false,
                                    flatten: false,
                                    deprecated: None,
                                    docs: Cow::Borrowed(""),
                                    ty: Some(DataType::Primitive(PrimitiveType::u64)),
                                }],
                            }),
                        },
                    ),
                ],
                generics: vec![],
            })
        }
    }
};

#[cfg(feature = "serde_yaml")]
const _: () = {
    use serde_yaml::{value::TaggedValue, Mapping, Number, Sequence, Value};

    #[derive(Type)]
    #[specta(rename = "YamlValue", untagged, remote = Value, crate = crate, export = false)]
    pub enum YamlValue {
        Null,
        Bool(bool),
        Number(Number),
        String(String),
        Sequence(Sequence),
        Mapping(Mapping),
        Tagged(Box<TaggedValue>),
    }

    impl Type for serde_yaml::Mapping {
        fn inline(_: DefOpts, _: &[DataType]) -> DataType {
            // We don't type this more accurately because `serde_json` doesn't allow non-string map keys so neither does Specta
            DataType::Unknown
        }
    }

    impl Type for serde_yaml::value::TaggedValue {
        fn inline(_: DefOpts, _: &[DataType]) -> DataType {
            DataType::Map(Box::new((
                DataType::Primitive(PrimitiveType::String),
                DataType::Unknown,
            )))
        }
    }

    impl Type for serde_yaml::Number {
        fn inline(_: DefOpts, _: &[DataType]) -> DataType {
            DataType::Enum(EnumType {
                name: "Number".into(),
                repr: EnumRepr::Untagged,
                skip_bigint_checks: true,
                variants: vec![
                    (
                        "f64".into(),
                        EnumVariant {
                            skip: false,
                            docs: Cow::Borrowed(""),
                            deprecated: None,
                            inner: EnumVariants::Unnamed(UnnamedFields {
                                fields: vec![Field {
                                    optional: false,
                                    flatten: false,
                                    deprecated: None,
                                    docs: Cow::Borrowed(""),
                                    ty: Some(DataType::Primitive(PrimitiveType::f64)),
                                }],
                            }),
                        },
                    ),
                    (
                        "i64".into(),
                        EnumVariant {
                            skip: false,
                            docs: Cow::Borrowed(""),
                            deprecated: None,
                            inner: EnumVariants::Unnamed(UnnamedFields {
                                fields: vec![Field {
                                    optional: false,
                                    flatten: false,
                                    deprecated: None,
                                    docs: Cow::Borrowed(""),
                                    ty: Some(DataType::Primitive(PrimitiveType::i64)),
                                }],
                            }),
                        },
                    ),
                    (
                        "u64".into(),
                        EnumVariant {
                            skip: false,
                            docs: Cow::Borrowed(""),
                            deprecated: None,
                            inner: EnumVariants::Unnamed(UnnamedFields {
                                fields: vec![Field {
                                    optional: false,
                                    flatten: false,
                                    deprecated: None,
                                    docs: Cow::Borrowed(""),
                                    ty: Some(DataType::Primitive(PrimitiveType::u64)),
                                }],
                            }),
                        },
                    ),
                ],
                generics: vec![],
            })
        }
    }
};

#[cfg(feature = "toml")]
const _: () = {
    use toml::{value::Array, value::Datetime, value::Table, Value};

    impl_for_map!(toml::map::Map<K, V> as "Map");
    impl<K: Type, V: Type> Flatten for toml::map::Map<K, V> {}

    #[derive(Type)]
    #[specta(rename = "TomlValue", untagged, remote = Value, crate = crate, export = false, unstable_skip_bigint_checks)]
    pub enum TomlValue {
        String(String),
        Integer(i64),
        Float(f64),
        Boolean(bool),
        Datetime(Datetime),
        Array(Array),
        Table(Table),
    }

    #[derive(Type)]
    #[specta(rename = "Datetime", remote = Datetime, crate = crate, export = false)]
    #[allow(dead_code)]
    struct DatetimeDef {
        #[specta(rename = "$__toml_private_datetime")]
        pub v: String,
    }
};

#[cfg(feature = "ulid")]
impl_as!(ulid::Ulid as String);

#[cfg(feature = "uuid")]
impl_as!(
    uuid::Uuid as String
    uuid::fmt::Hyphenated as String
);

#[cfg(feature = "chrono")]
const _: () = {
    use chrono::*;

    impl_as!(
        NaiveDateTime as String
        NaiveDate as String
        NaiveTime as String
        chrono::Duration as String
    );

    impl<T: TimeZone> Type for DateTime<T> {
        fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
            String::inline(opts, generics)
        }
    }

    #[allow(deprecated)]
    impl<T: TimeZone> Type for Date<T> {
        fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
            String::inline(opts, generics)
        }
    }
};

#[cfg(feature = "time")]
impl_as!(
    time::PrimitiveDateTime as String
    time::OffsetDateTime as String
    time::Date as String
    time::Time as String
);

#[cfg(feature = "bigdecimal")]
impl_as!(bigdecimal::BigDecimal as String);

// This assumes the `serde-with-str` feature is enabled. Check #26 for more info.
#[cfg(feature = "rust_decimal")]
impl_as!(rust_decimal::Decimal as String);

#[cfg(feature = "ipnetwork")]
impl_as!(
    ipnetwork::IpNetwork as String
    ipnetwork::Ipv4Network as String
    ipnetwork::Ipv6Network as String
);

#[cfg(feature = "mac_address")]
impl_as!(mac_address::MacAddress as String);

#[cfg(feature = "chrono")]
impl_as!(
    chrono::FixedOffset as String
    chrono::Utc as String
    chrono::Local as String
);

#[cfg(feature = "bson")]
impl_as!(
    bson::oid::ObjectId as String
    bson::Decimal128 as i128
    bson::DateTime as String
    bson::Uuid as String
);

// TODO: bson::bson
// TODO: bson::Document

#[cfg(feature = "bytesize")]
impl_as!(bytesize::ByteSize as u64);

#[cfg(feature = "uhlc")]
const _: () = {
    use uhlc::*;

    impl_as!(
        NTP64 as u64
        ID as NonZeroU128
    );

    #[derive(Type)]
    #[specta(remote = Timestamp, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Timestamp {
        time: NTP64,
        id: ID,
    }
};

#[cfg(feature = "glam")]
const _: () = {
    #[derive(Type)]
    #[specta(remote = glam::DVec2, crate = crate, export = false)]
    #[allow(dead_code)]
    struct DVec2([f64; 2]);

    #[derive(Type)]
    #[specta(remote = glam::IVec2, crate = crate, export = false)]
    #[allow(dead_code)]
    struct IVec2([i32; 2]);

    #[derive(Type)]
    #[specta(remote = glam::DMat2, crate = crate, export = false)]
    #[allow(dead_code)]
    struct DMat2([f64; 4]);

    #[derive(Type)]
    #[specta(remote = glam::DAffine2, crate = crate, export = false)]
    #[allow(dead_code)]
    struct DAffine2([f64; 6]);

    #[derive(Type)]
    #[specta(remote = glam::Vec2, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Vec2([f32; 2]);

    #[derive(Type)]
    #[specta(remote = glam::Vec3, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Vec3([f32; 3]);

    #[derive(Type)]
    #[specta(remote = glam::Vec3A, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Vec3A([f32; 3]);

    #[derive(Type)]
    #[specta(remote = glam::Vec4, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Vec4([f32; 4]);

    #[derive(Type)]
    #[specta(remote = glam::Mat2, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Mat2([f32; 4]);

    #[derive(Type)]
    #[specta(remote = glam::Mat3, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Mat3([f32; 9]);

    #[derive(Type)]
    #[specta(remote = glam::Mat3A, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Mat3A([f32; 9]);

    #[derive(Type)]
    #[specta(remote = glam::Mat4, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Mat4([f32; 16]);

    #[derive(Type)]
    #[specta(remote = glam::Quat, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Quat([f32; 4]);

    #[derive(Type)]
    #[specta(remote = glam::Affine2, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Affine2([f32; 6]);

    #[derive(Type)]
    #[specta(remote = glam::Affine3A, crate = crate, export = false)]
    #[allow(dead_code)]
    struct Affine3A([f32; 12]);
};

#[cfg(feature = "url")]
impl_as!(url::Url as String);

#[cfg(feature = "either")]
impl<L: Type, R: Type> Type for either::Either<L, R> {
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        DataType::Enum(EnumType {
            name: "Either".into(),
            repr: EnumRepr::Untagged,
            skip_bigint_checks: false,
            variants: vec![
                (
                    "Left".into(),
                    EnumVariant {
                        skip: false,
                        docs: Cow::Borrowed(""),
                        deprecated: None,
                        inner: EnumVariants::Unnamed(UnnamedFields {
                            fields: vec![Field {
                                optional: false,
                                flatten: false,
                                deprecated: None,
                                docs: Cow::Borrowed(""),
                                ty: Some(L::inline(
                                    DefOpts {
                                        parent_inline: opts.parent_inline,
                                        type_map: opts.type_map,
                                    },
                                    generics,
                                )),
                            }],
                        }),
                    },
                ),
                (
                    "Right".into(),
                    EnumVariant {
                        skip: false,
                        docs: Cow::Borrowed(""),
                        deprecated: None,
                        inner: EnumVariants::Unnamed(UnnamedFields {
                            fields: vec![Field {
                                optional: false,
                                flatten: false,
                                deprecated: None,
                                docs: Cow::Borrowed(""),
                                ty: Some(R::inline(
                                    DefOpts {
                                        parent_inline: opts.parent_inline,
                                        type_map: opts.type_map,
                                    },
                                    generics,
                                )),
                            }],
                        }),
                    },
                ),
            ],
            generics: vec![],
        })
    }
}

#[cfg(feature = "bevy_ecs")]
const _: () = {
    #[derive(Type)]
    #[specta(rename = "bevy_ecs::entity::Entity", remote = bevy_ecs::entity::Entity, crate = crate, export = false)]
    #[allow(dead_code)]
    struct EntityDef(u64);
};

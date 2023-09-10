use std::borrow::Cow;

mod comments;
mod context;
mod error;
mod export_config;
mod formatter;
mod reserved_terms;

pub use comments::*;
pub use context::*;
pub use error::*;
pub use export_config::*;
pub use formatter::*;
use reserved_terms::*;

use crate::{
    advanced::{detect_duplicate_type_names, is_valid_ty},
    *,
};

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, TsExportError>;

type Output = Result<String>;

/// Convert a type which implements [`Type`](crate::Type) to a TypeScript string with an export.
///
/// Eg. `export type Foo = { demo: string; };`
pub fn export_ref<T: NamedType>(_: &T, conf: &ExportConfig) -> Output {
    export::<T>(conf)
}

/// Convert a type which implements [`Type`](crate::Type) to a TypeScript string with an export.
///
/// Eg. `export type Foo = { demo: string; };`
pub fn export<T: NamedType>(conf: &ExportConfig) -> Output {
    let mut type_map = TypeMap::default();
    let named_data_type = T::definition_named_data_type(DefOpts {
        parent_inline: false,
        type_map: &mut type_map,
    });
    is_valid_ty(&named_data_type.inner, &type_map)?;
    let result = export_named_datatype(conf, &named_data_type, &type_map);

    if let Some((ty_name, l0, l1)) = detect_duplicate_type_names(&type_map).into_iter().next() {
        return Err(TsExportError::DuplicateTypeName(ty_name, l0, l1));
    }

    result
}

/// Convert a type which implements [`Type`](crate::Type) to a TypeScript string.
///
/// Eg. `{ demo: string; };`
pub fn inline_ref<T: Type>(_: &T, conf: &ExportConfig) -> Output {
    inline::<T>(conf)
}

/// Convert a type which implements [`Type`](crate::Type) to a TypeScript string.
///
/// Eg. `{ demo: string; };`
pub fn inline<T: Type>(conf: &ExportConfig) -> Output {
    let mut type_map = TypeMap::default();
    let ty = T::inline(
        DefOpts {
            parent_inline: false,
            type_map: &mut type_map,
        },
        &[],
    );
    is_valid_ty(&ty, &type_map)?;
    let result = datatype(conf, &ty, &type_map);

    if let Some((ty_name, l0, l1)) = detect_duplicate_type_names(&type_map).into_iter().next() {
        return Err(TsExportError::DuplicateTypeName(ty_name, l0, l1));
    }

    result
}

/// Convert a DataType to a TypeScript string
///
/// Eg. `export Name = { demo: string; }`
pub fn export_named_datatype(
    conf: &ExportConfig,
    typ: &NamedDataType,
    type_map: &TypeMap,
) -> Output {
    // TODO: Duplicate type name detection?

    is_valid_ty(&typ.inner, type_map)?;
    export_datatype_inner(ExportContext { conf, path: vec![] }, typ, type_map)
}

fn export_datatype_inner(
    ctx: ExportContext,
    typ @ NamedDataType {
        name,
        comments,
        inner: item,
        ..
    }: &NamedDataType,
    type_map: &TypeMap,
) -> Output {
    let ctx = ctx.with(PathItem::Type(name.clone()));
    let name = sanitise_type_name(ctx.clone(), NamedLocation::Type, name)?;

    let inline_ts = datatype_inner(ctx.clone(), &typ.inner, type_map, "null")?;

    let generics = item
        .generics()
        .filter(|generics| !generics.is_empty())
        .map(|generics| format!("<{}>", generics.join(", ")))
        .unwrap_or_default();

    let comments = ctx
        .conf
        .comment_exporter
        .map(|v| v(comments))
        .unwrap_or_default();

    Ok(format!(
        "{comments}export type {name}{generics} = {inline_ts}"
    ))
}

/// Convert a DataType to a TypeScript string
///
/// Eg. `{ demo: string; }`
pub fn datatype(conf: &ExportConfig, typ: &DataType, type_map: &TypeMap) -> Output {
    // TODO: Duplicate type name detection?

    datatype_inner(ExportContext { conf, path: vec![] }, typ, type_map, "null")
}

fn datatype_inner(
    ctx: ExportContext,
    typ: &DataType,
    type_map: &TypeMap,
    empty_tuple_fallback: &'static str,
) -> Output {
    Ok(match &typ {
        DataType::Any => "any".into(),
        DataType::Primitive(p) => {
            let ctx = ctx.with(PathItem::Type(p.to_rust_str().into()));
            match p {
                primitive_def!(i8 i16 i32 u8 u16 u32 f32 f64) => "number".into(),
                primitive_def!(usize isize i64 u64 i128 u128) => match ctx.conf.bigint {
                    BigIntExportBehavior::String => "string".into(),
                    BigIntExportBehavior::Number => "number".into(),
                    BigIntExportBehavior::BigInt => "BigInt".into(),
                    BigIntExportBehavior::Fail => {
                        return Err(TsExportError::BigIntForbidden(ctx.export_path()))
                    }
                    BigIntExportBehavior::FailWithReason(reason) => {
                        return Err(TsExportError::Other(ctx.export_path(), reason.to_owned()))
                    }
                },
                primitive_def!(String char) => "string".into(),
                primitive_def!(bool) => "boolean".into(),
            }
        }
        DataType::Literal(literal) => literal.to_ts(),
        DataType::Nullable(def) => {
            let dt = datatype_inner(ctx, def, type_map, "null")?;

            if dt.ends_with(" | null") {
                dt
            } else {
                format!("{dt} | null")
            }
        }
        DataType::Map(def) => {
            format!(
                // We use this isn't of `Record<K, V>` to avoid issues with circular references.
                "{{ [key in {}]: {} }}",
                datatype_inner(ctx.clone(), &def.0, type_map, "null")?,
                datatype_inner(ctx, &def.1, type_map, "null")?
            )
        }
        // We use `T[]` instead of `Array<T>` to avoid issues with circular references.
        DataType::List(def) => {
            let dt = datatype_inner(ctx, def, type_map, "null")?;
            if dt.contains(' ') && !dt.ends_with('}') {
                format!("({dt})[]")
            } else {
                format!("{dt}[]")
            }
        }
        DataType::Struct(item) => struct_datatype(
            ctx.with(PathItem::Type(item.name().clone())),
            item.name(),
            item,
            type_map,
        )?,
        DataType::Enum(item) => enum_datatype(
            ctx.with(PathItem::Variant(item.name.clone())),
            item,
            type_map,
        )?,
        DataType::Tuple(tuple) => tuple_datatype(ctx, tuple, type_map, empty_tuple_fallback)?,
        DataType::Result(result) => {
            let mut variants = vec![
                datatype_inner(ctx.clone(), &result.0, type_map, "null")?,
                datatype_inner(ctx, &result.1, type_map, "null")?,
            ];
            variants.dedup();
            variants.join(" | ")
        }
        DataType::Reference(DataTypeReference { name, generics, .. }) => match &generics[..] {
            [] => name.to_string(),
            generics => {
                let generics = generics
                    .iter()
                    .map(|v| {
                        datatype_inner(
                            ctx.with(PathItem::Type(name.clone())),
                            v,
                            type_map,
                            empty_tuple_fallback,
                        )
                    })
                    .collect::<Result<Vec<_>>>()?
                    .join(", ");

                format!("{name}<{generics}>")
            }
        },
        DataType::Generic(GenericType(ident)) => ident.to_string(),
    })
}

// Can be used with `StructUnnamedFields.fields` or `EnumNamedFields.fields`
fn unnamed_fields_datatype(
    ctx: ExportContext,
    fields: &[Field],
    type_map: &TypeMap,
    empty_tuple_fallback: &'static str,
) -> Output {
    match fields {
        [] => Ok(empty_tuple_fallback.to_string()),
        [field] => datatype_inner(ctx, &field.ty, type_map, "null"),
        fields => Ok(format!(
            "[{}]",
            fields
                .iter()
                .map(|field| datatype_inner(ctx.clone(), &field.ty, type_map, "null"))
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        )),
    }
}

fn tuple_datatype(
    ctx: ExportContext,
    tuple: &TupleType,
    type_map: &TypeMap,
    empty_tuple_fallback: &'static str,
) -> Output {
    match &tuple.fields[..] {
        [] => Ok(empty_tuple_fallback.to_string()),
        [ty] => datatype_inner(ctx, ty, type_map, "null"),
        tys => Ok(format!(
            "[{}]",
            tys.iter()
                .map(|v| datatype_inner(ctx.clone(), v, type_map, "null"))
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        )),
    }
}

fn struct_datatype(ctx: ExportContext, key: &str, s: &StructType, type_map: &TypeMap) -> Output {
    match &s.fields {
        StructFields::Unit => Ok("null".into()),
        StructFields::Unnamed(s) => unnamed_fields_datatype(ctx, &s.fields, type_map, "[]"),
        StructFields::Named(s) => {
            if s.fields.is_empty() {
                return Ok("Record<string, never>".into());
            }

            let (flattened, non_flattened): (Vec<_>, Vec<_>) =
                s.fields.iter().partition(|(_, f)| f.flatten);

            let mut field_sections = flattened
                .into_iter()
                .map(|(key, field)| {
                    datatype_inner(
                        ctx.with(PathItem::Field(key.clone())),
                        &field.ty,
                        type_map,
                        "[]",
                    )
                    .map(|type_str| format!("({type_str})"))
                })
                .collect::<Result<Vec<_>>>()?;

            let mut unflattened_fields = non_flattened
                .into_iter()
                .map(|(key, f)| {
                    object_field_to_ts(
                        ctx.with(PathItem::Field(key.clone())),
                        key.clone(),
                        f,
                        type_map,
                    )
                })
                .collect::<Result<Vec<_>>>()?;

            if let Some(tag) = &s.tag {
                unflattened_fields.push(format!("{tag}: \"{key}\""));
            }

            if !unflattened_fields.is_empty() {
                field_sections.push(format!("{{ {} }}", unflattened_fields.join("; ")));
            }

            Ok(field_sections.join(" & "))
        }
    }
}

fn enum_variant_datatype(
    ctx: ExportContext,
    type_map: &TypeMap,
    name: Cow<'static, str>,
    variant: &EnumVariant,
) -> Output {
    match variant {
        // TODO: Remove unreachable in type system
        EnumVariant::Unit => unreachable!("Unit enum variants have no type!"),
        EnumVariant::Named(obj) => {
            let mut fields = if let Some(tag) = &obj.tag {
                let sanitised_name = sanitise_key(name, true);
                vec![format!("{tag}: {sanitised_name}")]
            } else {
                vec![]
            };

            fields.extend(
                obj.fields
                    .iter()
                    .map(|(name, field)| {
                        object_field_to_ts(
                            ctx.with(PathItem::Field(name.clone())),
                            name.clone(),
                            field,
                            type_map,
                        )
                    })
                    .collect::<Result<Vec<_>>>()?,
            );

            Ok(match &fields[..] {
                [] => "Record<string, never>".to_string(),
                fields => format!("{{ {} }}", fields.join("; ")),
            })
        }
        EnumVariant::Unnamed(obj) => {
            let fields = obj
                .fields
                .iter()
                .map(|field| datatype_inner(ctx.clone(), &field.ty, type_map, "[]"))
                .collect::<Result<Vec<_>>>()?;

            Ok(match &fields[..] {
                [] => "[]".to_string(),
                [field] => field.to_string(),
                fields => format!("[{}]", fields.join(", ")),
            })
        }
    }
}

fn enum_datatype(ctx: ExportContext, e: &EnumType, type_map: &TypeMap) -> Output {
    if e.variants().is_empty() {
        return Ok("never".to_string());
    }

    Ok(match &e.repr {
        EnumRepr::Untagged => {
            let mut variants = e
                .variants
                .iter()
                .map(|(name, variant)| {
                    Ok(match variant {
                        EnumVariant::Unit => "null".to_string(),
                        v => enum_variant_datatype(
                            ctx.with(PathItem::Variant(name.clone())),
                            type_map,
                            name.clone(),
                            v,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            variants.dedup();
            variants.join(" | ")
        }
        repr => {
            let mut variants = e
                .variants
                .iter()
                .map(|(variant_name, variant)| {
                    let sanitised_name = sanitise_key(variant_name.clone(), true);

                    Ok(match (repr, variant) {
                        (EnumRepr::Untagged, _) => unreachable!(),
                        (EnumRepr::Internal { tag }, EnumVariant::Unit) => {
                            format!("{{ {tag}: {sanitised_name} }}")
                        }
                        (EnumRepr::Internal { tag }, EnumVariant::Unnamed(tuple)) => {
                            let mut typ = unnamed_fields_datatype(
                                ctx.clone(),
                                &tuple.fields,
                                type_map,
                                "[]",
                            )?;

                            // TODO: This `null` check is a bad fix for an internally tagged type with a `null` variant being exported as `{ type: "A" } & null` (which is `never` in TS)
                            // TODO: Move this check into the macros so it can apply to any language cause it should (it's just hard to do in the macros)
                            if typ == "null" {
                                format!("({{ {tag}: {sanitised_name} }})")
                            } else {
                                // We wanna be sure `... & ... | ...` becomes `... & (... | ...)`
                                if typ.contains('|') {
                                    typ = format!("({typ})");
                                }
                                format!("({{ {tag}: {sanitised_name} }} & {typ})")
                            }
                        }
                        (EnumRepr::Internal { tag }, EnumVariant::Named(obj)) => {
                            let mut fields = vec![format!("{tag}: {sanitised_name}")];

                            fields.extend(
                                obj.fields
                                    .iter()
                                    .map(|(name, field)| {
                                        object_field_to_ts(
                                            ctx.with(PathItem::Field(name.clone())),
                                            name.clone(),
                                            field,
                                            type_map,
                                        )
                                    })
                                    .collect::<Result<Vec<_>>>()?,
                            );

                            format!("{{ {} }}", fields.join("; "))
                        }
                        (EnumRepr::External, EnumVariant::Unit) => sanitised_name.to_string(),

                        (EnumRepr::External, v) => {
                            let ts_values = enum_variant_datatype(
                                ctx.with(PathItem::Variant(variant_name.clone())),
                                type_map,
                                variant_name.clone(),
                                v,
                            )?;
                            let sanitised_name = sanitise_key(variant_name.clone(), false);

                            format!("{{ {sanitised_name}: {ts_values} }}")
                        }
                        (EnumRepr::Adjacent { tag, .. }, EnumVariant::Unit) => {
                            format!("{{ {tag}: {sanitised_name} }}")
                        }
                        (EnumRepr::Adjacent { tag, content }, v) => {
                            let ts_values = enum_variant_datatype(
                                ctx.with(PathItem::Variant(variant_name.clone())),
                                type_map,
                                variant_name.clone(),
                                v,
                            )?;

                            format!("{{ {tag}: {sanitised_name}; {content}: {ts_values} }}")
                        }
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            variants.dedup();
            variants.join(" | ")
        }
    })
}

impl LiteralType {
    fn to_ts(&self) -> String {
        match self {
            Self::i8(v) => v.to_string(),
            Self::i16(v) => v.to_string(),
            Self::i32(v) => v.to_string(),
            Self::u8(v) => v.to_string(),
            Self::u16(v) => v.to_string(),
            Self::u32(v) => v.to_string(),
            Self::f32(v) => v.to_string(),
            Self::f64(v) => v.to_string(),
            Self::bool(v) => v.to_string(),
            Self::String(v) => format!(r#""{v}""#),
            Self::char(v) => format!(r#""{v}""#),
            Self::None => "null".to_string(),
        }
    }
}

/// convert an object field into a Typescript string
fn object_field_to_ts(
    ctx: ExportContext,
    key: Cow<'static, str>,
    field: &Field,
    type_map: &TypeMap,
) -> Output {
    let field_name_safe = sanitise_key(key, false);

    // https://github.com/oscartbeaumont/rspc/issues/100#issuecomment-1373092211
    let (key, ty) = match field.optional {
        true => (format!("{field_name_safe}?").into(), &field.ty),
        false => (field_name_safe, &field.ty),
    };

    Ok(format!(
        "{key}: {}",
        datatype_inner(ctx, ty, type_map, "null")?
    ))
}

/// sanitise a string to be a valid Typescript key
fn sanitise_key<'a>(field_name: Cow<'static, str>, force_string: bool) -> Cow<'a, str> {
    let valid = field_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        && field_name
            .chars()
            .next()
            .map(|first| !first.is_numeric())
            .unwrap_or(true);

    if force_string || !valid {
        format!(r#""{field_name}""#).into()
    } else {
        field_name
    }
}

fn sanitise_type_name(ctx: ExportContext, loc: NamedLocation, ident: &str) -> Output {
    if let Some(name) = RESERVED_TYPE_NAMES.iter().find(|v| **v == ident) {
        return Err(TsExportError::ForbiddenName(loc, ctx.export_path(), name));
    }

    Ok(ident.to_string())
}

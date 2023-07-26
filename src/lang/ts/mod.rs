mod comments;
mod context;
mod error;
mod export_config;

pub use comments::*;
pub use context::*;
pub use error::*;
pub use export_config::*;

use crate::*;

/// Convert a type which implements [`Type`](crate::Type) to a TypeScript string with an export.
///
/// Eg. `export type Foo = { demo: string; };`
pub fn export<T: NamedType>(conf: &ExportConfiguration) -> Result<String, TsExportError> {
    let mut type_name = TypeDefs::default();
    let result = export_datatype(
        conf,
        &T::definition_named_data_type(DefOpts {
            parent_inline: false,
            type_map: &mut type_name,
        })?,
    );

    if conf.modules == ModuleExportBehavior::Disabled {
        if let Some((ty_name, l0, l1)) = detect_duplicate_type_names(&type_name).into_iter().next()
        {
            return Err(TsExportError::DuplicateTypeName(ty_name, l0, l1));
        }
    }

    result
}

/// Convert a type which implements [`Type`](crate::Type) to a TypeScript string.
///
/// Eg. `{ demo: string; };`
pub fn inline<T: Type>(conf: &ExportConfiguration) -> Result<String, TsExportError> {
    let mut type_name = TypeDefs::default();
    let result = datatype(
        conf,
        &T::inline(
            DefOpts {
                parent_inline: false,
                type_map: &mut type_name,
            },
            &[],
        )?,
    );

    if conf.modules == ModuleExportBehavior::Disabled {
        if let Some((ty_name, l0, l1)) = detect_duplicate_type_names(&type_name).into_iter().next()
        {
            return Err(TsExportError::DuplicateTypeName(ty_name, l0, l1));
        }
    }

    result
}

/// Convert a DataType to a TypeScript string
///
/// Eg. `export Name = { demo: string; }`
pub fn export_datatype(
    conf: &ExportConfiguration,
    typ: &NamedDataType,
) -> Result<String, TsExportError> {
    // TODO: Duplicate type name detection?

    export_datatype_inner(ExportContext { conf, path: vec![] }, typ)
}

fn export_datatype_inner(
    ctx: ExportContext,
    NamedDataType {
        name,
        comments,
        item,
        module_path,
        ..
    }: &NamedDataType,
) -> Result<String, TsExportError> {
    // let out_temp: Vec<String> = module_path.iter().map(|m| m.to_string()).collect();
    // let mut out = out_temp.join("_");
    // out.push_str(name);

    let module_path = String::from(module_path.unwrap_or(""));
    let mut module_path = module_path.replace("::", "_");
    module_path.push('_');
    module_path.push_str(name);

    let ctx = ctx.with(PathItem::Type(name));
    let name = sanitise_type_name(ctx.clone(), NamedLocation::Type, &module_path)?;

    let inline_ts = datatype_inner(
        ctx.clone(),
        &match item {
            NamedDataTypeItem::Object(obj) => DataType::Object(obj.clone()),
            NamedDataTypeItem::Tuple(tuple) => DataType::Tuple(tuple.clone()),
            NamedDataTypeItem::Enum(enum_) => DataType::Enum(enum_.clone()),
        },
    )?;

    let generics = match item {
        // Named struct
        NamedDataTypeItem::Object(ObjectType {
            generics, fields, ..
        }) => match fields.len() {
            0 => Some(generics),
            _ => (!generics.is_empty()).then_some(generics),
        },
        // Enum
        NamedDataTypeItem::Enum(e) => {
            let generics = e.generics();
            (!generics.is_empty()).then_some(generics)
        }
        // Struct with unnamed fields
        NamedDataTypeItem::Tuple(TupleType { generics, .. }) => {
            (!generics.is_empty()).then_some(generics)
        }
    };

    let generics = generics
        .map(|generics| format!("<{}>", generics.to_vec().join(", ")))
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
pub fn datatype(conf: &ExportConfiguration, typ: &DataType) -> Result<String, TsExportError> {
    // TODO: Duplicate type name detection?

    datatype_inner(ExportContext { conf, path: vec![] }, typ)
}

fn datatype_inner(ctx: ExportContext, typ: &DataType) -> Result<String, TsExportError> {
    let result = match &typ {
        DataType::Any => "any".into(),
        DataType::Primitive(p) => {
            let ctx = ctx.with(PathItem::Type(p.to_rust_str()));
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
            let dt = datatype_inner(ctx, def)?;

            if dt.ends_with(" | null") {
                dt
            } else {
                format!("{dt} | null",)
            }
        }
        DataType::Record(def) => {
            let divider = match &def.0 {
                DataType::Enum(_) => " in",
                DataType::Named(dt) => match dt.item {
                    NamedDataTypeItem::Enum(_) => " in",
                    _ => ":",
                },
                _ => ":",
            };

            format!(
                // We use this isn't of `Record<K, V>` to avoid issues with circular references.
                "{{ [key{divider} {}]: {} }}",
                datatype_inner(ctx.clone(), &def.0)?,
                datatype_inner(ctx, &def.1)?
            )
        }
        // We use `T[]` instead of `Array<T>` to avoid issues with circular references.
        DataType::List(def) => {
            let dt = datatype_inner(ctx, def)?;
            if dt.contains(' ') && !dt.ends_with("}") {
                format!("({dt})[]")
            } else {
                format!("{dt}[]")
            }
        }
        DataType::Named(NamedDataType {
            name,
            item: NamedDataTypeItem::Tuple(TupleType { fields, .. }),
            ..
        }) => tuple_datatype(ctx.with(PathItem::Type(name)), fields)?,
        DataType::Tuple(TupleType { fields, .. }) => tuple_datatype(ctx, fields)?,
        DataType::Named(NamedDataType {
            name,
            item: NamedDataTypeItem::Object(item),
            ..
        }) => object_datatype(ctx.with(PathItem::Type(name)), Some(name), item)?,
        DataType::Object(item) => object_datatype(ctx, None, item)?,
        DataType::Named(NamedDataType {
            name,
            item: NamedDataTypeItem::Enum(item),
            ..
        }) => enum_datatype(ctx.with(PathItem::Type(name)), Some(name), item)?,
        DataType::Enum(item) => enum_datatype(ctx, None, item)?,
        DataType::Reference(DataTypeReference {
            name,
            generics,
            module_path,
            ..
        }) => {
            let mut updated_name = module_path.to_string();
            updated_name = updated_name.replace("::", "_");
            updated_name.push('_');
            updated_name.push_str(name);

            match &generics[..] {
                [] => updated_name,
                generics => {
                    let generics = generics
                        .iter()
                        .map(|v| datatype_inner(ctx.with(PathItem::Type(name)), v))
                        .collect::<Result<Vec<_>, _>>()?
                        .join(", ");

                    format!("{updated_name}<{generics}>")
                }
            }
        }
        DataType::Generic(GenericType(ident)) => ident.to_string(),
    };

    Ok(result)
}

fn tuple_datatype(ctx: ExportContext, fields: &[DataType]) -> Result<String, TsExportError> {
    match fields {
        [] => Ok("null".to_string()),
        [ty] => datatype_inner(ctx, ty),
        tys => Ok(format!(
            "[{}]",
            tys.iter()
                .map(|v| datatype_inner(ctx.clone(), v))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ")
        )),
    }
}

fn object_datatype(
    ctx: ExportContext,
    name: Option<&'static str>,
    ObjectType {
        fields,
        tag,
        module_path,
        ..
    }: &ObjectType,
) -> Result<String, TsExportError> {
    match &fields[..] {
        [] => Ok("null".to_string()),
        fields => {
            let mut field_sections = fields
                .iter()
                .filter(|f| f.flatten)
                .map(|field| {
                    datatype_inner(ctx.with(PathItem::Field(field.key)), &field.ty)
                        .map(|type_str| format!("({type_str})"))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let mut unflattened_fields = fields
                .iter()
                .filter(|f| !f.flatten)
                .map(|f| object_field_to_ts(ctx.with(PathItem::Field(f.key)), f))
                .collect::<Result<Vec<_>, _>>()?;

            let module_string = match module_path {
                Some(path) => path.to_string(),
                None => String::new(),
            };

            if let Some(tag) = tag {
                unflattened_fields.push(format!(
                    "{module_string}{tag}: \"{}\"",
                    name.ok_or_else(|| TsExportError::UnableToTagUnnamedType(ctx.export_path()))?
                ));
            }

            if !unflattened_fields.is_empty() {
                field_sections.push(format!("{{ {} }}", unflattened_fields.join("; ")));
            }

            Ok(field_sections.join(" & "))
        }
    }
}

fn enum_datatype(
    ctx: ExportContext,
    _ty_name: Option<&'static str>,
    e: &EnumType,
) -> Result<String, TsExportError> {
    if e.variants_len() == 0 {
        return Ok("never".to_string());
    }

    Ok(match e {
        EnumType::Tagged { variants, repr, .. } => variants
            .iter()
            .map(|(variant_name, variant)| {
                let ctx = ctx.with(PathItem::Variant(variant_name));
                let sanitised_name = sanitise_key(variant_name, true);

                Ok(match (repr, variant) {
                    (EnumRepr::Internal { tag }, EnumVariant::Unit) => {
                        format!("{{ {tag}: {sanitised_name} }}")
                    }
                    (EnumRepr::Internal { tag }, EnumVariant::Unnamed(tuple)) => {
                        let typ = datatype_inner(ctx, &DataType::Tuple(tuple.clone()))?;
                        format!("({{ {tag}: {sanitised_name} }} & {typ})")
                    }
                    (EnumRepr::Internal { tag }, EnumVariant::Named(obj)) => {
                        let mut fields = vec![format!("{tag}: {sanitised_name}")];

                        fields.extend(
                            obj.fields
                                .iter()
                                .map(|v| object_field_to_ts(ctx.with(PathItem::Field(v.key)), v))
                                .collect::<Result<Vec<_>, _>>()?,
                        );

                        format!("{{ {} }}", fields.join("; "))
                    }
                    (EnumRepr::External, EnumVariant::Unit) => {
                        format!("{sanitised_name}")
                    }

                    (EnumRepr::External, v) => {
                        let ts_values = datatype_inner(ctx.clone(), &v.data_type())?;
                        let sanitised_name = sanitise_key(variant_name, false);

                        format!("{{ {sanitised_name}: {ts_values} }}")
                    }
                    (EnumRepr::Adjacent { tag, .. }, EnumVariant::Unit) => {
                        format!("{{ {tag}: {sanitised_name} }}")
                    }
                    (EnumRepr::Adjacent { tag, content }, v) => {
                        let ts_values = datatype_inner(ctx, &v.data_type())?;

                        format!("{{ {tag}: {sanitised_name}; {content}: {ts_values} }}")
                    }
                })
            })
            .collect::<Result<Vec<_>, TsExportError>>()?
            .join(" | "),
        EnumType::Untagged { variants, .. } => variants
            .iter()
            .map(|variant| {
                Ok(match variant {
                    EnumVariant::Unit => "null".to_string(),
                    v => datatype_inner(ctx.clone(), &v.data_type())?,
                })
            })
            .collect::<Result<Vec<_>, TsExportError>>()?
            .join(" | "),
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
            Self::None => "null".to_string(),
        }
    }
}

/// convert an object field into a Typescript string
fn object_field_to_ts(ctx: ExportContext, field: &ObjectField) -> Result<String, TsExportError> {
    let field_name_safe = sanitise_key(field.key, false);

    // https://github.com/oscartbeaumont/rspc/issues/100#issuecomment-1373092211
    let (key, ty) = match field.optional {
        true => (format!("{field_name_safe}?"), &field.ty),
        false => (field_name_safe, &field.ty),
    };

    Ok(format!("{key}: {}", datatype_inner(ctx, ty)?))
}

/// sanitise a string to be a valid Typescript key
fn sanitise_key(field_name: &str, force_string: bool) -> String {
    let valid = field_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        && field_name
            .chars()
            .next()
            .map(|first| !first.is_numeric())
            .unwrap_or(true);

    if force_string || !valid {
        format!(r#""{field_name}""#)
    } else {
        field_name.to_string()
    }
}

fn sanitise_type_name(
    ctx: ExportContext,
    loc: NamedLocation,
    ident: &str,
) -> Result<String, TsExportError> {
    if let Some(name) = RESERVED_TYPE_NAMES.iter().find(|v| **v == ident) {
        return Err(TsExportError::ForbiddenName(loc, ctx.export_path(), name));
    }

    Ok(ident.to_string())
}

/// Taken from: https://github.com/microsoft/TypeScript/blob/fad889283e710ee947e8412e173d2c050107a3c1/src/compiler/types.ts#L276
const RESERVED_TYPE_NAMES: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "as",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "any",
    "boolean",
    "constructor",
    "declare",
    "get",
    "module",
    "require",
    "number",
    "set",
    "string",
    "symbol",
    "type",
    "from",
    "of",
];

/// Taken from: https://github.com/microsoft/TypeScript/blob/fad889283e710ee947e8412e173d2c050107a3c1/src/compiler/types.ts#L276
pub const RESERVED_IDENTS: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "as",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
];

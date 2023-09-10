use thiserror::Error;

use crate::{
    DataType, EnumRepr, EnumType, EnumVariant, LiteralType, PrimitiveType, StructFields, TypeMap,
};

// TODO: The error should show a path to the type causing the issue like the BigInt error reporting.

#[derive(Error, Debug, PartialEq)]
pub enum SerdeError {
    #[error("A map key must be a 'string' or 'number' type")]
    InvalidMapKey,
    #[error("#[specta(tag = \"...\")] cannot be used with tuple variants")]
    InvalidInternallyTaggedEnum,
}

/// Check that a [DataType] is a valid for Serde.
///
/// This can be used by exporters which wanna do export-time checks that all types are compatible with Serde formats.
pub(crate) fn is_valid_ty(dt: &DataType, type_map: &TypeMap) -> Result<(), SerdeError> {
    match dt {
        DataType::Nullable(ty) => is_valid_ty(ty, type_map)?,
        DataType::Map(ty) => {
            is_valid_map_key(&ty.0, type_map)?;
            is_valid_ty(&ty.1, type_map)?;
        }
        DataType::Struct(ty) => match ty.fields() {
            StructFields::Unit => {}
            StructFields::Unnamed(ty) => {
                for field in ty.fields().iter() {
                    is_valid_ty(&field.ty, type_map)?;
                }
            }
            StructFields::Named(ty) => {
                for (_field_name, field) in ty.fields().iter() {
                    is_valid_ty(&field.ty, type_map)?;
                }
            }
        },
        DataType::Enum(ty) => {
            validate_enum(ty, type_map)?;

            for (_variant_name, variant) in ty.variants().iter() {
                match variant {
                    EnumVariant::Unit => {}
                    EnumVariant::Named(variant) => {
                        for (_field_name, field) in variant.fields.iter() {
                            is_valid_ty(&field.ty, type_map)?;
                        }
                    }
                    EnumVariant::Unnamed(variant) => {
                        for field in variant.fields.iter() {
                            is_valid_ty(&field.ty, type_map)?;
                        }
                    }
                }
            }
        }
        DataType::Tuple(ty) => {
            for field in ty.fields.iter() {
                is_valid_ty(field, type_map)?;
            }
        }
        DataType::Result(ty) => {
            is_valid_ty(&ty.0, type_map)?;
            is_valid_ty(&ty.1, type_map)?;
        }
        DataType::Reference(ty) => {
            for generic in &ty.generics {
                is_valid_ty(generic, type_map)?;
            }

            let ty = type_map
                .get(&ty.sid)
                .as_ref()
                .expect("Reference type not found")
                .as_ref()
                .expect("Type was never populated"); // TODO: Error properly

            is_valid_ty(&ty.inner, type_map)?;
        }
        _ => {}
    }

    Ok(())
}

// Typescript: Must be assignable to `string | number | symbol` says Typescript.
fn is_valid_map_key(key_ty: &DataType, type_map: &TypeMap) -> Result<(), SerdeError> {
    match key_ty {
        DataType::Any => Ok(()),
        DataType::Primitive(ty) => match ty {
            PrimitiveType::i8
            | PrimitiveType::i16
            | PrimitiveType::i32
            | PrimitiveType::i64
            | PrimitiveType::i128
            | PrimitiveType::isize
            | PrimitiveType::u8
            | PrimitiveType::u16
            | PrimitiveType::u32
            | PrimitiveType::u64
            | PrimitiveType::u128
            | PrimitiveType::usize
            | PrimitiveType::f32
            | PrimitiveType::f64
            | PrimitiveType::String
            | PrimitiveType::char => Ok(()),
            _ => Err(SerdeError::InvalidMapKey),
        },
        DataType::Literal(ty) => match ty {
            LiteralType::i8(_)
            | LiteralType::i16(_)
            | LiteralType::i32(_)
            | LiteralType::u8(_)
            | LiteralType::u16(_)
            | LiteralType::u32(_)
            | LiteralType::f32(_)
            | LiteralType::f64(_)
            | LiteralType::String(_)
            | LiteralType::char(_) => Ok(()),
            _ => Err(SerdeError::InvalidMapKey),
        },
        // Enum of other valid types are also valid Eg. `"A" | "B"` or `"A" | 5` are valid
        DataType::Enum(ty) => {
            for (_variant_name, variant) in &ty.variants {
                match variant {
                    EnumVariant::Unit => {}
                    EnumVariant::Unnamed(item) => {
                        if item.fields.len() > 1 {
                            return Err(SerdeError::InvalidMapKey);
                        }

                        if ty.repr != EnumRepr::Untagged {
                            return Err(SerdeError::InvalidMapKey);
                        }
                    }
                    _ => return Err(SerdeError::InvalidMapKey),
                }
            }

            Ok(())
        }
        DataType::Reference(ty) => {
            let ty = type_map
                .get(&ty.sid)
                .as_ref()
                .expect("Reference type not found")
                .as_ref()
                .expect("Type was never populated"); // TODO: Error properly

            is_valid_map_key(&ty.inner, type_map)
        }
        _ => Err(SerdeError::InvalidMapKey),
    }
}

// Serde does not allow serializing a variant of certain types of enum's.
fn validate_enum(e: &EnumType, type_map: &TypeMap) -> Result<(), SerdeError> {
    // Only internally tagged enums can be invalid.
    if let EnumRepr::Internal { .. } = e.repr() {
        validate_internally_tag_enum(e, type_map)?;
    }

    Ok(())
}

// Checks for specially internally tagged enums.
fn validate_internally_tag_enum(e: &EnumType, type_map: &TypeMap) -> Result<(), SerdeError> {
    for (_variant_name, variant) in &e.variants {
        match variant {
            EnumVariant::Unit => {}
            EnumVariant::Named(_) => {}
            EnumVariant::Unnamed(item) => {
                let fields = item.fields();
                if fields.len() > 1 {
                    return Err(SerdeError::InvalidInternallyTaggedEnum);
                }

                validate_internally_tag_enum_datatype(&fields[0].ty, type_map)?;
            }
        }
    }

    Ok(())
}

// Internally tagged enums require map-type's (with a couple of exceptions like `null`)
// Which makes sense when you can't represent `{ "type": "A" } & string` in a single JSON value.
fn validate_internally_tag_enum_datatype(
    ty: &DataType,
    type_map: &TypeMap,
) -> Result<(), SerdeError> {
    match ty {
        // `serde_json::Any` can be *technically* be either valid or invalid based on the actual data but we are being strict and reject it.
        DataType::Any => return Err(SerdeError::InvalidInternallyTaggedEnum),
        DataType::Map(_) => {}
        // Structs's are always map-types unless they are transparent then it depends on inner type. However, transparent passes through when calling `Type::inline` so we don't need to specially check that case.
        DataType::Struct(_) => {}
        DataType::Enum(ty) => match ty.repr {
            // Is only valid if the enum itself is also valid.
            EnumRepr::Untagged => validate_internally_tag_enum(ty, type_map)?,
            // Eg. `{ "Variant": "value" }` is a map-type so valid.
            EnumRepr::External => {}
            // Eg. `{ "type": "variant", "field": "value" }` is a map-type so valid.
            EnumRepr::Internal { .. } => {}
            // Eg. `{ "type": "variant", "c": {} }` is a map-type so valid.
            EnumRepr::Adjacent { .. } => {}
        },
        // `()` is `null` and is valid
        DataType::Tuple(ty) if ty.fields.is_empty() => {}
        // Are valid as they are serialized as an map-type. Eg. `"Ok": 5` or `"Error": "todo"`
        DataType::Result(_) => {}
        // References need to be checked against the same rules.
        DataType::Reference(ty) => {
            let ty = type_map
                .get(&ty.sid)
                .as_ref()
                .expect("Reference type not found")
                .as_ref()
                .expect("Type was never populated"); // TODO: Error properly

            validate_internally_tag_enum_datatype(&ty.inner, type_map)?;
        }
        _ => return Err(SerdeError::InvalidInternallyTaggedEnum),
    }

    Ok(())
}

use specta::{datatype, FunctionResultVariant, TypeMap};

use super::Result;
use crate::{datatype, CommentFormatterArgs, ExportConfig, Typescript};

pub struct FunctionBuilder {
    function: datatype::Function,
}

impl FunctionBuilder {
    pub fn new(function: datatype::Function) -> Self {
        Self { function }
    }

    pub fn render(&self, lang: Typescript) -> Result<String> {
        todo!();
    }
}

// /// Convert a [Function](crate::datatype::Function) into a function header like would be used in a `.d.ts` file.
// /// If your function requires a function body you can copy this function into your own codebase.
// ///
// /// Eg. `function name();`

/// TODO
// TODO: Write some tests for this function
// TODO: Convert into builder???
pub fn render_function(
    dt: specta::datatype::Function,
    config: &ExportConfig,
    type_map: &TypeMap,
    line_prefix: &str,
    // TODO: Better name
    skip_function: bool,
    body: Option<&str>,
) -> Result<String> {
    // TODO: Using `+=` instead of `push_str`???
    let mut s = String::new();
    // TODO: Deprecated, docs
    // config
    //     .comment_exporter
    //     .map(|v| {
    //         v(CommentFormatterArgs {
    //             docs: &dt.docs(),
    //             deprecated: dt.deprecated(),
    //         })
    //     })
    //     .unwrap_or_default();

    if dt.asyncness() {
        s.push_str("async ");
    }

    if !skip_function {
        s.push_str("function ");
    }

    s.push_str(&dt.name());
    s.push_str("(");
    for (i, (name, ty)) in dt.args().enumerate() {
        if i != 0 {
            s.push_str(", ");
        }

        s.push_str(&name);
        s.push_str(": ");
        s.push_str(&datatype(
            config,
            &FunctionResultVariant::Value(ty.clone()),
            &type_map,
        )?);
    }
    s.push_str(")");

    if let Some(ty) = dt.result() {
        s.push_str(": ");
        if dt.asyncness() {
            s.push_str("Promise<");
        }
        s.push_str(&datatype(config, &ty, &type_map)?);
        if dt.asyncness() {
            s.push_str(">");
        }
    }

    if let Some(body) = body {
        s.push_str(" {\n");
        s.push_str(line_prefix);
        s.push_str("\t");
        s.push_str(body);
        s.push_str("\n");
        s.push_str(line_prefix);
        s.push_str("}");
    } else {
        s.push_str(";");
    }

    Ok(s)
}

// fn render_fn(result: &mut String, function: datatype::Function, include_types: bool, body: &str) -> Result<(), specta_typescript::ExportError> {
//     // result.push_str(doc); // TODO
//     result.push_str("async ");
//     result.push_str(&unraw(function.name()).to_lower_camel_case());
//     result.push_str("(");
//     for (i, (key, ty)) in function.args().enumerate() {
//         if i != 0 {
//             result.push_str(", ");
//         }
//         result.push_str(&unraw(key).to_lower_camel_case());
//         if include_types {
//             result.push_str(": ");
//             result
//                 // TODO: Error handling
//                 .push_str(&specta_typescript::datatype(, &mut ExportConfig::default())?);
//         }
//     }
//     result.push_str("): ");
//     // let return_type = return_type
//     //     .map(|t| format!(": Promise<{}>", t))
//     //     .unwrap_or_default();
//     // result.push_str(&function.return_type().unwrap_or_default());
//     result.push_str("Promise<");
//     result.push_str(&"string"); // TODO
//     result.push_str("> {\n\t\t");
//     result.push_str(&body);
//     result.push_str("\n\t},\n");

//     Ok(())
// }

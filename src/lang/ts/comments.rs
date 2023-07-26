/// Allows you to configure how Specta's Typescript exporter will deal with BigInt types ([i64], [i128] etc).
///
/// WARNING: None of these settings affect how your data is actually ser/deserialized.
/// It's up to you to adjust your ser/deserialize settings.
#[derive(Default)]
pub enum BigIntExportBehavior {
    /// Export BigInt as a Typescript `string`
    ///
    /// Doing this is serde is [pretty simple](https://github.com/serde-rs/json/issues/329#issuecomment-305608405).
    String,
    /// Export BigInt as a Typescript `number`.
    ///
    /// WARNING: `JSON.parse` in JS will truncate your number resulting in data loss so ensure your deserializer supports large numbers.
    Number,
    /// Export BigInt as a Typescript `BigInt`.
    BigInt,
    /// Abort the export with an error.
    ///
    /// This is the default behavior because without integration from your serializer and deserializer we can't guarantee data loss won't occur.
    #[default]
    Fail,
    /// Same as `Self::Fail` but it allows a library to configure the message shown to the end user.
    #[doc(hidden)]
    FailWithReason(&'static str),
}

#[derive(Default, PartialEq, Eq)]
pub enum ModuleExportBehavior {
    Enabled,
    #[default]
    Disabled,
}

/// The signature for a function responsible for exporting Typescript comments.
pub type CommentFormatterFn = fn(&[&str]) -> String;

/// Converts Typescript comments into JSDoc comments.
pub fn js_doc(comments: &[&str]) -> String {
    if comments.is_empty() {
        return "".to_owned();
    }

    let mut result = "/**\n".to_owned();
    for comment in comments {
        let comment = comment.trim_start();
        result.push_str(&format!(" * {comment}\n"));
    }
    result.push_str(" */\n");
    result
}

// Assert that the function signature matches the expected type.
const _: CommentFormatterFn = js_doc;

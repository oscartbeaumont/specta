use crate::ts::assert_ts;

#[test]
#[cfg(feature = "serde")]
fn test_json() {
    use specta::{json, True};

    assert_ts!(() => json!(null), "null");
    assert_ts!(() => json!(true), "true");
    assert_ts!(() => json!(false), "false");

    assert_ts!(() => json!({}), "Record<string, never>");
    assert_ts!(() => json!({ "hello": "world" }), "{ hello: string }");
    // assert_ts!(() => json!({
    //     "hello": "world",
    // }), "{ hello: string }");

    assert_ts!(() => json!([]), "[]");
    // assert_ts!(() => json!(["a", "b", "c"]), "string[]");
}

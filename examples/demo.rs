use specta::ts::inline_ref;

fn main() {
    // let v = specta::json!(42u32);

    // println!("{:?}", serde_json::to_string(&v));
    // println!("{:?}", inline_ref(&v, &Default::default()).unwrap());

    let v = specta::json!({
        "hello": "world"
    });

    println!("{:?}", serde_json::to_string(&v));
    println!("{:?}", inline_ref(&v, &Default::default()).unwrap());
}

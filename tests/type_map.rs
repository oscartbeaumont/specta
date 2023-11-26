use specta::{ts, Type, TypeMap};

#[derive(Type)]
#[specta(untagged)]
pub enum GenericType<T> {
    Undefined,
    Value(T),
}

#[derive(Type)]
pub struct ActualType {
    a: GenericType<String>,
}

#[test]
fn test_generic_type_in_type_map() {
    let mut type_map = TypeMap::default();
    ActualType::inline(&mut type_map);

    assert_eq!(type_map.len(), 1);
    let first = type_map.iter().next().unwrap().1;
    // https://github.com/oscartbeaumont/specta/issues/171
    assert_eq!(
        ts::export_named_datatype(&Default::default(), first, &type_map).unwrap(),
        "export type GenericType<T> = null | T".to_string()
    );
}

use specta::{DataType, PrimitiveType, Type};

use crate::ts::assert_ts;

#[derive(Type)]
#[specta(export = false, transparent)]
struct TupleStruct(String);

#[repr(transparent)]
#[derive(Type)]
#[specta(export = false)]
struct TupleStructWithRep(String);

#[derive(Type)]
#[specta(export = false, transparent)]
struct GenericTupleStruct<T>(T);

#[derive(Type)]
#[specta(export = false, transparent)]
pub struct BracedStruct {
    a: String,
}

#[test]
fn transparent() {
    // We check the datatype layer can TS can look correct but be wrong!
    assert_eq!(
        TupleStruct::inline(&mut Default::default()),
        DataType::Primitive(PrimitiveType::String)
    );
    assert_eq!(
        TupleStructWithRep::inline(&mut Default::default()),
        DataType::Primitive(PrimitiveType::String)
    );
    assert_eq!(
        GenericTupleStruct::<String>::inline(&mut Default::default()),
        DataType::Primitive(PrimitiveType::String)
    );
    assert_eq!(
        BracedStruct::inline(&mut Default::default()),
        DataType::Primitive(PrimitiveType::String)
    );

    assert_ts!(TupleStruct, "string");
    assert_ts!(TupleStructWithRep, "string");
    assert_ts!(GenericTupleStruct::<String>, "string");
    assert_ts!(BracedStruct, "string");
}

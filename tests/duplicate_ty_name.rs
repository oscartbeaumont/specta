use specta::{
    ts::{export, ExportError},
    Type,
};

mod one {
    use super::*;

    #[derive(Type)]
    #[specta(export = false)]
    pub struct One {
        pub a: String,
    }
}

mod two {
    use super::*;

    #[derive(Type)]
    #[specta(export = false)]
    pub struct One {
        pub b: String,
        pub c: i32,
    }
}

#[derive(Type)]
#[specta(export = false)]
pub struct Demo {
    pub one: one::One,
    pub two: two::One,
}

#[test]
fn test_duplicate_ty_name() {
    // DO NOT COPY THIS. This is a hack to construct the impl locations but IS NOT STABLE.
    use specta::internal::construct::impl_location;

    #[cfg(not(target_os = "windows"))]
    let err = Err(ExportError::DuplicateTypeName(
        "One".into(),
        impl_location("tests/duplicate_ty_name.rs:9:14"),
        impl_location("tests/duplicate_ty_name.rs:19:14"),
    ));
    #[cfg(target_os = "windows")]
    let err = Err(ExportError::DuplicateTypeName(
        "One".into(),
        impl_location("tests\\duplicate_ty_name.rs:9:14"),
        impl_location("tests\\duplicate_ty_name.rs:19:14"),
    ));

    assert_eq!(export::<Demo>(&Default::default()), err);
}

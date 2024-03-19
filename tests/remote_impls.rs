#[specta_macros::stest(feature = "glam")]
fn typescript_types_glam() {
    use crate::ts::assert_ts;

    assert_ts!(glam::DVec2, "[number, number]");
    assert_ts!(glam::IVec2, "[number, number]");
    assert_ts!(glam::DMat2, "[number, number, number, number]");
    assert_ts!(
        glam::DAffine2,
        "[number, number, number, number, number, number]"
    );
    assert_ts!(glam::Vec2, "[number, number]");
    assert_ts!(glam::Vec3, "[number, number, number]");
    assert_ts!(glam::Vec3A, "[number, number, number]");
    assert_ts!(glam::Vec4, "[number, number, number, number]");
    assert_ts!(glam::Mat2, "[number, number, number, number]");
    assert_ts!(
        glam::Mat3,
        "[number, number, number, number, number, number, number, number, number]"
    );
    assert_ts!(
        glam::Mat3A,
        "[number, number, number, number, number, number, number, number, number]"
    );
    assert_ts!(glam::Mat4, "[number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number]");
    assert_ts!(glam::Quat, "[number, number, number, number]");
    assert_ts!(
        glam::Affine2,
        "[number, number, number, number, number, number]"
    );
    assert_ts!(glam::Affine3A, "[number, number, number, number, number, number, number, number, number, number, number, number]");
}

#[test]
#[cfg(feature = "bevy_ecs")]
fn typescript_types_bevy_ecs() {
    use specta::ts::{self, BigIntExportBehavior, ExportConfig, ExportPath};

    use crate::ts::assert_ts;

    assert_eq!(
        ts::inline::<bevy_ecs::entity::Entity>(
            &ExportConfig::default().bigint(BigIntExportBehavior::Number)
        ),
        Ok("number".into())
    );
    // TODO: As we inline `Entity` never ends up in the type map so it falls back to "Entity" in the error instead of the path to the type. Is this what we want or not?
    assert_ts!(error; bevy_ecs::entity::Entity, specta::ts::ExportError::BigIntForbidden(ExportPath::new_unsafe("Entity -> u64")));

    // https://github.com/oscartbeaumont/specta/issues/161#issuecomment-1822735951
    assert_eq!(
        ts::export::<bevy_ecs::entity::Entity>(
            &ExportConfig::default().bigint(BigIntExportBehavior::Number)
        ),
        Ok("export type Entity = number".into())
    );
}

#[cfg(feature = "engine")]
#[test]
fn engine_feature_reexports_engine_api() {
    let session = tokel::engine::session::Session::empty();

    assert!(session.registry().is_empty());
}

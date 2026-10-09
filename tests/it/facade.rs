#[test]
fn enabled_features_are_reexported() {
    use sabai::http as _;

    #[cfg(feature = "orm")]
    use sabai::orm as _;

    #[cfg(feature = "auth")]
    use sabai::auth as _;

    #[cfg(feature = "queue")]
    use sabai::queue as _;
}

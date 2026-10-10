use sabai::log::{self, Logging};

#[test]
fn apps_log_through_the_facade_without_depending_on_tracing() {
    let post_id = 7;

    log::info!(post_id, "post published");
    log::error!("payment failed for post {post_id}");

    assert!(Logging::new("info,sabai=debug").is_ok());
}

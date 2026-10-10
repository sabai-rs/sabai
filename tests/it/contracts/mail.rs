use std::sync::{Arc, Mutex};

use pollster::block_on;
use sabai::contracts::{BoxFuture, MailTransport, Message};
use sabai::{Container, Result};

/// Records messages instead of sending them, the way `Mail::fake()` will (QA-06).
#[derive(Default)]
struct FakeTransport {
    sent: Mutex<Vec<Message>>,
}

impl MailTransport for FakeTransport {
    fn send<'a>(&'a self, message: &'a Message) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.sent.lock().unwrap().push(message.clone());
            Ok(())
        })
    }
}

async fn send_welcome(container: &Container, email: &str) -> Result<()> {
    let message = Message::builder()
        .from("hello@sabai.dev")
        .to(email)
        .subject("Welcome to Sabai")
        .text("Rust, but sabai.")
        .build()?;
    container.get::<dyn MailTransport>()?.send(&message).await
}

#[test]
fn code_sends_through_whatever_transport_is_bound() {
    let fake = Arc::new(FakeTransport::default());
    let mut container = Container::default();
    container.bind::<dyn MailTransport>(fake.clone());

    block_on(send_welcome(&container, "ann@example.com")).unwrap();

    let sent = fake.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to()[0].email(), "ann@example.com");
    assert_eq!(sent[0].subject(), "Welcome to Sabai");
}

#[test]
fn an_unsafe_message_never_reaches_the_transport() {
    let fake = Arc::new(FakeTransport::default());
    let mut container = Container::default();
    container.bind::<dyn MailTransport>(fake.clone());

    let result = block_on(send_welcome(
        &container,
        "ann@example.com\r\nBcc: everyone@victim.example",
    ));

    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("could inject headers")
    );
    assert!(fake.sent.lock().unwrap().is_empty());
}

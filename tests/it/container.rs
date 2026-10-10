use std::sync::{Arc, Mutex};

use sabai::{Container, Result};

trait Mailer: Send + Sync {
    fn send(&self, to: &str, subject: &str);
}

#[derive(Default)]
struct FakeMailer {
    sent: Mutex<Vec<String>>,
}

impl Mailer for FakeMailer {
    fn send(&self, to: &str, subject: &str) {
        self.sent.lock().unwrap().push(format!("{to}: {subject}"));
    }
}

fn welcome(container: &Container, email: &str) -> Result<()> {
    let mailer = container.get::<dyn Mailer>()?;
    mailer.send(email, "Welcome to Sabai");
    Ok(())
}

#[test]
fn code_asks_for_a_contract_and_tests_bind_a_fake() {
    let fake = Arc::new(FakeMailer::default());
    let mut container = Container::default();
    container.bind::<dyn Mailer>(fake.clone());

    welcome(&container, "ann@example.com").unwrap();

    assert_eq!(
        *fake.sent.lock().unwrap(),
        ["ann@example.com: Welcome to Sabai"]
    );
}

#[test]
fn a_missing_contract_fails_with_a_hint_instead_of_panicking() {
    let error = welcome(&Container::default(), "ann@example.com").unwrap_err();

    assert!(error.to_string().contains("bind it in a provider"));
}

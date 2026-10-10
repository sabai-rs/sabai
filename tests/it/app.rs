use std::sync::{Arc, Mutex};

use sabai::{App, Config, Error, Provider, Result, StatusCode};

type Journal = Arc<Mutex<Vec<String>>>;

trait Mailer: Send + Sync {
    fn transport(&self) -> &'static str;
}

struct LogMailer;

impl Mailer for LogMailer {
    fn transport(&self) -> &'static str {
        "log"
    }
}

/// Needs a `Mailer` in `boot`, but is added before the provider that binds it.
struct NotificationProvider(Journal);

impl Provider for NotificationProvider {
    fn register(&self, _app: &mut App) -> Result<()> {
        self.0
            .lock()
            .unwrap()
            .push("notifications: register".into());
        Ok(())
    }

    fn boot(&self, app: &App) -> Result<()> {
        let transport = app.get::<dyn Mailer>()?.transport();
        self.0
            .lock()
            .unwrap()
            .push(format!("notifications: boot with {transport} mailer"));
        Ok(())
    }
}

struct MailProvider(Journal);

impl Provider for MailProvider {
    fn register(&self, app: &mut App) -> Result<()> {
        app.bind::<dyn Mailer>(Arc::new(LogMailer));
        self.0.lock().unwrap().push("mail: register".into());
        Ok(())
    }
}

struct BrokenProvider;

impl Provider for BrokenProvider {
    fn register(&self, _app: &mut App) -> Result<()> {
        Err(Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "mail.host is not set",
        ))
    }
}

#[test]
fn every_provider_registers_before_any_boots() {
    let journal = Journal::default();

    App::builder(Config::default())
        .provider(NotificationProvider(journal.clone()))
        .provider(MailProvider(journal.clone()))
        .boot()
        .unwrap();

    assert_eq!(
        *journal.lock().unwrap(),
        [
            "notifications: register",
            "mail: register",
            "notifications: boot with log mailer",
        ]
    );
}

#[test]
fn a_failing_provider_stops_boot_with_its_error() {
    let journal = Journal::default();

    let error = App::builder(Config::default())
        .provider(BrokenProvider)
        .provider(MailProvider(journal.clone()))
        .boot()
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "provider `it::app::BrokenProvider` failed to register: mail.host is not set"
    );
    assert!(journal.lock().unwrap().is_empty());
}

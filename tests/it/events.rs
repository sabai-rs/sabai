use std::sync::{Arc, Mutex};

use sabai::{App, Config, Error, Event, Listener, Provider, Result, StatusCode};

type Journal = Arc<Mutex<Vec<String>>>;

struct OrderShipped {
    order_id: u64,
}

impl Event for OrderShipped {}

struct UserRegistered;

impl Event for UserRegistered {}

/// A listener as a struct, like a Laravel listener class.
struct SendShipmentNotification;

impl Listener<OrderShipped> for SendShipmentNotification {
    fn handle(&self, event: &OrderShipped, app: &App) -> Result<()> {
        let journal = app.get::<Mutex<Vec<String>>>()?;
        journal
            .lock()
            .unwrap()
            .push(format!("notify order {}", event.order_id));
        Ok(())
    }
}

struct ShopProvider(Journal);

impl Provider for ShopProvider {
    fn register(&self, app: &mut App) -> Result<()> {
        app.bind(self.0.clone());
        app.listen(SendShipmentNotification);
        app.listen(|event: &OrderShipped, app: &App| {
            let journal = app.get::<Mutex<Vec<String>>>()?;
            journal
                .lock()
                .unwrap()
                .push(format!("update stock for order {}", event.order_id));
            Ok(())
        });
        Ok(())
    }
}

fn shop(journal: &Journal) -> Arc<App> {
    App::builder(Config::default())
        .provider(ShopProvider(journal.clone()))
        .boot()
        .unwrap()
}

#[test]
fn listeners_run_in_the_order_they_were_added() {
    let journal = Journal::default();

    shop(&journal)
        .dispatch(&OrderShipped { order_id: 7 })
        .unwrap();

    assert_eq!(
        *journal.lock().unwrap(),
        ["notify order 7", "update stock for order 7"]
    );
}

#[test]
fn an_event_without_listeners_is_fine() {
    let journal = Journal::default();

    shop(&journal).dispatch(&UserRegistered).unwrap();

    assert!(journal.lock().unwrap().is_empty());
}

/// Adds a listener that fails, before one that would record a notification.
struct FlakyShopProvider(Journal);

impl Provider for FlakyShopProvider {
    fn register(&self, app: &mut App) -> Result<()> {
        app.bind(self.0.clone());
        app.listen(|_: &OrderShipped, _: &App| {
            Err(Error::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "mail server down",
            ))
        });
        app.listen(SendShipmentNotification);
        Ok(())
    }
}

#[test]
fn a_failing_listener_stops_the_rest_and_is_named() {
    let journal = Journal::default();
    let app = App::builder(Config::default())
        .provider(FlakyShopProvider(journal.clone()))
        .boot()
        .unwrap();

    let error = app.dispatch(&OrderShipped { order_id: 7 }).unwrap_err();

    assert!(error.to_string().starts_with("listener `"), "{error}");
    assert!(
        error
            .to_string()
            .ends_with("for `it::events::OrderShipped` failed: mail server down")
    );
    assert!(journal.lock().unwrap().is_empty());
}

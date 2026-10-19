use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use sabai::{App, Command, Result};

const QUOTES: &[&str] = &[
    "Simplicity is the ultimate sophistication. - Leonardo da Vinci",
    "Well begun is half done. - Aristotle",
    "Act only according to that maxim whereby you can, at the same time, will that it should become a universal law. - Immanuel Kant",
    "Nothing worth having comes easy. - Theodore Roosevelt",
];

/// Prints a quote, like Laravel's `php artisan inspire`.
pub struct Inspire;

impl Command for Inspire {
    fn name(&self) -> &str {
        "inspire"
    }

    fn description(&self) -> &str {
        "Show an inspiring quote"
    }

    fn handle(&self, _app: &Arc<App>, _args: &[String]) -> Result<()> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        println!("{}", QUOTES[seconds as usize % QUOTES.len()]);
        Ok(())
    }
}

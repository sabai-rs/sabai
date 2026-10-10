mod address;
mod message;

pub use address::Address;
pub use message::{Message, MessageBuilder};

use super::BoxFuture;
use crate::Result;

/// Delivers a finished message, like a Symfony mailer transport (SMTP, SES, log). The app-facing
/// `Mailer` with mailables, views and queues is built on top of it.
pub trait MailTransport: Send + Sync {
    /// Sends `message` to all of its recipients.
    fn send<'a>(&'a self, message: &'a Message) -> BoxFuture<'a, Result<()>>;
}

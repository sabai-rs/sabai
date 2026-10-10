use std::error::Error as StdError;
use std::fmt;

use super::Address;
use crate::{Error, Result};

/// A finished email, checked by [`MessageBuilder::build`], ready to hand to a transport.
// Fields are private so attachments and headers can be added without breaking transports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    from: Address,
    to: Vec<Address>,
    cc: Vec<Address>,
    bcc: Vec<Address>,
    reply_to: Option<Address>,
    subject: String,
    text: Option<String>,
    html: Option<String>,
}

/// Builds a [`Message`]: `Message::builder().from(..).to(..).subject(..).text(..).build()?`.
#[derive(Debug, Default)]
pub struct MessageBuilder {
    from: Option<Address>,
    to: Vec<Address>,
    cc: Vec<Address>,
    bcc: Vec<Address>,
    reply_to: Option<Address>,
    subject: String,
    text: Option<String>,
    html: Option<String>,
}

impl Message {
    /// Starts a new message.
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }

    /// The sender.
    pub fn from(&self) -> &Address {
        &self.from
    }

    /// The main recipients.
    pub fn to(&self) -> &[Address] {
        &self.to
    }

    /// Recipients in copy.
    pub fn cc(&self) -> &[Address] {
        &self.cc
    }

    /// Hidden recipients; transports must never write them into the message headers.
    pub fn bcc(&self) -> &[Address] {
        &self.bcc
    }

    /// Where replies go, when it is not the sender.
    pub fn reply_to(&self) -> Option<&Address> {
        self.reply_to.as_ref()
    }

    /// The subject line.
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// The plain-text body.
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// The HTML body.
    pub fn html(&self) -> Option<&str> {
        self.html.as_deref()
    }

    fn recipients(&self) -> impl Iterator<Item = &Address> {
        self.to.iter().chain(&self.cc).chain(&self.bcc)
    }

    fn first_problem(&self) -> Option<String> {
        if self.recipients().next().is_none() {
            return Some("it has no recipient; call `.to(...)`".into());
        }
        if self.text.is_none() && self.html.is_none() {
            return Some("it has no body; call `.text(...)` or `.html(...)`".into());
        }
        if has_line_break(&self.subject) {
            return Some("the subject contains a line break, which could inject headers".into());
        }
        let mut addresses = std::iter::once(&self.from)
            .chain(&self.reply_to)
            .chain(self.recipients());
        addresses.find_map(address_problem)
    }
}

impl MessageBuilder {
    /// Sets the sender.
    pub fn from(mut self, address: impl Into<Address>) -> Self {
        self.from = Some(address.into());
        self
    }

    /// Adds a main recipient; call it again for more.
    pub fn to(mut self, address: impl Into<Address>) -> Self {
        self.to.push(address.into());
        self
    }

    /// Adds a recipient in copy.
    pub fn cc(mut self, address: impl Into<Address>) -> Self {
        self.cc.push(address.into());
        self
    }

    /// Adds a hidden recipient.
    pub fn bcc(mut self, address: impl Into<Address>) -> Self {
        self.bcc.push(address.into());
        self
    }

    /// Sets where replies go.
    pub fn reply_to(mut self, address: impl Into<Address>) -> Self {
        self.reply_to = Some(address.into());
        self
    }

    /// Sets the subject line.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = subject.into();
        self
    }

    /// Sets the plain-text body.
    pub fn text(mut self, body: impl Into<String>) -> Self {
        self.text = Some(body.into());
        self
    }

    /// Sets the HTML body.
    pub fn html(mut self, body: impl Into<String>) -> Self {
        self.html = Some(body.into());
        self
    }

    /// Checks the message and returns it, or an error that says what is missing or unsafe.
    pub fn build(self) -> Result<Message> {
        let Some(from) = self.from else {
            return Err(invalid("it has no sender; call `.from(...)`".into()));
        };
        let message = Message {
            from,
            to: self.to,
            cc: self.cc,
            bcc: self.bcc,
            reply_to: self.reply_to,
            subject: self.subject,
            text: self.text,
            html: self.html,
        };
        match message.first_problem() {
            Some(problem) => Err(invalid(problem)),
            None => Ok(message),
        }
    }
}

// A line break in a header lets a caller append their own headers, such as a hidden `Bcc:`.
fn has_line_break(text: &str) -> bool {
    text.contains(['\r', '\n'])
}

fn address_problem(address: &Address) -> Option<String> {
    let email = address.email();
    if has_line_break(email) || address.name().is_some_and(has_line_break) {
        let shown = email.escape_debug();
        return Some(format!(
            "the address `{shown}` contains a line break, which could inject headers"
        ));
    }
    let looks_like_email = !email.contains(char::is_whitespace)
        && email
            .split_once('@')
            .is_some_and(|(user, domain)| !user.is_empty() && !domain.is_empty());
    (!looks_like_email).then(|| format!("`{email}` is not an email address"))
}

#[derive(Debug)]
struct InvalidMessage {
    problem: String,
}

impl fmt::Display for InvalidMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot send this mail: {}", self.problem)
    }
}

impl StdError for InvalidMessage {}

fn invalid(problem: String) -> Error {
    InvalidMessage { problem }.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn welcome() -> MessageBuilder {
        Message::builder()
            .from("app@example.com")
            .to("ann@example.com")
            .subject("Welcome")
            .text("Hi")
    }

    fn problem(builder: MessageBuilder) -> String {
        builder.build().unwrap_err().to_string()
    }

    #[test]
    fn a_complete_message_builds() {
        let message = welcome()
            .to(Address::named("bob@example.com", "Bob"))
            .bcc("audit@example.com")
            .build()
            .unwrap();

        assert_eq!(message.to().len(), 2);
        assert_eq!(message.to()[1].to_string(), "\"Bob\" <bob@example.com>");
        assert_eq!(message.bcc()[0].email(), "audit@example.com");
    }

    #[test]
    fn missing_parts_say_which_call_to_add() {
        assert_eq!(
            problem(Message::builder()),
            "cannot send this mail: it has no sender; call `.from(...)`"
        );
        assert!(problem(Message::builder().from("a@b.c").text("Hi")).ends_with("call `.to(...)`"));
        assert!(problem(Message::builder().from("a@b.c").to("d@e.f")).ends_with("or `.html(...)`"));
    }

    #[test]
    fn line_breaks_in_headers_are_rejected() {
        let injected_subject = welcome().subject("Hi\r\nBcc: everyone@victim.example");
        let injected_name = welcome().to(Address::named("bob@example.com", "Bob\nBcc: x@y.z"));

        assert!(problem(injected_subject).contains("subject contains a line break"));
        assert!(problem(injected_name).contains("`bob@example.com` contains a line break"));
    }

    #[test]
    fn something_that_is_not_an_email_is_rejected() {
        assert!(
            problem(welcome().to("ann at example.com"))
                .contains("`ann at example.com` is not an email address")
        );
        assert!(problem(welcome().cc("@example.com")).contains("is not an email address"));
    }
}

use std::error::Error as StdError;
use std::fmt;

use tracing::Subscriber;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

pub use tracing::{debug, error, info, trace, warn};

use crate::Result;

type BoxedLayer = Box<dyn Layer<Registry> + Send + Sync>;

/// Log setup: a level filter such as `info,sabai=debug`, console output, and extra layers such as Sentry.
pub struct Logging {
    filter: Targets,
    layers: Vec<BoxedLayer>,
}

impl Logging {
    /// Logging at `filter`: one level (`info`) or per-target levels (`info,sabai=debug`).
    pub fn new(filter: &str) -> Result<Self> {
        let invalid = |problem: String| InvalidFilter {
            filter: filter.to_owned(),
            problem,
        };
        if let Some(word) = first_bare_word_that_is_not_a_level(filter) {
            return Err(invalid(format!("`{word}` is not a level")).into());
        }
        let filter = filter
            .parse()
            .map_err(|source| invalid(format!("{source}")))?;
        Ok(Self {
            filter,
            layers: Vec::new(),
        })
    }

    /// Adds a `tracing` layer that receives every event the filter lets through, for example Sentry's.
    pub fn layer(self, layer: impl Layer<Registry> + Send + Sync + 'static) -> Self {
        self.push_layer(Box::new(layer))
    }

    fn push_layer(mut self, layer: BoxedLayer) -> Self {
        self.layers.push(layer);
        self
    }

    /// The subscriber without installing it, so tests can scope it with `tracing::subscriber::with_default`.
    pub fn into_subscriber(self) -> impl Subscriber + Send + Sync {
        tracing_subscriber::registry()
            .with(self.layers)
            .with(tracing_subscriber::fmt::layer())
            .with(self.filter)
    }

    /// Installs this setup for the whole process; call it once at boot.
    pub fn init(self) -> Result<()> {
        Ok(self.into_subscriber().try_init()?)
    }
}

// `Targets` reads a bare word as a target name, so a typo such as `infoo` would
// silently turn logging off instead of failing.
fn first_bare_word_that_is_not_a_level(filter: &str) -> Option<&str> {
    filter
        .split(',')
        .map(str::trim)
        .filter(|directive| !directive.contains('='))
        .find(|word| word.parse::<LevelFilter>().is_err())
}

#[derive(Debug)]
struct InvalidFilter {
    filter: String,
    problem: String,
}

impl fmt::Display for InvalidFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid log filter `{}`: {}; use a level such as `info`, \
             or per-target levels such as `info,sabai=debug`",
            self.filter, self.problem
        )
    }
}

impl StdError for InvalidFilter {}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tracing::Event;
    use tracing_subscriber::layer::Context;

    use super::*;

    #[derive(Clone, Default)]
    struct Recorder(Arc<Mutex<Vec<String>>>);

    impl<S: Subscriber> Layer<S> for Recorder {
        fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
            let metadata = event.metadata();
            let line = format!("{} {}", metadata.level(), metadata.target());
            self.0.lock().unwrap().push(line);
        }
    }

    #[test]
    fn extra_layers_receive_only_what_the_filter_lets_through() {
        let recorder = Recorder::default();
        let logging = Logging::new("warn,sabai_core=debug")
            .unwrap()
            .layer(recorder.clone());

        tracing::subscriber::with_default(logging.into_subscriber(), || {
            debug!(target: "sabai_core", "kept: sabai_core allows debug");
            info!(target: "other", "dropped: below warn");
            warn!(target: "other", "kept: warn is allowed everywhere");
        });

        assert_eq!(
            *recorder.0.lock().unwrap(),
            ["DEBUG sabai_core", "WARN other"]
        );
    }

    #[test]
    fn an_invalid_filter_explains_the_format() {
        let error = Logging::new("infoo,sabai=debug").err().unwrap();

        assert_eq!(
            error.to_string(),
            "invalid log filter `infoo,sabai=debug`: `infoo` is not a level; \
             use a level such as `info`, or per-target levels such as `info,sabai=debug`"
        );
    }
}

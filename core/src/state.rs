use config::Config;
use event::{Event, Events};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub struct State {
    pub config: Config,
    pub events: HashMap<Events, Vec<Arc<Event>>>,
}

impl State {
    pub fn new(config: Config, events: HashMap<Events, Vec<Arc<Event>>>) -> Self {
        Self { config, events }
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "State: {{ config: {:?} }}", self.config)
    }
}

/// Builder for constructing the application state
pub struct StateBuilder {
    config: Config,
    events: HashMap<Events, Vec<Arc<Event>>>,
}

impl StateBuilder {
    /// Creates a default builder
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            events: HashMap::new(),
        }
    }

    /// Sets the configuration
    pub fn set_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Adds a new event
    pub fn add_event(mut self, events: Events, event: Arc<Event>) -> Self {
        match self.events.entry(events) {
            Entry::Vacant(e)       => {
                e.insert(vec![event]);
                ()
            }
            Entry::Occupied(mut e) => e.get_mut().push(event),
        }
        self
    }

    /// Creates a new state
    pub fn build(self) -> State {
        State::new(self.config, self.events)
    }
}

impl Debug for StateBuilder {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "State: {{ config: {:?} }}", self.config)
    }
}

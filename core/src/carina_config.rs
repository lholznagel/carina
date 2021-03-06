use carina_core_protocol::Events;
use config::Config;
use event::Event;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

/// Contains the configuration and all events
pub struct CarinaConfig {
    /// configuration of the peer
    pub config: Config,
    /// events to listen
    pub events: HashMap<Events, Vec<Arc<Mutex<Event>>>>,
}

impl CarinaConfig {
    /// creates a new instance
    pub fn new(config: Config, events: HashMap<Events, Vec<Arc<Mutex<Event>>>>) -> Self {
        Self { config, events }
    }
}

impl Debug for CarinaConfig {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "CarinaConfig: {{ config: {:?} }}", self.config)
    }
}

/// Builder for constructing the application carina config
pub struct CarinaConfigBuilder {
    config: Config,
    events: HashMap<Events, Vec<Arc<Mutex<Event>>>>,
}

impl CarinaConfigBuilder {
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
    pub fn add_event<T: Event + 'static>(mut self, events: Events, event: Arc<Mutex<T>>) -> Self {
        match self.events.entry(events) {
            Entry::Vacant(e)       => {
                e.insert(vec![event]);
                ()
            }
            Entry::Occupied(mut e) => e.get_mut().push(event),
        }
        self
    }

    /// Creates a new carina config instance
    pub fn build(self) -> CarinaConfig {
        CarinaConfig::new(self.config, self.events)
    }
}

impl Debug for CarinaConfigBuilder {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Carina: {{ config: {:?} }}", self.config)
    }
}

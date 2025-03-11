use std::fmt::Debug;

use chrono::{DateTime, TimeDelta, Utc};

use super::arithmetic::Arithmetic;

/// A structure that defines a signal, which carries information
/// about its intensity and the time of its emission.
#[derive(Debug, Clone, PartialEq)]
pub struct Signal<S: Arithmetic> {
    /// The signal creation DateTime
    created_at: DateTime<Utc>,

    /// The intensity
    intensity: S,
}

impl<S> Signal<S>
where
    S: Arithmetic,
{
    /// Create new signal instance.
    pub fn new(intensity: S) -> Self {
        Signal {
            intensity,
            created_at: Utc::now(),
        }
    }

    /// Returns age of the signal.
    pub fn age(&self) -> TimeDelta {
        Utc::now() - self.created_at
    }

    /// Return intensity of the signal.
    pub fn intensity(&self) -> S {
        self.intensity.clone()
    }
}

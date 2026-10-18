//! Hardware abstraction layer (Part D).
//!
//! Everything above this crate talks to lights, hit sensors and time through
//! the traits below, so the same code runs against real Pi hardware or the
//! mocks in [`mock`].
//!
//! Status: skeleton. Trait shapes are provisional (see docs/PLAN.md, 0.5).

pub mod mock;

use std::fmt;
use std::sync::mpsc::Receiver;
use std::time::Instant;

/// Index of one target (hole). `0..N`, where `N` is configurable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TargetId(pub u8);

/// 8-bit RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub const OFF: Rgb = Rgb(0, 0, 0);
}

/// A detected hit on one target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitEvent {
    pub target: TargetId,
    /// Monotonic timestamp taken as close to the sample as possible.
    pub at: Instant,
    /// Raw sensor magnitude, kept for threshold tuning.
    pub peak: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HalError {
    InvalidTarget(TargetId),
    Io(String),
}

impl fmt::Display for HalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HalError::InvalidTarget(t) => write!(f, "invalid target id {}", t.0),
            HalError::Io(msg) => write!(f, "hardware I/O error: {msg}"),
        }
    }
}

impl std::error::Error for HalError {}

pub type Result<T> = std::result::Result<T, HalError>;

/// Stream of hit events. A plain channel for now; Part D decides whether this
/// becomes an async `Stream`.
pub type HitStream = Receiver<HitEvent>;

pub trait Lights {
    fn set(&mut self, target: TargetId, color: Rgb) -> Result<()>;
    fn clear(&mut self, target: TargetId) -> Result<()>;
    fn clear_all(&mut self) -> Result<()>;
}

pub trait HitSensor {
    /// Take the event stream. Can only be taken once per sensor.
    fn events(&mut self) -> Result<HitStream>;
}

pub trait Clock {
    fn now(&self) -> Instant;
}

/// Real monotonic clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_clock_is_monotonic() {
        let c = SystemClock;
        let a = c.now();
        let b = c.now();
        assert!(b >= a);
    }

    #[test]
    fn error_display() {
        assert_eq!(
            HalError::InvalidTarget(TargetId(42)).to_string(),
            "invalid target id 42"
        );
    }
}

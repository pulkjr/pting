//! Mock hardware for tests and for running on a developer machine.

use crate::{Clock, HalError, HitEvent, HitSensor, HitStream, Lights, Result, Rgb, TargetId};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// In-memory lights. Inspect [`MockLights::state`] in tests.
#[derive(Debug, Clone)]
pub struct MockLights {
    state: Vec<Rgb>,
}

impl MockLights {
    pub fn new(targets: u8) -> Self {
        Self {
            state: vec![Rgb::OFF; targets as usize],
        }
    }

    pub fn state(&self) -> &[Rgb] {
        &self.state
    }

    fn slot(&mut self, target: TargetId) -> Result<&mut Rgb> {
        self.state
            .get_mut(target.0 as usize)
            .ok_or(HalError::InvalidTarget(target))
    }
}

impl Lights for MockLights {
    fn set(&mut self, target: TargetId, color: Rgb) -> Result<()> {
        *self.slot(target)? = color;
        Ok(())
    }

    fn clear(&mut self, target: TargetId) -> Result<()> {
        self.set(target, Rgb::OFF)
    }

    fn clear_all(&mut self) -> Result<()> {
        self.state.fill(Rgb::OFF);
        Ok(())
    }
}

/// Handle used by tests (or a keyboard harness) to inject hits.
#[derive(Debug, Clone)]
pub struct HitInjector {
    tx: Sender<HitEvent>,
}

impl HitInjector {
    pub fn hit(&self, target: TargetId, at: Instant, peak: u16) {
        // If the receiver is gone nobody is listening; ignoring is fine in a mock.
        let _ = self.tx.send(HitEvent { target, at, peak });
    }
}

#[derive(Debug)]
pub struct MockHitSensor {
    tx: Sender<HitEvent>,
    rx: Option<HitStream>,
}

impl MockHitSensor {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self { tx, rx: Some(rx) }
    }

    pub fn injector(&self) -> HitInjector {
        HitInjector {
            tx: self.tx.clone(),
        }
    }
}

impl Default for MockHitSensor {
    fn default() -> Self {
        Self::new()
    }
}

impl HitSensor for MockHitSensor {
    fn events(&mut self) -> Result<HitStream> {
        self.rx
            .take()
            .ok_or_else(|| HalError::Io("hit stream already taken".into()))
    }
}

/// Manually advanced clock for deterministic tests. Clones share time.
#[derive(Debug, Clone)]
pub struct MockClock {
    base: Instant,
    offset: Arc<Mutex<Duration>>,
}

impl MockClock {
    pub fn new() -> Self {
        Self {
            base: Instant::now(),
            offset: Arc::new(Mutex::new(Duration::ZERO)),
        }
    }

    pub fn advance(&self, by: Duration) {
        *self.offset.lock().expect("mock clock poisoned") += by;
    }
}

impl Default for MockClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for MockClock {
    fn now(&self) -> Instant {
        self.base + *self.offset.lock().expect("mock clock poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lights_set_and_clear() {
        let mut l = MockLights::new(3);
        l.set(TargetId(1), Rgb(255, 0, 0)).unwrap();
        assert_eq!(l.state()[1], Rgb(255, 0, 0));
        l.clear(TargetId(1)).unwrap();
        assert_eq!(l.state()[1], Rgb::OFF);
        l.set(TargetId(0), Rgb(1, 2, 3)).unwrap();
        l.clear_all().unwrap();
        assert!(l.state().iter().all(|c| *c == Rgb::OFF));
    }

    #[test]
    fn lights_reject_unknown_target() {
        let mut l = MockLights::new(2);
        assert_eq!(
            l.set(TargetId(2), Rgb(1, 1, 1)),
            Err(HalError::InvalidTarget(TargetId(2)))
        );
    }

    #[test]
    fn injected_hit_is_received() {
        let mut s = MockHitSensor::new();
        let inj = s.injector();
        let rx = s.events().unwrap();
        let now = Instant::now();
        inj.hit(TargetId(4), now, 700);
        let ev = rx.recv().unwrap();
        assert_eq!(ev.target, TargetId(4));
        assert_eq!(ev.at, now);
        assert_eq!(ev.peak, 700);
    }

    #[test]
    fn event_stream_can_only_be_taken_once() {
        let mut s = MockHitSensor::new();
        assert!(s.events().is_ok());
        assert!(s.events().is_err());
    }

    #[test]
    fn mock_clock_advances_only_when_told() {
        let c = MockClock::new();
        let t0 = c.now();
        assert_eq!(c.now(), t0);
        c.advance(Duration::from_millis(250));
        assert_eq!(c.now() - t0, Duration::from_millis(250));
    }
}

//! Game engine (Part F). Pure logic: no hardware, no networking.
//!
//! Status: placeholder. Only enough here to prove the crate graph works.

use pting_hal::HitEvent;
use std::time::Instant;

/// Coarse game phases (see docs/PLAN.md, Part F).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Lobby,
    ColorsChosen,
    TurnActive,
    Scored,
    Finished,
}

/// Milliseconds from the target lighting up to the hit, or `None` if the hit
/// is timestamped before the light came on (stale or bogus event).
pub fn reaction_ms(lit_at: Instant, hit: &HitEvent) -> Option<u64> {
    hit.at
        .checked_duration_since(lit_at)
        .map(|d| d.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pting_hal::mock::MockClock;
    use pting_hal::{Clock, TargetId};
    use std::time::Duration;

    fn hit_at(at: Instant) -> HitEvent {
        HitEvent {
            target: TargetId(0),
            at,
            peak: 100,
        }
    }

    #[test]
    fn reaction_time_uses_fake_clock() {
        let clock = MockClock::new();
        let lit = clock.now();
        clock.advance(Duration::from_millis(312));
        assert_eq!(reaction_ms(lit, &hit_at(clock.now())), Some(312));
    }

    #[test]
    fn hit_before_light_is_rejected() {
        let clock = MockClock::new();
        let early = clock.now();
        clock.advance(Duration::from_millis(10));
        let lit = clock.now();
        assert_eq!(reaction_ms(lit, &hit_at(early)), None);
    }

    #[test]
    fn phases_are_distinct() {
        assert_ne!(Phase::Lobby, Phase::Finished);
    }
}

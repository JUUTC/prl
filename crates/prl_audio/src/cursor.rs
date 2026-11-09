use bevy::log::warn;
use prl_core::CoreOutcome;

/// One presentation consumer's position in core's ordered outcome stream
/// (Q-017 provisional).
///
/// Each consumer owns its cursor, so render, UI and audio never share or steal
/// outcomes. Sequences are monotonic across runs; only outcomes newer than
/// `last_seen` are returned, in order, so nothing is replayed. If core's bounded
/// retention has already dropped outcomes this consumer never saw, the gap is
/// skipped with one warning instead of replaying or stalling.
#[derive(Debug, Default)]
pub(crate) struct OutcomeCursor {
    last_seen: u64,
    gap_warned: bool,
}

impl OutcomeCursor {
    /// Newest outcomes not yet seen by this consumer; advances the cursor.
    pub(crate) fn take_new<'a>(
        &mut self,
        outcomes: &'a [CoreOutcome],
        consumer: &str,
    ) -> &'a [CoreOutcome] {
        let start = outcomes.partition_point(|outcome| outcome.sequence <= self.last_seen);
        let fresh = outcomes.get(start..).unwrap_or_default();
        if let Some(first) = fresh.first()
            && first.sequence > self.last_seen.saturating_add(1)
            && !self.gap_warned
        {
            warn!(
                "{consumer}: skipped outcomes {}..{} that left core retention before presentation",
                self.last_seen.saturating_add(1),
                first.sequence
            );
            self.gap_warned = true;
        }
        if let Some(last) = fresh.last() {
            self.last_seen = last.sequence;
        }
        fresh
    }

    /// Sequence of the newest outcome this consumer has processed.
    #[cfg(test)]
    pub(crate) const fn last_seen(&self) -> u64 {
        self.last_seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prl_core::OutcomeKind;

    fn outcome(sequence: u64) -> CoreOutcome {
        CoreOutcome {
            sequence,
            kind: OutcomeKind::TickAdvanced { tick: sequence },
        }
    }

    #[test]
    fn cursor_returns_each_outcome_once_in_order() {
        let mut cursor = OutcomeCursor::default();
        let log: Vec<_> = (1..=3).map(outcome).collect();
        assert_eq!(cursor.take_new(&log, "test").len(), 3);
        assert_eq!(cursor.take_new(&log, "test"), []);
        let log: Vec<_> = (1..=5).map(outcome).collect();
        let fresh = cursor.take_new(&log, "test");
        assert_eq!(fresh.iter().map(|o| o.sequence).collect::<Vec<_>>(), [4, 5]);
        assert_eq!(cursor.last_seen(), 5);
    }

    #[test]
    fn cursor_skips_outcomes_dropped_by_retention_without_replaying() {
        let mut cursor = OutcomeCursor::default();
        let log: Vec<_> = (1..=2).map(outcome).collect();
        cursor.take_new(&log, "test");
        let log: Vec<_> = (10..=12).map(outcome).collect();
        let fresh = cursor.take_new(&log, "test");
        assert_eq!(fresh.first().map(|o| o.sequence), Some(10));
        assert!(cursor.gap_warned);
        assert_eq!(cursor.take_new(&log, "test"), []);
    }
}

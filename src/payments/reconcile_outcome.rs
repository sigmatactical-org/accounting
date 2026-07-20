//! [`ReconcileOutcome`].

use serde::Serialize;

/// Result of a receipt reconcile sweep.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct ReconcileOutcome {
    /// Successful charges seen in the payments charge log.
    pub charges_seen: usize,
    /// Receipts newly recorded by this sweep.
    pub created: usize,
    /// Charges that already had a receipt.
    pub already_recorded: usize,
}

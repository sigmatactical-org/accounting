//! [`MoneySummary`].

/// Per-currency cash summary for the index header.
///
/// Money in sums receipts (refunds subtract); money out sums expenses plus
/// bills already marked paid; net is the difference. Each field is `None`
/// when there is nothing to sum.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MoneySummary {
    pub money_in: Option<String>,
    pub money_out: Option<String>,
    pub net: Option<String>,
}

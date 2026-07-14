//! [`BillRow`].

#[allow(unused_imports)]
use super::*;
use crate::model::Bill;

/// One rendered table row.
pub struct BillRow {
    pub bill: Bill,
    pub kind_label: String,
    pub status_label: String,
    pub total_display: String,
}

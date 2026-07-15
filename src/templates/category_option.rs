//! [`CategoryOption`].

#[allow(unused_imports)]
use super::*;

/// One `<option>` in the expense category select.
pub(crate) struct CategoryOption {
    pub(crate) value: &'static str,
    pub(crate) label: &'static str,
    pub(crate) selected: bool,
}

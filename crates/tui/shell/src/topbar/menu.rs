use crate::MenuItem;

/// A collection of [`MenuItem`]s grouped in one dropdown.
#[must_use]
#[derive(Debug, Clone)]
pub(crate) struct Menu<Action> {
    pub(crate) label: String,
    pub(crate) items: Vec<MenuItem<Action>>,
}

impl<Action> Menu<Action> {
    /// Makes new `Menu` from the given values.
    #[inline]
    pub fn new(label: impl Into<String>, items: Vec<MenuItem<Action>>) -> Self {
        Self {
            label: label.into(),
            items,
        }
    }
}

/// Item of dropdown menu.
#[must_use]
#[derive(Debug, Clone)]
pub struct MenuItem<Action> {
    pub label: String,
    pub action: Action,
}

impl<Action> MenuItem<Action> {
    /// Makes new `MenuItem` from the given values.
    #[inline]
    pub fn new(label: impl Into<String>, action: Action) -> Self {
        Self {
            label: label.into(),
            action,
        }
    }
}

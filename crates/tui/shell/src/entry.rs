use crate::{App, icon::Icon};

/// A desktop icon paired with a lazy constructor for the `App` it opens.
#[must_use]
pub struct Entry {
    name: String,
    icon: Icon,
    launch: Box<dyn Fn() -> Box<dyn App>>,
}

impl Entry {
    /// Makes new `Entry` from the given values.
    #[inline(always)]
    pub fn new(
        name: impl Into<String>,
        icon: Icon,
        launch: impl Fn() -> Box<dyn App> + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            icon,
            launch: Box::new(launch),
        }
    }

    /// This `Entry`'s display name.
    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// This `Entry`'s icon.
    #[inline(always)]
    pub const fn icon(&self) -> Icon {
        self.icon
    }

    /// Launches this `Entry`'s app.
    #[inline(always)]
    pub fn launch(&self) -> Box<dyn App> {
        (self.launch)()
    }
}

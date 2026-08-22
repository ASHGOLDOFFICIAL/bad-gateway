use i18n_embed::{
    I18nEmbedError, LanguageLoader,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "locales/"]
struct Localizations;

pub(crate) static LOADER: Lazy<FluentLanguageLoader> = Lazy::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    let languages = &[loader.fallback_language().clone()];

    loader
        .load_languages(&Localizations, languages)
        .expect("failed to load desktop crate's embedded locales");
    loader
});

/// Sets the language for this crate's own built-in strings.
pub fn set_language(lang: LanguageIdentifier) -> Result<(), I18nEmbedError> {
    LOADER.load_languages(&Localizations, &[lang])
}

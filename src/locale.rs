use std::{collections::{btree_map::Entry, BTreeMap, HashMap}, str::FromStr};

use crc32fast::Hasher;
use fluent_bundle::FluentValue;
use fluent_templates::Loader;
use list_files_macro::list_files;
use unic_langid::{langid, langids, LanguageIdentifier};

fluent_templates::static_loader! {
    static LOCALES = {
        // The directory of localisations and fluent resources.
        locales: "./locales",
        // The language to falback on if something is not present.
        fallback_language: "en",
        // Optional: A fluent resource that is shared with every locale.
        core_locales: "./locales/imgui.ftl",
        // Removes unicode isolating marks around argument. These marks make
        // sure the text will respect LTR or RTL rendering, but I think ImGui
        // doesn't support that.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

/// Contents of all English Fluent Files, with unknown sorting
const FLUENT_FILE_CONTENTS: &[&str] = &list_files!(include_str, "../locales/en/*.ftl");

/// Result of the concatenation of all English Fluent Files' contents
const FLUENT_UNITS: &str = konst::string::str_concat!(FLUENT_FILE_CONTENTS);

/// Contains a static array of words available to be used in translations,
/// parsed from the English Fluent units. Given a line `word = ...`, extracts
/// `word`
const WORDS: &[&str] = &konst::iter::collect_const!(&str =>
    konst::string::split(FLUENT_UNITS, "\n"),
        filter_map(|s| konst::string::split_once(s, "=")),
        map(|(name, _)| konst::string::trim(name)),
);

pub const LANGUAGES: &[(LanguageIdentifier, &str)] = &[
    (langid!("en"), "English"),
    (langid!("pt"), "Português"),
];

pub struct Locale {
    lang: LanguageIdentifier,
    translations: BTreeMap<&'static str, String>,
    formatted: BTreeMap<(&'static str, u32), String>,
}

/// Caching wrapper around [Fluent](https://projectfluent.org/)
impl Locale {
    pub fn new(lang: LanguageIdentifier) -> Self {
        Self {
            translations: Self::get_translations(&lang),
            formatted: BTreeMap::new(),
            lang,
        }
    }

    /// Updates the locale's language. This clears the whole cache and builds a
    /// new one
    pub fn set_lang(&mut self, lang: LanguageIdentifier) {
        self.translations = Self::get_translations(&lang);
        self.formatted.clear();
        self.lang = lang;
    }

    /// Looks up all unparametrized messages in the English Fluent Files and
    /// returns a map with the results
    fn get_translations(lang: &LanguageIdentifier) -> BTreeMap<&'static str, String> {
        WORDS
            .iter()
            // Fallback languages is English, and WORDS has been taken from the
            // English Fluent units. Therefore, lookup shouldn't panic
            .filter_map(|word| Some((*word, LOCALES.try_lookup(lang, word)?)))
            .collect()
    }

    /// Fetches an unparametrized message translation from the cache
    /// # Panics
    /// This method will panic only in debug mode when the message id does not
    /// exist. If running on release, it will return the message id
    pub fn get(&self, word: &'static str) -> &str {
        if cfg!(debug_assertions) {
            self.translations.get(word)
                .unwrap_or_else(|| panic!("Word could not be translated: {word}"))
        } else {
            self.translations.get(word)
                .map(String::as_str)
                .unwrap_or(word)
        }
    }

    /// Fetches/Caches a parametrized message translation. Both the message id
    /// and the arguments are used for indexing. In the case of the arguments,
    /// the pairs (argument_name, argument_value) are hashed together
    pub fn fmt(
        &mut self,
        word: &'static str,
        args: &HashMap<&'static str, FluentValue>
    ) -> &str {
        let mut hasher = Hasher::new();
        args.iter().for_each(|(k, v)| {
            hasher.update(k.as_bytes());
            match v {
                FluentValue::String(s) => hasher.update(s.as_bytes()),
                FluentValue::Number(n) => hasher.update(&n.value.to_be_bytes()),
                t => unimplemented!("We aren't hashing this type yet: {t:?}"),
            }
        });

        let hash = hasher.finalize();
        let key = (word, hash);

        if let Entry::Vacant(e) = self.formatted.entry(key) {
            let s = LOCALES.lookup_with_args(&self.lang, word, args);
            e.insert(s);
        }

        self.formatted.get(&key).unwrap()
    }

    pub fn current(&self) -> &LanguageIdentifier {
        &self.lang
    }
}

impl Default for Locale {
    fn default() -> Self {
        let lang = std::env::var("LANG")
            .ok()
            .as_deref()
            .and_then(|l| l.parse().ok())
            .unwrap_or(langid!("en"));

        Locale::new(lang)
    }
}
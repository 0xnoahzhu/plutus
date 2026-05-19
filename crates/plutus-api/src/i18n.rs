//! Locale handling for translatable agent outputs.
//!
//! Each translatable record has a `translations` column holding JSON of shape:
//!
//! ```json
//! { "zh-CN": { "headline": "...", "summary_md": "..." } }
//! ```
//!
//! Handlers parse `?locale=zh-CN`, then call [`apply_overrides`] on each Out
//! struct to swap in the localized strings server-side. Frontend gets a clean
//! payload — no locale-resolution logic needed on the client.
//!
//! `en` is the canonical/default locale; if `?locale=en` (or no override is
//! found for the requested locale), the base columns are returned unchanged.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Query extractor for `?locale=...`. Falls back to `en` when absent or empty.
#[derive(Debug, Deserialize)]
pub struct LocaleQuery {
    #[serde(default = "default_locale")]
    pub locale: String,
}

fn default_locale() -> String {
    "en".into()
}

/// The canonical locale — base columns hold this language's text.
pub const DEFAULT_LOCALE: &str = "en";

/// Apply per-locale field overrides to `out` in place. For the default locale
/// (or when `translations` is missing/empty/invalid JSON), `out` is unchanged.
///
/// The mechanism: serialize `out` to a JSON object, merge in the keys from
/// `translations[locale]`, and deserialize back. Each Out struct's field names
/// (after `#[serde(rename)]`, if any) act as the i18n keys — the agent must
/// use those exact keys in the JSON payload.
pub fn apply_overrides<T>(out: &mut T, translations: Option<&str>, locale: &str)
where
    T: Serialize + DeserializeOwned,
{
    if locale == DEFAULT_LOCALE {
        return;
    }
    let Some(json) = translations else { return };
    if json.trim().is_empty() {
        return;
    }
    let Ok(map) = serde_json::from_str::<serde_json::Value>(json) else {
        return;
    };
    let Some(overrides) = map.get(locale).and_then(|v| v.as_object()) else {
        return;
    };
    let Ok(mut value) = serde_json::to_value(&*out) else {
        return;
    };
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    for (k, v) in overrides {
        // Only override existing fields. The agent could ship junk keys but we
        // don't want them to leak into the response.
        if obj.contains_key(k) {
            obj.insert(k.clone(), v.clone());
        }
    }
    if let Ok(patched) = serde_json::from_value::<T>(value) {
        *out = patched;
    }
}

use isolang::Language;
use whatlang::{Lang, detect};

const MAX_DETECTION_SAMPLE_BYTES: usize = 32 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchTextConfig {
    English,
    Simple,
}

impl SearchTextConfig {
    pub const fn as_regconfig(self) -> &'static str {
        match self {
            Self::English => "english",
            Self::Simple => "simple",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchLanguageDecision {
    pub language: Option<String>,
    pub search_config: SearchTextConfig,
}

pub fn classify_search_language(
    declared_language: Option<&str>,
    text_parts: &[&str],
) -> SearchLanguageDecision {
    if let Some(language) = normalize_language_tag(declared_language) {
        let primary = language.split('-').next().unwrap_or_default();
        return SearchLanguageDecision {
            search_config: if matches!(primary, "en" | "eng") {
                SearchTextConfig::English
            } else {
                SearchTextConfig::Simple
            },
            language: Some(language),
        };
    }

    let sample = detection_sample(text_parts);
    let Some(info) = detect(&sample).filter(|info| info.is_reliable()) else {
        return SearchLanguageDecision {
            language: None,
            search_config: SearchTextConfig::English,
        };
    };

    let detected_language = info.lang();
    SearchLanguageDecision {
        language: normalize_language_tag(Some(detected_language.code())),
        search_config: if detected_language == Lang::Eng {
            SearchTextConfig::English
        } else {
            SearchTextConfig::Simple
        },
    }
}

pub fn normalize_language_tag(language: Option<&str>) -> Option<String> {
    let normalized = language?.trim().replace('_', "-").to_lowercase();
    let (primary, suffix) = normalized
        .split_once('-')
        .map_or((normalized.as_str(), None), |(primary, suffix)| {
            (primary, Some(suffix))
        });
    if primary.is_empty() || primary == "und" {
        return None;
    }

    let preferred_primary = match primary.len() {
        2 => Language::from_639_1(primary)
            .and_then(|language| language.to_639_1())
            .unwrap_or(primary),
        3 => Language::from_639_3(primary)
            .and_then(|language| language.to_639_1())
            .unwrap_or(primary),
        _ => primary,
    };

    Some(match suffix {
        Some(suffix) if !suffix.is_empty() => format!("{preferred_primary}-{suffix}"),
        _ => preferred_primary.to_string(),
    })
}

fn detection_sample(parts: &[&str]) -> String {
    let mut sample = String::with_capacity(MAX_DETECTION_SAMPLE_BYTES);
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if !sample.is_empty() && sample.len() < MAX_DETECTION_SAMPLE_BYTES {
            sample.push('\n');
        }
        let remaining = MAX_DETECTION_SAMPLE_BYTES.saturating_sub(sample.len());
        if remaining == 0 {
            break;
        }
        let end = part.floor_char_boundary(part.len().min(remaining));
        sample.push_str(&part[..end]);
    }
    sample
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_english_variants_use_english_configuration() {
        for (language, canonical) in [
            ("en", "en"),
            ("en-US", "en-us"),
            ("EN_us", "en-us"),
            ("eng", "en"),
        ] {
            let decision = classify_search_language(Some(language), &[FRENCH_TEXT]);
            assert_eq!(
                decision.search_config,
                SearchTextConfig::English,
                "{language}"
            );
            assert_eq!(decision.language.as_deref(), Some(canonical));
        }
    }

    #[test]
    fn declared_non_english_language_wins_over_english_text() {
        let decision = classify_search_language(Some("fr-FR"), &[ENGLISH_TEXT]);
        assert_eq!(decision.language.as_deref(), Some("fr-fr"));
        assert_eq!(decision.search_config, SearchTextConfig::Simple);

        let iso_639_3 = classify_search_language(Some("deu_DE"), &[ENGLISH_TEXT]);
        assert_eq!(iso_639_3.language.as_deref(), Some("de-de"));
    }

    #[test]
    fn missing_or_und_language_uses_reliable_detection() {
        for language in [None, Some(""), Some("und"), Some("UND-Latn")] {
            let english = classify_search_language(language, &[ENGLISH_TEXT]);
            assert_eq!(english.language.as_deref(), Some("en"));
            assert_eq!(english.search_config, SearchTextConfig::English);

            let french = classify_search_language(language, &[FRENCH_TEXT]);
            assert_eq!(french.language.as_deref(), Some("fr"));
            assert_eq!(french.search_config, SearchTextConfig::Simple);
        }
    }

    #[test]
    fn inconclusive_text_defaults_to_english_without_persisting_language() {
        let decision = classify_search_language(None, &["Hi"]);
        assert_eq!(decision.language, None);
        assert_eq!(decision.search_config, SearchTextConfig::English);
    }

    #[test]
    fn detection_sample_respects_utf8_byte_limit() {
        let sample = detection_sample(&[&"a".repeat(MAX_DETECTION_SAMPLE_BYTES - 1), "界"]);
        assert!(sample.len() <= MAX_DETECTION_SAMPLE_BYTES);
        assert!(sample.is_char_boundary(sample.len()));
        assert!(!sample.ends_with('界'));
    }

    const ENGLISH_TEXT: &str = "The researchers are running careful experiments across several systems. Their detailed observations explain how the components interact, why the results remain stable, and which conclusions follow from the evidence.";
    const FRENCH_TEXT: &str = "Les chercheurs mènent des expériences soigneuses sur plusieurs systèmes. Leurs observations détaillées expliquent comment les composants interagissent, pourquoi les résultats restent stables et quelles conclusions découlent des preuves.";
}

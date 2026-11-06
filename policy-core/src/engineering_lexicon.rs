// Copyright (c) 2026 AI-Bopomofo contributors. MIT License.
// Port of MixedInputCore/EngineeringLexicon.swift. See resources/AI-Bopomofo-LICENSE.txt.
use serde::Deserialize;
use std::collections::BTreeSet;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechnicalCandidate {
    pub text: String,
    pub annotation: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Catalog {
    version: u32,
    terms: Vec<Term>,
}

#[derive(Debug, Deserialize)]
struct Term {
    id: String,
    category: String,
    aliases: Vec<String>,
    outputs: Vec<String>,
    annotation: String,
    sources: Vec<String>,
}

#[derive(Debug, Default)]
pub struct EngineeringLexicon {
    terms: Vec<Term>,
}

fn safe_output(s: &str) -> bool {
    !s.is_empty() && s.encode_utf16().count() <= 128 && !s.chars().any(|c| {
        c.is_control() || matches!(c, '\u{00ad}' | '\u{061c}' | '\u{200b}'..='\u{200f}' |
            '\u{2028}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}')
    })
}

impl EngineeringLexicon {
    pub fn bundled() -> &'static Self {
        static BUNDLED: OnceLock<EngineeringLexicon> = OnceLock::new();
        BUNDLED.get_or_init(|| Self::from_json(include_str!("../resources/engineering-terms.json")))
    }

    pub fn from_json(data: &str) -> Self {
        if data.len() > 1_048_576 { return Self::default(); }
        let Ok(catalog) = serde_json::from_str::<Catalog>(data) else { return Self::default(); };
        let mut ids = BTreeSet::new();
        if catalog.version != 1 || catalog.terms.len() > 1000 || !catalog.terms.iter().all(|t| {
            !t.id.is_empty() && ids.insert(&t.id) && !t.category.is_empty()
                && !t.annotation.is_empty() && t.annotation.chars().count() <= 80
                && safe_output(&t.annotation) && !t.sources.is_empty()
                && (1..=16).contains(&t.aliases.len()) && t.aliases.iter().all(|a| {
                    (2..=64).contains(&a.len()) && a.bytes().all(|b| (33..=126).contains(&b))
                }) && (1..=8).contains(&t.outputs.len()) && t.outputs.iter().all(|s| safe_output(s))
        }) { return Self::default(); }
        Self { terms: catalog.terms }
    }

    /// Exact matches precede prefixes; two-character aliases only match exactly.
    /// Preserve reviewed Unicode/ASCII output order and never normalize the output.
    pub fn matches(&self, query: &str) -> (Vec<TechnicalCandidate>, Vec<TechnicalCandidate>) {
        if !(2..=64).contains(&query.len()) || !query.bytes().all(|b| (33..=126).contains(&b)) {
            return (vec![], vec![]);
        }
        let lower = query.to_ascii_lowercase();
        let mut exact = vec![];
        let mut prefix = vec![];
        for t in &self.terms {
            if t.aliases.iter().any(|a| a.eq_ignore_ascii_case(query)) { exact.push(t); }
            else if query.len() >= 3 && t.aliases.iter().any(|a| a.to_ascii_lowercase().starts_with(&lower)) { prefix.push(t); }
        }
        let collect = |mut terms: Vec<&Term>, full: bool| {
            terms.truncate(64);
            terms.sort_by_key(|t| !t.aliases.iter().any(|a| if full { a == query } else { a.starts_with(query) }));
            terms.into_iter().flat_map(|t| t.outputs.iter().map(|s| TechnicalCandidate {
                text: s.clone(), annotation: Some(t.annotation.clone()),
            })).collect()
        };
        (collect(exact, true), collect(prefix, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reviewed_catalog_loads_with_sources() {
        let l = EngineeringLexicon::bundled();
        assert_eq!(l.terms.len(), 104);
        assert!(l.terms.iter().all(|t| !t.sources.is_empty()));
    }
    #[test]
    fn unicode_ascii_order_and_annotation() {
        let (items, _) = EngineeringLexicon::bundled().matches("i2c");
        assert_eq!(items.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), ["I²C", "I2C"]);
        assert_eq!(items[0].annotation.as_deref(), Some("內部整合電路匯流排"));
    }
    #[test]
    fn water_is_an_exact_match_separate_from_peroxide() {
        let lexicon = EngineeringLexicon::bundled();
        let (water, longer) = lexicon.matches("H2O");
        assert_eq!(water.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), ["H₂O", "H2O"]);
        assert_eq!(water[0].annotation.as_deref(), Some("水"));
        assert!(longer.iter().any(|c| c.text == "H₂O₂"));
        let (peroxide, _) = lexicon.matches("H2O2");
        assert_eq!(peroxide.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), ["H₂O₂", "H2O2"]);
        assert_eq!(crate::technical_lexicon::TechnicalLexicon::default().completions("H2O")[0], "H₂O");
    }
    #[test]
    fn two_characters_require_exact_alias() {
        let (exact, prefix) = EngineeringLexicon::bundled().matches("i2");
        assert!(exact.is_empty() && prefix.is_empty());
        assert!(!EngineeringLexicon::bundled().matches("sio").1.is_empty());
    }
    #[test]
    fn reject_malformed_duplicate_or_unsafe_catalog() {
        for data in ["{}", r#"{"version":2,"terms":[]}"#] { assert!(EngineeringLexicon::from_json(data).terms.is_empty()); }
        let base = include_str!("../resources/engineering-terms.json");
        let mut value: serde_json::Value = serde_json::from_str(base).unwrap();
        value["terms"][1]["id"] = value["terms"][0]["id"].clone();
        assert!(EngineeringLexicon::from_json(&value.to_string()).terms.is_empty());
        let value = base.replace("I²C", "I\\u202eC");
        assert!(EngineeringLexicon::from_json(&value).terms.is_empty());
    }
}

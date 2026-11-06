// SPDX-License-Identifier: MIT
//! 專業技術詞彙庫與前綴補全 (TechnicalLexicon)
//! 移植自 edwardhome/AI-Bopomofo (TechnicalLexicon.swift)

use std::collections::BTreeSet;
use crate::engineering_lexicon::{EngineeringLexicon, TechnicalCandidate};
use std::path::PathBuf;
use crate::user_engineering::{UserEngineeringTerm, UserTermsError, validate_user_terms,
    parse_user_terms, MAX_USER_TERMS_BYTES, USER_TERMS_FILENAME};

pub const BUILTIN_TERMS: &[&str] = &[
    "Apple",
    "C++",
    "Codex",
    "CoreML",
    "FPGA",
    "Flask",
    "Git",
    "GitHub",
    "InputMethodKit",
    "MacBook",
    "McBopomofo",
    "OpenOCD",
    "PySide6",
    "Python",
    "RTX",
    "STM32",
    "Swift",
    "SystemVerilog",
    "VHDL",
    "Verilog",
    "Vivado",
    "Xcode",
    "branch",
    "module",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechnicalLexicon {
    words: Vec<String>,
    user_terms: Vec<UserEngineeringTerm>,
}

impl Default for TechnicalLexicon {
    fn default() -> Self {
        Self::new(&[])
    }
}

impl TechnicalLexicon {
    pub fn new(custom: &[String]) -> Self {
        let mut set = BTreeSet::new();
        for b in BUILTIN_TERMS {
            set.insert(b.to_string());
        }
        for w in custom.iter().take(5000) {
            if Self::is_valid(w) {
                set.insert(w.clone());
            }
        }
        Self {
            words: set.into_iter().collect(),
            user_terms: vec![],
        }
    }

    pub fn with_user_terms(mut self, terms: Vec<UserEngineeringTerm>) -> Result<Self, UserTermsError> {
        validate_user_terms(&terms)?;
        self.user_terms = terms;
        Ok(self)
    }

    /// 驗證單字格式：
    /// 1. 非空且不以 '#' 開頭
    /// 2. UTF-8 長度 <= 128
    /// 3. 不包含空白字元
    /// 4. 允許 ASCII 字母/數字及標準工程與自然科學 Unicode 拓撲字符（上下標、希臘字元等）
    pub fn is_valid(word: &str) -> bool {
        let trimmed = word.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.len() > 128 {
            return false;
        }
        if trimmed != word || trimmed.chars().any(|c| c.is_whitespace() || c.is_control()
            || matches!(c, '\u{00ad}' | '\u{061c}' | '\u{200b}'..='\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}')) {
            return false;
        }
        // 不能為純 ASCII 數字
        if trimmed.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        true
    }

    /// 解析文字內容（支援純每行一詞，或小麥注音/新酷音格式 `word _english_...`）
    pub fn parse_terms(content: &str) -> Vec<String> {
        let mut terms = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.is_empty() {
                continue;
            }
            let candidate = fields[0];
            if fields.len() >= 2 && fields[1].starts_with("_english_") {
                if Self::is_valid(candidate) {
                    terms.push(candidate.to_string());
                }
            } else if fields.len() == 1 {
                if Self::is_valid(candidate) {
                    terms.push(candidate.to_string());
                }
            }
            if terms.len() >= 5000 {
                break;
            }
        }
        terms
    }

    /// 取得 Windows %APPDATA%\ChewingDev\technical-terms.txt 檔案路徑
    pub fn default_user_lexicon_path() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(|appdata| {
            let mut p = PathBuf::from(appdata);
            p.push("ChewingDev");
            p.push("technical-terms.txt");
            p
        })
    }

    /// 從檔案載入自訂技術詞庫
    pub fn load_from_file_or_default() -> Self {
        let mut lexicon = Self::default();
        if let Some(path) = Self::default_user_lexicon_path() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let terms = Self::parse_terms(&content);
                    lexicon = Self::new(&terms);
                }
            }
            use std::io::Read;
            if let Ok(file) = std::fs::File::open(path.with_file_name(USER_TERMS_FILENAME)) {
                let mut bytes = Vec::new();
                if file.take((MAX_USER_TERMS_BYTES + 1) as u64).read_to_end(&mut bytes).is_ok() {
                    if let Ok(terms) = parse_user_terms(&bytes) { lexicon.user_terms = terms; }
                }
            }
        }
        lexicon
    }

    /// 前綴補全查詢：
    /// - 傳統詞庫前綴 >= 2；工程別名兩字元僅精確匹配，三字元起才展開
    /// - 不包含空白
    /// - 工程詞條使用已校對的 ASCII 別名，輸出 Unicode 不做相容性正規化
    /// - 排除與前綴完全一致之項目
    /// - 最多回傳 8 個結果
    pub fn completions(&self, prefix: &str) -> Vec<String> {
        self.candidate_items(prefix).into_iter().map(|c| c.text).collect()
    }

    pub fn candidate_items(&self, prefix: &str) -> Vec<TechnicalCandidate> {
        if !(2..=64).contains(&prefix.len()) || !prefix.bytes().all(|b| (33..=126).contains(&b)) {
            return vec![];
        }
        let lower = prefix.to_ascii_lowercase();
        let user_matches = |exact: bool| self.user_terms.iter().filter(move |term| {
            if exact { term.input.eq_ignore_ascii_case(prefix) }
            else { prefix.len() >= 3 && !term.input.eq_ignore_ascii_case(prefix)
                && term.input.to_ascii_lowercase().starts_with(&prefix.to_ascii_lowercase()) }
        }).map(|term| TechnicalCandidate { text: term.output.clone(),
            annotation: if term.annotation.is_empty() { None } else { Some(term.annotation.clone()) } });
        let (exact, engineering_prefix) = EngineeringLexicon::bundled().matches(prefix);
        let legacy = self.words.iter().filter(|w| w.to_ascii_lowercase().starts_with(&lower))
            .map(|w| TechnicalCandidate { text: w.clone(), annotation: None });
        let mut seen = BTreeSet::from([prefix.to_string()]);
        user_matches(true).chain(exact).chain(legacy).chain(user_matches(false)).chain(engineering_prefix)
            .filter(|c| seen.insert(c.text.clone())).take(8).collect()
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_aliases_keep_raw_first_and_preserve_legacy_completions() {
        let terms = vec![UserEngineeringTerm { input: "x2o".into(), output: "X₂O".into(), annotation: "合成測試".into() }];
        let lexicon = TechnicalLexicon::new(&["ExampleWord".into()]).with_user_terms(terms).unwrap();
        assert_eq!(lexicon.completions("X2O"), ["X₂O"]);
        assert!(lexicon.completions("x2").is_empty());
        assert!(lexicon.completions("Exa").contains(&"ExampleWord".into()));
        let composition = crate::shift_letter::ASCIIComposition::new("X2O", None, 0, &lexicon);
        assert_eq!(composition.display(), "X2O");
        assert!(TechnicalLexicon::default().completions("X2O").is_empty());
    }

    #[test]
    fn test_builtin_terms_loaded() {
        let lexicon = TechnicalLexicon::default();
        assert!(lexicon.words().contains(&"FPGA".to_string()));
        assert!(lexicon.words().contains(&"Vivado".to_string()));
        assert!(lexicon.words().contains(&"SystemVerilog".to_string()));
        assert!(lexicon.words().contains(&"PySide6".to_string()));
        assert!(lexicon.words().contains(&"STM32".to_string()));
        assert!(lexicon.words().contains(&"OpenOCD".to_string()));
        assert!(lexicon.words().contains(&"C++".to_string()));
    }

    #[test]
    fn test_validity() {
        assert!(TechnicalLexicon::is_valid("FPGA"));
        assert!(TechnicalLexicon::is_valid("C++"));
        assert!(TechnicalLexicon::is_valid("PySide6"));
        assert!(TechnicalLexicon::is_valid("I²C"));
        assert!(TechnicalLexicon::is_valid("SiO₂"));
        assert!(TechnicalLexicon::is_valid("LiCoO₂"));
        assert!(TechnicalLexicon::is_valid("Wi-Fi"));
        assert!(!TechnicalLexicon::is_valid(""));
        assert!(!TechnicalLexicon::is_valid("#comment"));
        assert!(!TechnicalLexicon::is_valid("12345")); // 純數字
        assert!(!TechnicalLexicon::is_valid("with space"));
        assert!(!TechnicalLexicon::is_valid("name\u{202e}txt"));
        assert!(!TechnicalLexicon::is_valid("name\0"));
    }

    #[test]
    fn test_parse_terms() {
        let content = "\
# This is a comment
FPGA
MyCustomLib
McPhrase _english_something
Invalid line with multiple items
12345
";
        let terms = TechnicalLexicon::parse_terms(content);
        assert_eq!(terms, vec!["FPGA", "MyCustomLib", "McPhrase"]);
    }

    #[test]
    fn test_completions() {
        let lexicon = TechnicalLexicon::default();
        let comps = lexicon.completions("fp");
        assert!(comps.contains(&"FPGA".to_string()));

        let comps = lexicon.completions("py");
        assert!(comps.contains(&"PySide6".to_string()));
        assert!(comps.contains(&"Python".to_string()));

        let comps = lexicon.completions("stm");
        assert_eq!(comps, vec!["STM32"]);

        // 單字元不觸發補全
        assert!(lexicon.completions("p").is_empty());

        // 完全相同不列出
        assert!(!lexicon.completions("FPGA").contains(&"FPGA".to_string()));

        // 學術拓撲符號補全驗證
        let comps_i2c = lexicon.completions("i2c");
        assert!(comps_i2c.contains(&"I²C".to_string()));

        let comps_sio = lexicon.completions("sio");
        assert!(comps_sio.contains(&"SiO₂".to_string()));

        let comps_wifi = lexicon.completions("wifi");
        assert!(comps_wifi.contains(&"Wi-Fi".to_string()));

        let comps_fin = lexicon.completions("fin");
        assert!(comps_fin.contains(&"FinFET".to_string()));
    }
}

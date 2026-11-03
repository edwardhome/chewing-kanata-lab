// SPDX-License-Identifier: MIT
//! 專業技術詞彙庫與前綴補全 (TechnicalLexicon)
//! 移植自 edwardhome/AI-Bopomofo (TechnicalLexicon.swift)

use std::collections::BTreeSet;
use std::path::PathBuf;

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
        }
    }

    /// 驗證單字格式：
    /// 1. 非空且不以 '#' 開頭
    /// 2. UTF-8 長度 <= 64
    /// 3. 至少包含一個英文字母 (A-Z 或 a-z)
    /// 4. 所有位元組均為 ASCII 33..=126 (可列印且無空白)
    pub fn is_valid(word: &str) -> bool {
        if word.is_empty() || word.starts_with('#') || word.len() > 64 {
            return false;
        }
        let bytes = word.as_bytes();
        let has_letter = bytes.iter().any(|&b| (b'A'..=b'Z').contains(&b) || (b'a'..=b'z').contains(&b));
        let all_printable_non_space = bytes.iter().all(|&b| (33..=126).contains(&b));
        has_letter && all_printable_non_space
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

    /// 取得 Windows %APPDATA%\Chewing\technical-terms.txt 檔案路徑
    pub fn default_user_lexicon_path() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(|appdata| {
            let mut p = PathBuf::from(appdata);
            p.push("Chewing");
            p.push("technical-terms.txt");
            p
        })
    }

    /// 從檔案載入自訂技術詞庫
    pub fn load_from_file_or_default() -> Self {
        if let Some(path) = Self::default_user_lexicon_path() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let terms = Self::parse_terms(&content);
                    return Self::new(&terms);
                }
            }
        }
        Self::default()
    }

    /// 前綴補全查詢：
    /// - 前綴字元數 >= 2
    /// - 不包含空白
    /// - 不分大小寫匹配
    /// - 排除與前綴完全一致之項目
    /// - 最多回傳 8 個結果
    pub fn completions(&self, prefix: &str) -> Vec<String> {
        if prefix.len() < 2 || prefix.chars().any(char::is_whitespace) {
            return Vec::new();
        }
        let lower_prefix = prefix.to_lowercase();
        self.words
            .iter()
            .filter(|w| {
                let lower_w = w.to_lowercase();
                lower_w.starts_with(&lower_prefix) && *w != prefix
            })
            .take(8)
            .cloned()
            .collect()
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(!TechnicalLexicon::is_valid(""));
        assert!(!TechnicalLexicon::is_valid("#comment"));
        assert!(!TechnicalLexicon::is_valid("12345")); // 無英文字母
        assert!(!TechnicalLexicon::is_valid("with space"));
        assert!(!TechnicalLexicon::is_valid("中文詞彙"));
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
    }
}

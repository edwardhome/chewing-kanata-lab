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
    // 跨領域工程與自然科學學術規範詞彙 (NAER / IEEE / IUPAP 標準)
    "I²C",
    "I³C",
    "MOSFET",
    "FinFET",
    "GAAFET",
    "CMOS",
    "BJT",
    "Op-Amp",
    "PCB",
    "PCBA",
    "S₁₁",
    "S₂₁",
    "Z₀",
    "RDS(on)",
    "Laser",
    "FBG",
    "FWHM",
    "TEM₀₀",
    "M²",
    "cm⁻¹",
    "iPhone",
    "macOS",
    "iOS",
    "eBay",
    "Wi-Fi",
    "Ethernet",
    "IPv4",
    "IPv6",
    "Wi-Fi 6",
    "Wi-Fi 7",
    "gRPC",
    "MQTT",
    "QoS",
    "PoE",
    "SiO₂",
    "Si₃N₄",
    "GaAs",
    "GaN",
    "InP",
    "SiC",
    "Al₂O₃",
    "HfO₂",
    "ZrO₂",
    "TiO₂",
    "Ta₂O₅",
    "ITO",
    "FTO",
    "MoS₂",
    "WSe₂",
    "h-BN",
    "LiCoO₂",
    "LiFePO₄",
    "LiMn₂O₄",
    "Li₄Ti₅O₁₂",
    "YBCO",
    "MAPbI₃",
    "SiH₄",
    "NH₃",
    "SF₆",
    "CF₄",
    "HF",
    "H₂O₂",
    "Li⁺",
    "OH⁻",
    "H⁺",
    "H₃O⁺",
    "SO₄²⁻",
    "CO₃²⁻",
    "Cu²⁺",
    "DLP",
    "HUD",
    "LiDAR",
    "MicroLED",
    "OLED",
    "QD-OLED",
    "VCSEL",
    "DFB",
    "DBR",
    "HEMT",
    "IGBT",
    "MEMS",
    "ASIC",
    "DAC",
    "ADC",
    "PLL",
    "VCO",
    "LDO",
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
    /// 2. UTF-8 長度 <= 128
    /// 3. 不包含空白字元
    /// 4. 允許 ASCII 字母/數字及標準工程與自然科學 Unicode 拓撲字符（上下標、希臘字元等）
    pub fn is_valid(word: &str) -> bool {
        let trimmed = word.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.len() > 128 {
            return false;
        }
        if trimmed.chars().any(char::is_whitespace) {
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
    /// - 支援大小寫不敏感匹配與 Unicode 拓撲字符歸一化匹配（如 i2c 匹配 I²C、sio2 匹配 SiO₂、wifi 匹配 Wi-Fi）
    /// - 排除與前綴完全一致之項目
    /// - 最多回傳 8 個結果
    pub fn completions(&self, prefix: &str) -> Vec<String> {
        if prefix.len() < 2 || prefix.chars().any(char::is_whitespace) {
            return Vec::new();
        }
        let lower_prefix = prefix.to_lowercase();
        let norm_prefix = normalize_key(prefix);
        let clean_prefix: String = norm_prefix.chars().filter(|c| c.is_alphanumeric()).collect();

        self.words
            .iter()
            .filter(|w| {
                if *w == prefix {
                    return false;
                }
                let lower_w = w.to_lowercase();
                if lower_w.starts_with(&lower_prefix) {
                    return true;
                }
                let norm_w = normalize_key(w);
                if norm_w.starts_with(&norm_prefix) {
                    return true;
                }
                if !clean_prefix.is_empty() {
                    let clean_w: String = norm_w.chars().filter(|c| c.is_alphanumeric()).collect();
                    if clean_w.starts_with(&clean_prefix) {
                        return true;
                    }
                }
                false
            })
            .take(8)
            .cloned()
            .collect()
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }
}

/// 將特殊上下標與拓撲字符映射為標準 ASCII 鍵，便於快速補全查詢
fn normalize_symbol(c: char) -> String {
    match c {
        '⁰' => "0".to_string(),
        '¹' => "1".to_string(),
        '²' => "2".to_string(),
        '³' => "3".to_string(),
        '⁴' => "4".to_string(),
        '⁵' => "5".to_string(),
        '⁶' => "6".to_string(),
        '⁷' => "7".to_string(),
        '⁸' => "8".to_string(),
        '⁹' => "9".to_string(),
        '₀' => "0".to_string(),
        '₁' => "1".to_string(),
        '₂' => "2".to_string(),
        '₃' => "3".to_string(),
        '₄' => "4".to_string(),
        '₅' => "5".to_string(),
        '₆' => "6".to_string(),
        '₇' => "7".to_string(),
        '₈' => "8".to_string(),
        '₉' => "9".to_string(),
        '⁺' => "+".to_string(),
        '⁻' => "-".to_string(),
        'μ' | 'µ' => "u".to_string(),
        'ħ' => "h".to_string(),
        'Å' => "A".to_string(),
        '℃' => "C".to_string(),
        other => other.to_lowercase().to_string(),
    }
}

fn normalize_key(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    for c in text.chars() {
        s.push_str(&normalize_symbol(c));
    }
    s
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
        assert!(TechnicalLexicon::is_valid("I²C"));
        assert!(TechnicalLexicon::is_valid("SiO₂"));
        assert!(TechnicalLexicon::is_valid("LiCoO₂"));
        assert!(TechnicalLexicon::is_valid("Wi-Fi"));
        assert!(!TechnicalLexicon::is_valid(""));
        assert!(!TechnicalLexicon::is_valid("#comment"));
        assert!(!TechnicalLexicon::is_valid("12345")); // 純數字
        assert!(!TechnicalLexicon::is_valid("with space"));
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
        let comps_i2c = lexicon.completions("i2");
        assert!(comps_i2c.contains(&"I²C".to_string()));

        let comps_sio = lexicon.completions("sio");
        assert!(comps_sio.contains(&"SiO₂".to_string()));

        let comps_wifi = lexicon.completions("wifi");
        assert!(comps_wifi.contains(&"Wi-Fi".to_string()));

        let comps_fin = lexicon.completions("fin");
        assert!(comps_fin.contains(&"FinFET".to_string()));
    }
}

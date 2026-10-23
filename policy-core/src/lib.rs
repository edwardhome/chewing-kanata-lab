//! Cross-platform character output policy library for Chewing + Kanata
//! 
//! 本模組為純邏輯層，不包含平台特定 API (如 Win32/TSF 或 Fcitx)，亦不包含網路呼叫。

pub mod capi;
pub mod grammar_corrector;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub enum PolicyMode {
    #[default]
    Legacy = 0,
    AsciiFirst = 1,
    FormalZh = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub enum EditorMode {
    #[default]
    Unknown = 0,
    Normal = 1,
    Insert = 2,
    Visual = 3,
    Command = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct PolicyContext {
    pub mode: PolicyMode,
    pub editor_mode: EditorMode,
    pub has_active_preedit: bool,
    pub is_secure_field: bool,
    pub protect_urls_and_emails: bool,
}

impl PolicyContext {
    pub fn default_legacy() -> Self {
        Self {
            mode: PolicyMode::Legacy,
            editor_mode: EditorMode::Unknown,
            has_active_preedit: false,
            is_secure_field: false,
            protect_urls_and_emails: true,
        }
    }

    pub fn new(mode: PolicyMode) -> Self {
        let mut ctx = Self::default_legacy();
        ctx.mode = mode;
        ctx
    }
}

/// 判斷特定 ASCII 字元是否屬於常見的 Markdown / 程式語法關鍵標點
pub fn is_syntax_punctuation(ch: char) -> bool {
    // 排除標準注音鍵位使用的字元：'-' (ㄦ), '/' (ㄥ), ';' (ㄤ), ',' (ㄝ), '.' (ㄡ)
    // 嚴格保留寫程式、Markdown、終端機必備之結構標點
    matches!(
        ch,
        '[' | ']' | '{' | '}' | '(' | ')' | ':' | '<' | '>' | '`' | '_' | '|' | '=' | '+' | '*' | '\\' | '"' | '\'' | '!' | '@' | '#' | '$' | '%' | '^' | '&' | '~' | '?'
    )
}

/// 判斷前端是否應在按鍵分流階段將該按鍵 Bypass 放行給應用程式
pub fn should_bypass_key(ascii_key: char, ctx: &PolicyContext) -> bool {
    // 1. 安全欄位一律放行
    if ctx.is_secure_field {
        return true;
    }

    // 2. Vim Normal / Visual / Command 模式一律放行
    if matches!(
        ctx.editor_mode,
        EditorMode::Normal | EditorMode::Visual | EditorMode::Command
    ) {
        return true;
    }

    // 3. Legacy 模式完全維持上游新酷音行為，不放行
    if ctx.mode == PolicyMode::Legacy {
        return false;
    }

    // 4. Formal ZH 模式由輸入法提供全形標點，不放行
    if ctx.mode == PolicyMode::FormalZh {
        return false;
    }

    // 5. ASCII_FIRST 模式：組字緩衝區為空且按鍵為語法標點時直接放行
    if ctx.mode == PolicyMode::AsciiFirst && !ctx.has_active_preedit && is_syntax_punctuation(ascii_key) {
        return true;
    }

    false
}

/// 針對即將 commit 的字串進行策略轉換
pub fn transform_commit<'a>(input: &'a str, ctx: &PolicyContext) -> &'a str {
    if ctx.is_secure_field || ctx.mode == PolicyMode::Legacy || ctx.mode == PolicyMode::FormalZh {
        return input;
    }

    if ctx.mode == PolicyMode::AsciiFirst {
        match input {
            "：" => return ":",
            "「" => return "[",
            "」" => return "]",
            "『" => return "{",
            "』" => return "}",
            "（" => return "(",
            "）" => return ")",
            "；" => return ";",
            "—" => return "_",
            "～" => return "~",
            "！" => return "!",
            "？" => return "?",
            _ => {}
        }
    }

    input
}

/// AI 候選字重排序 (Reranker) Stub
pub fn rerank_candidates_stub(candidates: &[i32]) -> Vec<i32> {
    candidates.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_default_context() {
        let ctx = PolicyContext::default_legacy();
        assert_eq!(ctx.mode, PolicyMode::Legacy);
        assert!(!ctx.has_active_preedit);
    }

    #[test]
    fn test_rust_ascii_first_bypass() {
        let mut ctx = PolicyContext::new(PolicyMode::AsciiFirst);
        assert!(should_bypass_key(':', &ctx));
        assert!(should_bypass_key('[', &ctx));

        ctx.has_active_preedit = true;
        assert!(!should_bypass_key(':', &ctx));
    }

    #[test]
    fn test_rust_transform_commit() {
        let ctx = PolicyContext::new(PolicyMode::AsciiFirst);
        assert_eq!(transform_commit("：", &ctx), ":");
        assert_eq!(transform_commit("新酷音", &ctx), "新酷音");
    }
}

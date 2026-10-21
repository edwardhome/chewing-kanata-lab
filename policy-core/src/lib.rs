//! Cross-platform character output policy library for Chewing + Kanata
//! 
//! 本模組為純邏輯層，不包含平台特定 API (如 Win32/TSF 或 Fcitx)，亦不包含網路呼叫。

pub mod capi;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub enum PolicyProfile {
    #[default]
    Default = 0,
    Markdown = 1,
    Code = 2,
    Formal = 3,
    RawCommand = 4,
    ManualFullwidth = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct PolicyContext {
    pub profile: PolicyProfile,
    pub has_active_preedit: bool,
    pub is_secure_field: bool,
}

/// 判斷特定 ASCII 字元是否屬於常見的 Markdown / 程式語法關鍵標點
pub fn is_ascii_syntax_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '[' | ']' | '{' | '}' | '(' | ')' | ':' | ';' | '<' | '>' | '`' | '_' | '|' | '=' | '+' | '-' | '/' | '\\' | '"' | '\'' | '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '~' | '?'
    )
}

/// 判斷按鍵是否應直接 Bypass 放行給應用程式
pub fn should_bypass_key(ascii_key: char, ctx: &PolicyContext) -> bool {
    // 0. 安全欄位一律放行
    if ctx.is_secure_field {
        return true;
    }

    // 1. 命令模式一律放行
    if ctx.profile == PolicyProfile::RawCommand {
        return true;
    }

    // 2. 手動全形模式不放行 (交給上游組字全形邏輯)
    if ctx.profile == PolicyProfile::ManualFullwidth {
        return false;
    }

    // 3. 若無正在組字之注音緩衝區，在 Code 或 Markdown 模式下，語法符號直接放行
    if !ctx.has_active_preedit {
        match ctx.profile {
            PolicyProfile::Code | PolicyProfile::Markdown => {
                if is_ascii_syntax_punctuation(ascii_key) {
                    return true;
                }
            }
            PolicyProfile::Default => {
                // Default 模式下，高頻程式括號冒號亦建議放行
                if matches!(ascii_key, ':' | '[' | ']' | '{' | '}' | '`' | '_') {
                    return true;
                }
            }
            PolicyProfile::Formal => {
                // 正式公文模式保留中文標點，不主動放行
                return false;
            }
            _ => {}
        }
    }

    false
}

/// 針對即將 commit 的字串進行策略轉換
pub fn transform_commit<'a>(input: &'a str, ctx: &PolicyContext) -> &'a str {
    if ctx.is_secure_field || ctx.profile == PolicyProfile::RawCommand {
        return input;
    }

    // 若非全形模式，且在 Markdown/Code 模式下收到單一全形冒號，轉回 ASCII 冒號
    if matches!(ctx.profile, PolicyProfile::Markdown | PolicyProfile::Code) {
        if input == "：" {
            return ":";
        }
    }

    input
}

//! Windows TSF Policy Adapter Bridge
//! 
//! 將 Windows TSF 的鍵盤事件、狀態與組字提交對接至 chewing-policy-core。

use chewing_policy_core::{
    PolicyContext, PolicyMode, EditorMode, should_bypass_key, transform_commit
};

#[derive(Debug, Clone)]
pub struct TsfPolicyBridge {
    context: PolicyContext,
}

impl TsfPolicyBridge {
    pub fn new(mode: PolicyMode) -> Self {
        Self {
            context: PolicyContext::new(mode),
        }
    }

    pub fn set_mode(&mut self, mode: PolicyMode) {
        self.context.mode = mode;
    }

    pub fn mode(&self) -> PolicyMode {
        self.context.mode
    }

    pub fn set_preedit_active(&mut self, active: bool) {
        self.context.has_active_preedit = active;
    }

    pub fn set_secure_field(&mut self, secure: bool) {
        self.context.is_secure_field = secure;
    }

    pub fn set_editor_mode(&mut self, mode: EditorMode) {
        self.context.editor_mode = mode;
    }

    /// 在 TSF `on_test_keydown` 呼叫：判定是否應將此 ASCII 符號鍵放行給目標應用程式 (Bypass)
    pub fn should_bypass_key(&self, ascii_char: char) -> bool {
        should_bypass_key(ascii_char, &self.context)
    }

    /// 在 TSF `insert_text` 或 `update_preedit` 提交前呼叫：轉換 commit 字串
    pub fn transform_commit<'a>(&self, raw_commit: &'a str) -> &'a str {
        transform_commit(raw_commit, &self.context)
    }
}

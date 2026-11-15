// SPDX-License-Identifier: MIT
//! Windows 應用程式模式規則 (App Input Rules)
//!
//! 規範與審查修正：
//! 1. `normalize_canonical_app_id`：
//!    - 必須跨平台可編譯且行為一致（避免依賴平臺特定的 std::path::Prefix，在非 Windows 也能正確處理 Windows 磁碟路徑）。
//!    - 僅支援本機磁碟路徑（`C:\...`），拒絕 UNC（`\\server\share\...`）。
//!    - 限制路徑長度（1..=260，或 <= 1024 字元）與 display_name 長度（<= 128 字元）。
//!    - 拒絕控制字元。
//!    - 使用 Unicode lowercase (`to_lowercase()`)。
//!    - 嚴格反斜線格式化，去除相對路徑元素（`.`, `..`）。
//!    - 若文件內任何一條規則無效或重複，整份文件 fail-closed 回退至空規則。
//! 2. 設定變更與快照（Next-visit-only）：
//!    - 當前存取（Current Visit）中保持該次存取開始時的規則快照與啟用狀態。
//!    - `update_desired_config` 僅更新 `desired_config`，不影響目前已經在進行中的 visit。
//!    - 只有在實際進入不同應用程式（或全新 visit）時，才從 `desired_config` 拍下新的 visit 快照。
//! 3. 世代（Generation）與多 TSF 實例共享：
//!    - `visit_generation` 在實際 App leave/return 時遞增。
//!    - `poll_pending_apply` 與 `on_user_manual_toggle` 驗證預期的 `visit_generation`。
//!    - 同一 visit 內的使用者手動切換（Manual override）會作為目前 visit 的持續目標（Current-visit target），
//!      即使其他控制項/分頁/新 TSF 實例在同一 visit 期間取得焦點，也能取到該手動覆蓋值，而非忘記或重複重設。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 最大支援 JSON 字節數：1 MiB
pub const MAX_RULES_JSON_BYTES: usize = 1_048_576;
/// 最大支援規則數量：256 筆
pub const MAX_RULES_COUNT: usize = 256;
/// 最大路徑字元長度
pub const MAX_PATH_CHARS: usize = 1024;
/// 最大顯示名稱字元長度
pub const MAX_DISPLAY_NAME_CHARS: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleMode {
    English,
    Bopomofo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppInputRule {
    pub app_id: String,
    pub display_name: String,
    pub mode: RuleMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppInputRulesDocument {
    pub version: u32,
    pub rules: Vec<AppInputRule>,
}

impl Default for AppInputRulesDocument {
    fn default() -> Self {
        Self {
            version: 1,
            rules: Vec::new(),
        }
    }
}

/// 跨平台 Windows 本機執行檔絕對路徑正規化
///
/// 規則：
/// - 僅支援本機磁碟絕對路徑：`[A-Za-z]:\` 開頭
/// - 拒絕 UNC 路徑（`\\...`）與相對路徑
/// - 長度限制：1..=MAX_PATH_CHARS
/// - 拒絕含控制字元
/// - Unicode 小寫化，統一反斜線 `\`
/// - 消除 `.` 與合法消除 `..`，不可跳出磁碟機根目錄
pub fn normalize_canonical_app_id(raw_path: &str) -> Option<String> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty()
        || trimmed.len() > MAX_PATH_CHARS
        || trimmed.chars().any(char::is_control)
    {
        return None;
    }

    // 拒絕 UNC
    if trimmed.starts_with(r"\\") || trimmed.starts_with("//") {
        return None;
    }

    // 檢查磁碟機代號開頭，例如 "C:" 或 "c:"
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() < 3 {
        return None;
    }
    let drive = chars[0];
    if !drive.is_ascii_alphabetic() || chars[1] != ':' || (chars[2] != '\\' && chars[2] != '/') {
        return None;
    }

    let drive_lower = drive.to_ascii_lowercase();
    let rest = &trimmed[2..];

    // 分解 path segments，統一處理 / 與 \
    let mut segments = Vec::new();
    for seg in rest.split(['\\', '/']) {
        let seg_trim = seg.trim();
        if seg_trim.is_empty() || seg_trim == "." {
            continue;
        }
        if seg_trim == ".." {
            if segments.pop().is_none() {
                // 跳出根目錄，無效路徑
                return None;
            }
        } else {
            // 檔名/目錄名不能含非法字元或為空
            if seg_trim.chars().any(|c| c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|') {
                return None;
            }
            segments.push(seg_trim.to_lowercase());
        }
    }

    if segments.is_empty() {
        return None;
    }

    Some(format!(r"{}:\{}", drive_lower, segments.join(r"\")))
}

/// 解析並驗證 JSON 規則字串
///
/// 審查要求：invalid entries fail whole document（若有任何無效條目或重複，整份文件回退為空規則）
pub fn parse_and_validate_rules(json_str: &str) -> Vec<AppInputRule> {
    if json_str.len() > MAX_RULES_JSON_BYTES {
        return Vec::new();
    }

    let trimmed = json_str.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let doc: AppInputRulesDocument = match serde_json::from_str(trimmed) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    if doc.version != 1 || doc.rules.len() > MAX_RULES_COUNT {
        return Vec::new();
    }

    let mut valid_rules = Vec::with_capacity(doc.rules.len());
    let mut seen_ids = HashSet::with_capacity(doc.rules.len());

    for rule in doc.rules {
        let name_trimmed = rule.display_name.trim();
        if name_trimmed.is_empty()
            || name_trimmed.len() > MAX_DISPLAY_NAME_CHARS
            || name_trimmed.chars().any(char::is_control)
        {
            return Vec::new();
        }

        let norm_id = match normalize_canonical_app_id(&rule.app_id) {
            Some(id) => id,
            None => return Vec::new(), // invalid entry fails whole document
        };

        if !seen_ids.insert(norm_id.clone()) {
            // 重複 app_id，fail whole document
            return Vec::new();
        }

        valid_rules.push(AppInputRule {
            app_id: norm_id,
            display_name: name_trimmed.to_string(),
            mode: rule.mode,
        });
    }

    valid_rules
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInputRulesConfigSnapshot {
    pub enabled: bool,
    pub rules: HashMap<String, RuleMode>,
}

impl Default for AppInputRulesConfigSnapshot {
    fn default() -> Self {
        Self {
            enabled: false,
            rules: HashMap::new(),
        }
    }
}

/// 應用程式存取階段與狀態機
#[derive(Debug, Clone)]
pub struct AppInputRulesTracker {
    /// 下一次 visit 時預計採用的設定
    desired_config: AppInputRulesConfigSnapshot,

    /// 當前 visit 鎖定的設定快照
    active_visit_config: AppInputRulesConfigSnapshot,

    /// 當前活動的應用程式路徑
    current_app_id: Option<String>,

    /// 當前 visit 世代，每次離開並進入不同 App（或返回 App）時遞增
    visit_generation: u64,

    /// 當前 visit 的手動切換目標（若有手動切換則設定，該 visit 內的所有控制項與新 TSF 實例均使用此目標）
    current_visit_override: Option<RuleMode>,

    /// 待套用的規則預設模式（首次進入且尚未完成套用）
    pending_rule_apply: Option<(u64, String, RuleMode)>,
}

impl Default for AppInputRulesTracker {
    fn default() -> Self {
        Self::new(false, &[])
    }
}

impl AppInputRulesTracker {
    pub fn new(enabled: bool, rules: &[AppInputRule]) -> Self {
        let mut map = HashMap::new();
        for r in rules {
            map.insert(r.app_id.clone(), r.mode);
        }
        let desired = AppInputRulesConfigSnapshot {
            enabled,
            rules: map,
        };
        Self {
            desired_config: desired.clone(),
            active_visit_config: desired,
            current_app_id: None,
            visit_generation: 0,
            current_visit_override: None,
            pending_rule_apply: None,
        }
    }

    /// 設定頁修改設定時更新期望設定（Next-visit-only：不影響目前正在進行中的 visit）
    pub fn update_desired_config(&mut self, enabled: bool, rules: &[AppInputRule]) {
        let mut map = HashMap::new();
        for r in rules {
            map.insert(r.app_id.clone(), r.mode);
        }
        self.desired_config = AppInputRulesConfigSnapshot {
            enabled,
            rules: map,
        };
    }

    /// 當前景應用程式變更時觸發（傳入由 Windows 解析的正規化執行檔路徑）
    ///
    /// 回傳 `(visit_generation, target_mode)`：
    /// - 若是進入新 App 或離開後返回：推進 `visit_generation`，套用 `desired_config` 快照，
    ///   清除之前的 `current_visit_override`，並設定 `pending_rule_apply`。
    /// - 若是同一 App 內的焦點切換：保持原 `visit_generation`，回傳當前 visit 目標模式（若已有手動設定則為手動設定，否則為 pending 預設）。
    pub fn on_foreground_app_changed(&mut self, new_app_id: Option<&str>) -> Option<(u64, RuleMode)> {
        let norm_new = new_app_id.and_then(normalize_canonical_app_id);

        if norm_new.is_some() && norm_new == self.current_app_id {
            // 同一 App 存取（Visit）中：
            let current = self.current_visit_target();
            return current.map(|mode| (self.visit_generation, mode));
        }

        // 實際 App leave / return：推進 visit generation
        self.current_app_id = norm_new.clone();
        self.visit_generation = self.visit_generation.wrapping_add(1);
        self.active_visit_config = self.desired_config.clone();
        self.current_visit_override = None;
        self.pending_rule_apply = None;

        let app_id = norm_new?;
        if !self.active_visit_config.enabled {
            return None;
        }

        let rule_mode = self.active_visit_config.rules.get(&app_id).copied()?;

        // 新 visit 排程套用
        self.pending_rule_apply = Some((self.visit_generation, app_id, rule_mode));
        Some((self.visit_generation, rule_mode))
    }

    /// 取得目前 visit 的目標模式（供新啟動或重新活化的 TSF 實例獲取同 visit 的持續狀態）
    pub fn current_visit_target(&self) -> Option<RuleMode> {
        if !self.active_visit_config.enabled {
            return None;
        }
        let app_id = self.current_app_id.as_ref()?;
        if let Some(manual) = self.current_visit_override {
            return Some(manual);
        }
        self.active_visit_config.rules.get(app_id).copied()
    }

    /// 安全邊界輪詢並套用待處理的規則
    ///
    /// 審查要求：必須校驗 `expected_generation` 與目前實際所在的 App ID，並在有未完成組字時延後
    pub fn poll_pending_apply(
        &mut self,
        expected_generation: u64,
        current_app_id: Option<&str>,
        has_active_composition: bool,
    ) -> Option<RuleMode> {
        if !self.active_visit_config.enabled || has_active_composition {
            return None;
        }

        let norm_current = current_app_id.and_then(normalize_canonical_app_id)?;
        if let Some((gen, ref pending_app, default_mode)) = self.pending_rule_apply {
            if gen == expected_generation
                && gen == self.visit_generation
                && pending_app == &norm_current
                && self.current_app_id.as_deref() == Some(&norm_current)
            {
                self.pending_rule_apply = None;
                let target = self.current_visit_override.unwrap_or(default_mode);
                return Some(target);
            }
        }

        None
    }

    /// 使用者手動切換中英文模式時呼叫（例如按 Shift 切換）
    ///
    /// 審查要求：校驗 `expected_generation`，取消該次 visit 待處理的預設套用，
    /// 並設定 `current_visit_override` 供同 visit 內的所有 TSF 實例持續使用。
    pub fn on_user_manual_toggle(
        &mut self,
        expected_generation: u64,
        active_app_id: Option<&str>,
        new_mode: RuleMode,
    ) -> bool {
        if expected_generation != self.visit_generation {
            return false;
        }
        let norm_app = active_app_id.and_then(normalize_canonical_app_id);
        if norm_app.is_some() && norm_app != self.current_app_id {
            return false;
        }

        self.pending_rule_apply = None;
        self.current_visit_override = Some(new_mode);
        true
    }

    /// 目前的應用程式路徑
    pub fn current_app_id(&self) -> Option<&str> {
        self.current_app_id.as_deref()
    }

    /// 目前的 visit 世代
    pub fn visit_generation(&self) -> u64 {
        self.visit_generation
    }

    /// 目前 visit 是否啟用了規則
    pub fn is_active_visit_enabled(&self) -> bool {
        self.active_visit_config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_canonical_app_id_cross_platform() {
        assert_eq!(
            normalize_canonical_app_id(r"C:\Windows\System32\Notepad.exe"),
            Some(r"c:\windows\system32\notepad.exe".to_string())
        );
        assert_eq!(
            normalize_canonical_app_id("d:/apps/sub/../editor.EXE"),
            Some(r"d:\apps\editor.exe".to_string())
        );
        assert_eq!(
            normalize_canonical_app_id(r"c:\windows\.\system32\calc.exe"),
            Some(r"c:\windows\system32\calc.exe".to_string())
        );
    }

    #[test]
    fn test_normalize_rejects_unc_and_invalid() {
        // Reject UNC
        assert_eq!(normalize_canonical_app_id(r"\\Server\Share\app.exe"), None);
        assert_eq!(normalize_canonical_app_id("//Server/Share/app.exe"), None);

        // Reject relative paths
        assert_eq!(normalize_canonical_app_id("notepad.exe"), None);
        assert_eq!(normalize_canonical_app_id("System32/notepad.exe"), None);

        // Reject escape above root
        assert_eq!(normalize_canonical_app_id(r"C:\..\notepad.exe"), None);

        // Reject empty or invalid drive
        assert_eq!(normalize_canonical_app_id(""), None);
        assert_eq!(normalize_canonical_app_id("C:"), None);
        assert_eq!(normalize_canonical_app_id("C:\\"), None);
        assert_eq!(normalize_canonical_app_id("1:\\test.exe"), None);
    }

    #[test]
    fn test_parse_rules_fail_whole_document_on_invalid() {
        let valid_json = r#"{
            "version": 1,
            "rules": [
                {
                    "app_id": "C:\\Windows\\notepad.exe",
                    "display_name": "Notepad",
                    "mode": "english"
                }
            ]
        }"#;
        let rules = parse_and_validate_rules(valid_json);
        assert_eq!(rules.len(), 1);

        // UNC entry fails whole document
        let unc_json = r#"{
            "version": 1,
            "rules": [
                {
                    "app_id": "C:\\Windows\\notepad.exe",
                    "display_name": "Notepad",
                    "mode": "english"
                },
                {
                    "app_id": "\\\\server\\share\\tool.exe",
                    "display_name": "UNC Tool",
                    "mode": "english"
                }
            ]
        }"#;
        assert_eq!(parse_and_validate_rules(unc_json), vec![]);

        // Duplicate app_id fails whole document
        let dup_json = r#"{
            "version": 1,
            "rules": [
                {
                    "app_id": "C:\\Windows\\notepad.exe",
                    "display_name": "Notepad 1",
                    "mode": "english"
                },
                {
                    "app_id": "c:\\windows\\NOTEPAD.EXE",
                    "display_name": "Notepad 2",
                    "mode": "bopomofo"
                }
            ]
        }"#;
        assert_eq!(parse_and_validate_rules(dup_json), vec![]);

        // Bad display_name fails whole document
        let bad_name = r#"{
            "version": 1,
            "rules": [
                {
                    "app_id": "C:\\Windows\\notepad.exe",
                    "display_name": "   ",
                    "mode": "english"
                }
            ]
        }"#;
        assert_eq!(parse_and_validate_rules(bad_name), vec![]);
    }

    #[test]
    fn test_next_visit_only_setting_changes() {
        let rule1 = AppInputRule {
            app_id: r"c:\windows\notepad.exe".to_string(),
            display_name: "Notepad".to_string(),
            mode: RuleMode::English,
        };
        let mut tracker = AppInputRulesTracker::new(true, &[rule1]);

        // Visit 1: 进入 Notepad
        let (gen1, mode1) = tracker.on_foreground_app_changed(Some(r"C:\Windows\Notepad.exe")).unwrap();
        assert_eq!(gen1, 1);
        assert_eq!(mode1, RuleMode::English);

        // 在 Visit 1 進行中修改設定：停用規則並改為空規則
        tracker.update_desired_config(false, &[]);

        // Visit 1 內部依然處於啟用狀態，poll_pending_apply 正常運作！
        assert!(tracker.is_active_visit_enabled());
        let applied = tracker.poll_pending_apply(gen1, Some(r"C:\Windows\Notepad.exe"), false);
        assert_eq!(applied, Some(RuleMode::English));

        // 離開並進入 Explorer，再回到 Notepad (Visit 2)
        tracker.on_foreground_app_changed(Some(r"C:\Windows\explorer.exe"));
        let visit2 = tracker.on_foreground_app_changed(Some(r"C:\Windows\Notepad.exe"));

        // Visit 2 現在才套用修改後的設定（已停用）
        assert!(!tracker.is_active_visit_enabled());
        assert_eq!(visit2, None);
    }

    #[test]
    fn test_manual_persistence_across_tsf_instances_in_same_visit() {
        let rule = AppInputRule {
            app_id: r"c:\windows\notepad.exe".to_string(),
            display_name: "Notepad".to_string(),
            mode: RuleMode::English,
        };
        let mut tracker = AppInputRulesTracker::new(true, &[rule]);

        // 首次進入 Notepad (Visit 1)
        let (gen1, mode1) = tracker.on_foreground_app_changed(Some(r"C:\Windows\Notepad.exe")).unwrap();
        assert_eq!(mode1, RuleMode::English);

        // 使用者手動切換為 Bopomofo
        assert!(tracker.on_user_manual_toggle(gen1, Some(r"C:\Windows\Notepad.exe"), RuleMode::Bopomofo));

        // 另一個控制項/分頁/新 TSF 實例在同一 visit 啟用，查詢 current_visit_target 依然是手動的 Bopomofo！
        assert_eq!(tracker.current_visit_target(), Some(RuleMode::Bopomofo));

        // 重複觸發 on_foreground_app_changed（同 App）也是回傳手動的 Bopomofo
        let same = tracker.on_foreground_app_changed(Some(r"C:\Windows\Notepad.exe")).unwrap();
        assert_eq!(same, (gen1, RuleMode::Bopomofo));

        // 舊 generation 的 manual toggle 被拒絕
        assert!(!tracker.on_user_manual_toggle(gen1 - 1, Some(r"C:\Windows\Notepad.exe"), RuleMode::English));
    }
}

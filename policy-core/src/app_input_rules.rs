// SPDX-License-Identifier: MIT
//! Windows 應用程式模式規則 (App Input Rules)
//!
//! 規範與審查精簡架構：
//! 1. `normalize_canonical_app_id`：
//!    - 跨平台可編譯且行為一致（不依賴平臺特定的 std::path::Prefix）。
//!    - 僅支援本機磁碟路徑（`[A-Za-z]:\` 開頭），拒絕 UNC（`\\server\share\...`）與相對路徑。
//!    - 限制路徑長度（<= 1024 字元）與 display_name 長度（<= 128 字元）。
//!    - 拒絕控制字元。
//!    - 使用 Unicode lowercase (`to_lowercase()`)。
//!    - 嚴格反斜線格式化，去除相對路徑元素（`.`, `..`）。
//!    - 若文件內任何一條規則無效或重複，整份文件 fail-closed 回退至空規則。
//! 2. 狀態機精簡架構（Desired Config + Current Visit）：
//!    - `desired_config`: 由 `update_config` 更新，只影響未來的 visit（Next-visit-only）。
//!    - `current_visit`: Option<CurrentVisit { app_id: String, generation: u64, target_mode: Option<RuleMode> }>。
//!      不維護過去所有 App 的龐大 Hash 表。
//!    - `on_foreground_app_changed`:
//!      - 若 App 未改變（同 app_id），保持 visit 與 target_mode 不變。
//!      - 若 App 改變（或離開返回），generation 遞增，並從 desired_config 拍下該 App 的初始 target_mode。
//!    - `target(expected_generation, client_app_id, safe, active, secure)`:
//!      - 校驗 expected_generation、client_app_id、安全條件（!secure, active, safe）。
//!      - 回傳 current visit target_mode 而不消耗它，讓同 visit 內的新/多個 TSF 實例也能採用。
//!    - `manual(expected_generation, client_app_id, mode)`:
//!      - 僅修改匹配的 current_visit 的 target_mode，過期或不符的呼叫直接忽略。

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
    let trimmed = raw_path.strip_prefix(r"\\?\").unwrap_or(raw_path);
    if trimmed.is_empty()
        || trimmed.encode_utf16().count() > MAX_PATH_CHARS
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
        let seg_trim = seg;
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
/// invalid entries fail whole document（若有任何無效條目或重複，整份文件回退為空規則）
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
            || name_trimmed.chars().count() > MAX_DISPLAY_NAME_CHARS
            || name_trimmed.chars().any(char::is_control)
        {
            return Vec::new();
        }

        let norm_id = match normalize_canonical_app_id(&rule.app_id) {
            Some(id) => id,
            None => return Vec::new(), // invalid entry fails whole document
        };

        if !seen_ids.insert(norm_id.clone()) {
            // Preserve the first rule when an import repeats an identity.
            continue;
        }

        valid_rules.push(AppInputRule {
            app_id: norm_id,
            display_name: name_trimmed.to_string(),
            mode: rule.mode,
        });
    }

    valid_rules
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// 當前應用程式存取階段 (Current Visit)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentVisit {
    pub app_id: String,
    pub generation: u64,
    pub target_mode: Option<RuleMode>,
}

/// 應用程式存取追蹤器 (Desired Config + Current Visit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInputRulesTracker {
    /// 下一次 visit 時採用的期望設定（update_config 僅修改此項）
    desired_config: AppInputRulesConfigSnapshot,

    /// 當前進行中的 visit（包含 app_id、generation、target_mode）
    current_visit: Option<CurrentVisit>,

    /// 全域單調遞增 generation 計數器
    next_generation: u64,
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
        Self {
            desired_config: AppInputRulesConfigSnapshot {
                enabled,
                rules: map,
            },
            current_visit: None,
            next_generation: 1,
        }
    }

    /// 更新期望設定（Next-visit-only：不影響目前正在進行中的 current_visit）
    pub fn update_config(&mut self, enabled: bool, rules: &[AppInputRule]) {
        let mut map = HashMap::new();
        for r in rules {
            map.insert(r.app_id.clone(), r.mode);
        }
        self.desired_config = AppInputRulesConfigSnapshot {
            enabled,
            rules: map,
        };
    }

    /// 當前景應用程式變更時觸發
    ///
    /// - 若新 App ID 與目前 visit 的 app_id 相同：保持 visit 與 target_mode 不變。
    /// - 若新 App ID 改變（或離開後返回）：遞增 generation，並從 desired_config 拍下該 App 的初始 target_mode。
    pub fn on_foreground_app_changed(&mut self, new_app_id: Option<&str>) -> Option<(u64, Option<RuleMode>)> {
        let norm_new = new_app_id.and_then(normalize_canonical_app_id);

        if let (Some(cur), Some(new)) = (&self.current_visit, &norm_new) {
            if &cur.app_id == new {
                // 未改變 App，保持現有 visit
                return Some((cur.generation, cur.target_mode));
            }
        }

        // App 改變或進入無效 App：
        let app_id = match norm_new {
            Some(id) => id,
            None => {
                self.current_visit = None;
                return None;
            }
        };

        let gen = self.next_generation;
        self.next_generation = self.next_generation.wrapping_add(1);

        let initial_target = if self.desired_config.enabled {
            self.desired_config.rules.get(&app_id).copied()
        } else {
            None
        };

        let visit = CurrentVisit {
            app_id,
            generation: gen,
            target_mode: initial_target,
        };
        self.current_visit = Some(visit);

        Some((gen, initial_target))
    }

    /// 取得目前 visit 的目標模式（供安全邊界檢查或新 TSF 實例獲取目標）
    ///
    /// 審查合約：`target(expected_generation, client_app_id, safe, active, secure)`
    /// - expected_generation 必須符合目前的 visit.generation。
    /// - client_app_id 必須符合目前的 visit.app_id。
    /// - safe: 必須為 true（即沒有 active composition、syllables、candidates、pending edits、gestures）。
    /// - active: 必須為 true（TSF context 處於啟用與 focus 狀態）。
    /// - secure: 必須為 false（安全桌面/密碼欄不套用規則）。
    /// - 回傳 target_mode 而不消耗它，讓同 visit 內的新/多個 TSF 實例都能採用。
    pub fn target(
        &self,
        expected_generation: u64,
        client_app_id: Option<&str>,
        safe: bool,
        active: bool,
        secure: bool,
    ) -> Option<RuleMode> {
        if secure || !active || !safe {
            return None;
        }

        let visit = self.current_visit.as_ref()?;
        if visit.generation != expected_generation {
            return None;
        }

        let norm_client = client_app_id.and_then(normalize_canonical_app_id)?;
        if visit.app_id != norm_client {
            return None;
        }

        visit.target_mode
    }

    /// 使用者手動切換中英文模式時呼叫（例如按 Shift 切換）
    ///
    /// 審查合約：`manual(expected_generation, client_app_id, mode)`
    /// - 僅修改匹配的 current_visit 的 target_mode，過期或不符的呼叫直接忽略。
    pub fn manual(
        &mut self,
        expected_generation: u64,
        client_app_id: Option<&str>,
        mode: RuleMode,
    ) -> bool {
        let norm_client = client_app_id.and_then(normalize_canonical_app_id);
        if let Some(visit) = &mut self.current_visit {
            if visit.target_mode.is_some()
                && visit.generation == expected_generation
                && norm_client.as_deref() == Some(&visit.app_id) {
                visit.target_mode = Some(mode);
                return true;
            }
        }
        false
    }

    /// 取得當前進行中的 visit
    pub fn current_visit(&self) -> Option<&CurrentVisit> {
        self.current_visit.as_ref()
    }

    /// 檢查 desired_config 是否啟用
    pub fn is_desired_enabled(&self) -> bool {
        self.desired_config.enabled
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

        // Duplicate identity retains the first configured mode.
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
        assert_eq!(parse_and_validate_rules(dup_json).len(), 1);
        assert_eq!(parse_and_validate_rules(dup_json)[0].mode, RuleMode::English);

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
    fn test_desired_config_next_visit_only() {
        let rule1 = AppInputRule {
            app_id: r"c:\windows\notepad.exe".to_string(),
            display_name: "Notepad".to_string(),
            mode: RuleMode::English,
        };
        let mut tracker = AppInputRulesTracker::new(true, &[rule1]);

        // Visit 1: 進入 Notepad
        let (gen1, target1) = tracker.on_foreground_app_changed(Some(r"C:\Windows\Notepad.exe")).unwrap();
        assert_eq!(gen1, 1);
        assert_eq!(target1, Some(RuleMode::English));

        // 進行中修改設定：停用規則
        tracker.update_config(false, &[]);

        // Visit 1 的 target 依然存在！
        assert_eq!(
            tracker.target(gen1, Some(r"C:\Windows\Notepad.exe"), true, true, false),
            Some(RuleMode::English)
        );

        // 離開並進入 Explorer，再回到 Notepad (Visit 2)
        tracker.on_foreground_app_changed(Some(r"C:\Windows\explorer.exe"));
        let (gen2, target2) = tracker.on_foreground_app_changed(Some(r"C:\Windows\Notepad.exe")).unwrap();
        assert_eq!(gen2, 3);
        // Visit 2 現在才套用修改後的設定（已停用，無 target_mode）
        assert_eq!(target2, None);
    }

    #[test]
    fn test_manual_persistence_across_multiple_tsf_instances() {
        let rule = AppInputRule {
            app_id: r"c:\windows\notepad.exe".to_string(),
            display_name: "Notepad".to_string(),
            mode: RuleMode::English,
        };
        let mut tracker = AppInputRulesTracker::new(true, &[rule]);

        // Visit 1: 進入 Notepad
        let (gen1, target1) = tracker.on_foreground_app_changed(Some(r"C:\Windows\Notepad.exe")).unwrap();
        assert_eq!(target1, Some(RuleMode::English));

        // TSF Instance 1 取得 target
        assert_eq!(
            tracker.target(gen1, Some(r"C:\Windows\Notepad.exe"), true, true, false),
            Some(RuleMode::English)
        );

        // 使用者手動切換為 Bopomofo
        assert!(tracker.manual(gen1, Some(r"C:\Windows\Notepad.exe"), RuleMode::Bopomofo));

        // TSF Instance 2（或新分頁/控制項）在同一 visit 查詢 target，獲得手動覆蓋值 Bopomofo，且不被消耗！
        assert_eq!(
            tracker.target(gen1, Some(r"C:\Windows\Notepad.exe"), true, true, false),
            Some(RuleMode::Bopomofo)
        );
        assert_eq!(
            tracker.target(gen1, Some(r"C:\Windows\Notepad.exe"), true, true, false),
            Some(RuleMode::Bopomofo)
        );

        // 若 safe 為 false（例如組字中），target 拒絕回傳
        assert_eq!(
            tracker.target(gen1, Some(r"C:\Windows\Notepad.exe"), false, true, false),
            None
        );

        // 若 secure 為 true（密碼欄），target 拒絕回傳
        assert_eq!(
            tracker.target(gen1, Some(r"C:\Windows\Notepad.exe"), true, true, true),
            None
        );

        // 舊 generation 的 manual 被拒絕
        assert!(!tracker.manual(gen1 - 1, Some(r"C:\Windows\Notepad.exe"), RuleMode::English));
    }
}

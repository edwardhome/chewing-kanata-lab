// SPDX-License-Identifier: GPL-3.0-or-later
//! 確定性高頻語法與同音字校正引擎 (Grammar Corrector)
//! 
//! 提供 0.1ms 內無時延之上下文搭配詞規則匹配，
//! 針對繁體中文高頻誤用字「在/再」、「的/得/地」、「做/作」進行確定性重新排序。

/// 根據前文字符串 (上下文)，調整候選字列表中同音字的優先順序
pub fn disambiguate_homophones(context_before: &str, candidates: &mut [String]) {
    if candidates.is_empty() || context_before.is_empty() {
        return;
    }

    // 1. 「在 / 再」歧義消除
    if has_pair(candidates, "在", "再") {
        if should_prefer_zai_again(context_before) {
            promote_to_front(candidates, "再");
        } else if should_prefer_zai_location(context_before) {
            promote_to_front(candidates, "在");
        }
    }

    // 2. 「的 / 得 / 地」歧義消除
    if has_pair(candidates, "的", "得") || has_pair(candidates, "的", "地") {
        if should_prefer_de_manner(context_before) {
            promote_to_front(candidates, "得");
        } else if should_prefer_de_adverbial(context_before) {
            promote_to_front(candidates, "地");
        } else if should_prefer_de_genitive(context_before) {
            promote_to_front(candidates, "的");
        }
    }

    // 3. 「做 / 作」歧義消除
    if has_pair(candidates, "做", "作") {
        if should_prefer_zuo_action(context_before) {
            promote_to_front(candidates, "做");
        } else if should_prefer_zuo_creation(context_before) {
            promote_to_front(candidates, "作");
        }
    }
}

/// 判斷前文是否強烈指向「再」(表示時間延續、再次、重複、條件接續)
fn should_prefer_zai_again(ctx: &str) -> bool {
    let ctx = ctx.trim_end();
    if ctx.is_empty() {
        return false;
    }

    // 強烈指標後綴：
    // 「想好」、「看完」、「查過」、「等一下」等完成式/條件副詞
    const AGAIN_SUFFIXES: &[&str] = &[
        "想好", "做好", "看完", "看過", "吃完", "查過", "搞定",
        "先", "別", "不再", "不要", "請勿", "莫", "切莫",
        "請", "請你", "請您", "下回", "下次", "明天", "改天", "回頭",
        "想", "要", "能", "敢", "會", "可", "得", "願意",
        "一", "二", "三", "幾", "再",
    ];

    for &suffix in AGAIN_SUFFIXES {
        if ctx.ends_with(suffix) {
            return true;
        }
    }

    // 句型包含「先」且離結尾近 (先...再...)
    if let Some(pos) = ctx.rfind('先') {
        let after_xian = &ctx[pos + '先'.len_utf8()..];
        // 「先」在最近 8 個字元內，且未被標點切斷
        if after_xian.chars().count() <= 8 && !after_xian.contains(&['，', '。', '；', ',', '.', ';'][..]) {
            return true;
        }
    }

    false
}

/// 判斷前文是否強烈指向「在」(表示方位、處所、進行式、存在)
fn should_prefer_zai_location(ctx: &str) -> bool {
    let ctx = ctx.trim_end();
    if ctx.is_empty() {
        return false;
    }

    const LOCATION_SUFFIXES: &[&str] = &[
        "正", "正在", "現", "現在", "實", "實在",
        "住", "留", "停", "落", "站", "坐", "躺", "擺", "放", "存", "設", "記",
        "好", "好在", "志", "志在",
    ];

    for &suffix in LOCATION_SUFFIXES {
        if ctx.ends_with(suffix) {
            return true;
        }
    }

    // 代詞 + 在 (例如 我在、你在、他在、誰在、大家在)
    const PRONOUN_SUFFIXES: &[&str] = &[
        "我", "你", "他", "她", "它", "我們", "你們", "他們", "她們", "誰", "大家", "人"
    ];

    for &suffix in PRONOUN_SUFFIXES {
        if ctx.ends_with(suffix) {
            return true;
        }
    }

    false
}

/// 判斷前文是否強烈指向「得」(形容動作結果或程度：跑得快、做得好、氣得)
fn should_prefer_de_manner(ctx: &str) -> bool {
    let ctx = ctx.trim_end();
    if ctx.is_empty() {
        return false;
    }

    const DE_MANNER_VERBS: &[&str] = &[
        "跑", "做", "寫", "看", "聽", "說", "講", "吃", "飛", "過",
        "氣", "笑", "哭", "嚇", "痛", "高興", "變", "長", "走", "跳",
        "覺", "記", "捨", "顯", "免", "贏", "獲", "懶",
    ];

    for &suffix in DE_MANNER_VERBS {
        if ctx.ends_with(suffix) {
            return true;
        }
    }

    false
}

/// 判斷前文是否強烈指向「地」(副詞修飾動詞：慢慢地、認真地、輕輕地)
fn should_prefer_de_adverbial(ctx: &str) -> bool {
    let ctx = ctx.trim_end();
    if ctx.is_empty() {
        return false;
    }

    const DE_ADVERB_SUFFIXES: &[&str] = &[
        "慢慢", "悄悄", "認真", "輕輕", "飛快", "默默", "高興", "痛快", "深情", "堅定",
        "劇烈", "逐步", "不斷", "專心", "熱情",
    ];

    for &suffix in DE_ADVERB_SUFFIXES {
        if ctx.ends_with(suffix) {
            return true;
        }
    }

    false
}

/// 判斷前文是否強烈指向「的」(名詞或定語：我的、好的、美麗的)
fn should_prefer_de_genitive(ctx: &str) -> bool {
    let ctx = ctx.trim_end();
    if ctx.is_empty() {
        return false;
    }

    const GENITIVE_SUFFIXES: &[&str] = &[
        "我", "你", "他", "她", "它", "我們", "你們", "他們", "誰",
        "好", "壞", "新", "舊", "大", "小", "真",
    ];

    for &suffix in GENITIVE_SUFFIXES {
        if ctx.ends_with(suffix) {
            return true;
        }
    }

    false
}

/// 判斷前文是否指向「做」(具體動作、製作：做事、做出、做人)
fn should_prefer_zuo_action(ctx: &str) -> bool {
    let ctx = ctx.trim_end();
    if ctx.is_empty() {
        return false;
    }

    const ZUO_ACTION_PREFIXES: &[&str] = &[
        "想", "要", "會", "能", "敢", "去", "來", "在", "再", "打算", "準備",
    ];

    for &suffix in ZUO_ACTION_PREFIXES {
        if ctx.ends_with(suffix) {
            return true;
        }
    }

    false
}

/// 判斷前文是否指向「作」(創作、抽象工作、寫作：作為、作品、作家、作者、作戰)
fn should_prefer_zuo_creation(ctx: &str) -> bool {
    let ctx = ctx.trim_end();
    if ctx.is_empty() {
        return false;
    }

    const ZUO_CREATION_PREFIXES: &[&str] = &[
        "寫", "創", "合", "著", "工", "創",
    ];

    for &suffix in ZUO_CREATION_PREFIXES {
        if ctx.ends_with(suffix) {
            return true;
        }
    }

    false
}

/// 輔助函式：檢查列表中是否同時包含兩個特定字元
fn has_pair(candidates: &[String], a: &str, b: &str) -> bool {
    let mut has_a = false;
    let mut has_b = false;
    for c in candidates {
        if c == a {
            has_a = true;
        } else if c == b {
            has_b = true;
        }
        if has_a && has_b {
            return true;
        }
    }
    false
}

/// 輔助函式：將指定項目移至列表第 1 位 (Index 0)
fn promote_to_front(candidates: &mut [String], target: &str) {
    if let Some(pos) = candidates.iter().position(|c| c == target) {
        if pos > 0 {
            candidates[..=pos].rotate_right(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zai_again_cases() {
        let mut cand = vec!["在".to_string(), "再".to_string(), "載".to_string()];
        disambiguate_homophones("我想好", &mut cand);
        assert_eq!(cand[0], "再");

        let mut cand = vec!["在".to_string(), "再".to_string(), "載".to_string()];
        disambiguate_homophones("請你", &mut cand);
        assert_eq!(cand[0], "再");

        let mut cand = vec!["在".to_string(), "再".to_string(), "載".to_string()];
        disambiguate_homophones("先洗手", &mut cand);
        assert_eq!(cand[0], "再");

        let mut cand = vec!["在".to_string(), "再".to_string(), "載".to_string()];
        disambiguate_homophones("不再", &mut cand);
        assert_eq!(cand[0], "再");
    }

    #[test]
    fn test_zai_location_cases() {
        let mut cand = vec!["再".to_string(), "在".to_string(), "載".to_string()];
        disambiguate_homophones("他正", &mut cand);
        assert_eq!(cand[0], "在");

        let mut cand = vec!["再".to_string(), "在".to_string(), "載".to_string()];
        disambiguate_homophones("我現", &mut cand);
        assert_eq!(cand[0], "在");

        let mut cand = vec!["再".to_string(), "在".to_string(), "載".to_string()];
        disambiguate_homophones("我", &mut cand);
        assert_eq!(cand[0], "在");
    }

    #[test]
    fn test_de_manner_and_genitive() {
        let mut cand = vec!["的".to_string(), "得".to_string(), "地".to_string()];
        disambiguate_homophones("跑", &mut cand);
        assert_eq!(cand[0], "得");

        let mut cand = vec!["得".to_string(), "的".to_string(), "地".to_string()];
        disambiguate_homophones("我", &mut cand);
        assert_eq!(cand[0], "的");

        let mut cand = vec!["的".to_string(), "得".to_string(), "地".to_string()];
        disambiguate_homophones("認真", &mut cand);
        assert_eq!(cand[0], "地");
    }
}

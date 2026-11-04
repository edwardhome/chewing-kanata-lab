// SPDX-License-Identifier: MIT
//! Shift 字母英文單字組字狀態機與 ASCII Composition
//! 移植自 edwardhome/AI-Bopomofo (ShiftLetterSession.swift & ASCIIComposition.swift)

use crate::technical_lexicon::TechnicalLexicon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ASCIIComposition {
    pub raw: String,
    pub cursor: usize,
    pub selected_index: usize,
    pub candidates: Vec<String>,
}

impl Default for ASCIIComposition {
    fn default() -> Self {
        Self::new("", None, 0, &TechnicalLexicon::default())
    }
}

impl ASCIIComposition {
    pub fn new(raw: &str, cursor: Option<usize>, selected_index: usize, lexicon: &TechnicalLexicon) -> Self {
        // 限制 ASCII 32..=126 以及 176 ('°')，長度上限 256
        let text: String = raw
            .chars()
            .filter(|&c| {
                let v = c as u32;
                (32..=126).contains(&v) || v == 176
            })
            .take(256)
            .collect();

        Self::accepted(text, cursor, selected_index, lexicon)
    }

    // Only verified candidate selections and edits of existing composition reach
    // this constructor; arbitrary keyboard input remains ASCII-filtered above.
    fn accepted(text: String, cursor: Option<usize>, selected_index: usize, lexicon: &TechnicalLexicon) -> Self {
        let char_count = text.chars().count();
        let cur = match cursor {
            Some(c) => c.min(char_count),
            None => char_count,
        };

        let mut candidates = Vec::new();
        if !text.is_empty() {
            candidates.push(text.clone());
            if cur == char_count {
                for comp in lexicon.completions(&text) {
                    if !candidates.contains(&comp) {
                        candidates.push(comp);
                    }
                }
            }
        }

        let sel = if !candidates.is_empty() && selected_index < candidates.len() {
            selected_index
        } else {
            0
        };

        Self {
            raw: text,
            cursor: cur,
            selected_index: sel,
            candidates,
        }
    }

    pub fn display(&self) -> &str {
        if self.candidates.is_empty() {
            &self.raw
        } else {
            &self.candidates[self.selected_index]
        }
    }

    pub fn display_cursor(&self) -> usize {
        if self.selected_index == 0 {
            self.cursor
        } else {
            self.display().chars().count()
        }
    }

    pub fn applying(&self, action: CompositionAction, lexicon: &TechnicalLexicon) -> CompositionResult {
        let chars: Vec<char> = self.display().chars().collect();
        let cur = self.display_cursor();

        match action {
            CompositionAction::Escape => {
                if self.selected_index == 0 {
                    CompositionResult::Cancel
                } else {
                    CompositionResult::Updated(Self::accepted(self.raw.clone(), Some(self.cursor), 0, lexicon))
                }
            }
            CompositionAction::Enter => CompositionResult::Commit(self.display().to_string()),
            CompositionAction::Shortcut => CompositionResult::PassThrough(self.display().to_string()),
            CompositionAction::Select(idx) => {
                if idx < self.candidates.len() {
                    CompositionResult::Updated(Self::accepted(self.raw.clone(), Some(self.cursor), idx, lexicon))
                } else {
                    CompositionResult::Rejected
                }
            }
            CompositionAction::Cycle(direction) => {
                if self.candidates.len() > 1 {
                    let count = self.candidates.len();
                    let new_idx = if direction < 0 {
                        (self.selected_index + count - 1) % count
                    } else {
                        (self.selected_index + 1) % count
                    };
                    CompositionResult::Updated(Self::accepted(self.raw.clone(), Some(self.cursor), new_idx, lexicon))
                } else {
                    CompositionResult::Updated(self.clone())
                }
            }
            CompositionAction::Insert(literal) => {
                if literal.is_empty()
                    || chars.len() + literal.chars().count() > 256
                    || !literal.chars().all(|c| {
                        let v = c as u32;
                        (32..=126).contains(&v) || v == 176
                    })
                {
                    return CompositionResult::Rejected;
                }
                let mut new_chars = Vec::with_capacity(chars.len() + literal.chars().count());
                new_chars.extend_from_slice(&chars[..cur]);
                new_chars.extend(literal.chars());
                new_chars.extend_from_slice(&chars[cur..]);
                let new_raw: String = new_chars.into_iter().collect();
                let new_cursor = cur + literal.chars().count();
                CompositionResult::Updated(Self::accepted(new_raw, Some(new_cursor), 0, lexicon))
            }
            CompositionAction::Backspace => {
                if cur > 0 {
                    let mut new_chars = Vec::with_capacity(chars.len());
                    new_chars.extend_from_slice(&chars[..cur - 1]);
                    new_chars.extend_from_slice(&chars[cur..]);
                    let new_raw: String = new_chars.into_iter().collect();
                    CompositionResult::Updated(Self::accepted(new_raw, Some(cur - 1), 0, lexicon))
                } else {
                    CompositionResult::Updated(self.clone())
                }
            }
            CompositionAction::Delete => {
                if cur < chars.len() {
                    let mut new_chars = Vec::with_capacity(chars.len());
                    new_chars.extend_from_slice(&chars[..cur]);
                    new_chars.extend_from_slice(&chars[cur + 1..]);
                    let new_raw: String = new_chars.into_iter().collect();
                    CompositionResult::Updated(Self::accepted(new_raw, Some(cur), 0, lexicon))
                } else {
                    CompositionResult::Updated(self.clone())
                }
            }
            CompositionAction::Left => {
                let new_cur = cur.saturating_sub(1);
                CompositionResult::Updated(Self::accepted(self.display().to_string(), Some(new_cur), 0, lexicon))
            }
            CompositionAction::Right => {
                let new_cur = (cur + 1).min(chars.len());
                CompositionResult::Updated(Self::accepted(self.display().to_string(), Some(new_cur), 0, lexicon))
            }
            CompositionAction::Home => {
                CompositionResult::Updated(Self::accepted(self.display().to_string(), Some(0), 0, lexicon))
            }
            CompositionAction::End => {
                CompositionResult::Updated(Self::accepted(self.display().to_string(), Some(chars.len()), 0, lexicon))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionAction {
    Insert(String),
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Cycle(i32),
    Select(usize),
    Escape,
    Enter,
    Shortcut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionResult {
    Updated(ASCIIComposition),
    Commit(String),
    Cancel,
    PassThrough(String),
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftMode {
    Initial,
    Word,
    Uppercase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShiftLetterSession {
    pub composition: ASCIIComposition,
    pub mode: ShiftMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionResult {
    Updated(ShiftLetterSession),
    Commit(String),
    CommitAndContinue(String, ShiftLetterSession),
    PassThrough(String),
    Forward,
    Cancel,
    Rejected,
}

impl ShiftLetterSession {
    pub fn new(composition: ASCIIComposition, mode: ShiftMode) -> Self {
        Self { composition, mode }
    }

    /// 從首個大寫字母建立 Session (如 Shift+S)
    pub fn start_with_letter(letter: char, lexicon: &TechnicalLexicon) -> Self {
        let comp = ASCIIComposition::new(&letter.to_string(), None, 0, lexicon);
        Self {
            composition: comp,
            mode: ShiftMode::Initial,
        }
    }

    /// 釋放 Shift 鍵時的狀態轉移
    pub fn releasing_shift(&self) -> SessionResult {
        match self.mode {
            ShiftMode::Initial => SessionResult::Updated(Self {
                composition: self.composition.clone(),
                mode: ShiftMode::Word,
            }),
            ShiftMode::Word => SessionResult::Updated(self.clone()),
            ShiftMode::Uppercase => SessionResult::Commit(self.composition.display().to_string()),
        }
    }

    pub fn applying(
        &self,
        action: CompositionAction,
        shift_held: bool,
        repeating: bool,
        lexicon: &TechnicalLexicon,
    ) -> SessionResult {
        if action == CompositionAction::Escape && self.composition.selected_index == 0 {
            return SessionResult::Cancel;
        }
        if action == CompositionAction::Shortcut {
            return SessionResult::PassThrough(self.composition.display().to_string());
        }
        if action == CompositionAction::Enter {
            return SessionResult::Commit(self.composition.display().to_string());
        }

        // 連續大寫只暫存當次要送出的內容；刪除與方向鍵交回原應用程式
        if self.mode == ShiftMode::Uppercase {
            if let CompositionAction::Insert(ref text) = action {
                if !text.is_empty() && text.chars().all(|c| (32..=126).contains(&(c as u32))) {
                    let commit_text = format!("{}{}", self.composition.display(), text);
                    return SessionResult::CommitAndContinue(
                        commit_text,
                        Self {
                            composition: ASCIIComposition::default(),
                            mode: ShiftMode::Uppercase,
                        },
                    );
                }
            }
            return SessionResult::Forward;
        }

        // Mode::Initial 且持續按住 Shift 打第二個大寫字母 -> 轉為連續大寫模式
        if self.mode == ShiftMode::Initial && shift_held && !repeating {
            if let CompositionAction::Insert(ref text) = action {
                if text.len() == 1 {
                    let b = text.as_bytes()[0];
                    if (b'A'..=b'Z').contains(&b) {
                        let commit_text = format!("{}{}", self.composition.display(), text);
                        return SessionResult::CommitAndContinue(
                            commit_text,
                            Self {
                                composition: ASCIIComposition::default(),
                                mode: ShiftMode::Uppercase,
                            },
                        );
                    }
                }
            }
        }

        // 單字模式下，按空白鍵送出單字 + 空格
        if let CompositionAction::Insert(ref s) = action {
            if s == " " {
                return SessionResult::Commit(format!("{} ", self.composition.display()));
            }
        }

        let next_mode = if self.mode == ShiftMode::Initial && shift_held {
            ShiftMode::Initial
        } else {
            ShiftMode::Word
        };

        match self.composition.applying(action, lexicon) {
            CompositionResult::Updated(edited) => SessionResult::Updated(Self {
                composition: edited,
                mode: next_mode,
            }),
            CompositionResult::Commit(text) => SessionResult::Commit(text),
            CompositionResult::PassThrough(text) => SessionResult::PassThrough(text),
            CompositionResult::Cancel => SessionResult::Cancel,
            CompositionResult::Rejected => SessionResult::Rejected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_word_flow() {
        let lexicon = TechnicalLexicon::default();
        // 1. Shift+S 啟動
        let session = ShiftLetterSession::start_with_letter('S', &lexicon);
        assert_eq!(session.mode, ShiftMode::Initial);
        assert_eq!(session.composition.display(), "S");

        // 2. 放開 Shift
        let res = session.releasing_shift();
        let session = match res {
            SessionResult::Updated(s) => s,
            _ => panic!("Expected updated"),
        };
        assert_eq!(session.mode, ShiftMode::Word);

        // 3. 輸入 "w", "i", "f", "t"
        let res = session.applying(CompositionAction::Insert("w".into()), false, false, &lexicon);
        let session = match res {
            SessionResult::Updated(s) => s,
            _ => panic!("Expected updated"),
        };
        let res = session.applying(CompositionAction::Insert("ift".into()), false, false, &lexicon);
        let session = match res {
            SessionResult::Updated(s) => s,
            _ => panic!("Expected updated"),
        };
        assert_eq!(session.composition.display(), "Swift");

        // 4. 按 Space 送出單字加空格
        let res = session.applying(CompositionAction::Insert(" ".into()), false, false, &lexicon);
        assert_eq!(res, SessionResult::Commit("Swift ".into()));
    }

    #[test]
    fn test_continuous_uppercase() {
        let lexicon = TechnicalLexicon::default();
        // 1. Shift+A 啟動
        let session = ShiftLetterSession::start_with_letter('A', &lexicon);
        assert_eq!(session.mode, ShiftMode::Initial);

        // 2. 依然按住 Shift，輸入 'B'
        let res = session.applying(CompositionAction::Insert("B".into()), true, false, &lexicon);
        let next_session = match res {
            SessionResult::CommitAndContinue(committed, s) => {
                assert_eq!(committed, "AB");
                s
            }
            _ => panic!("Expected CommitAndContinue"),
        };
        assert_eq!(next_session.mode, ShiftMode::Uppercase);

        // 3. 繼續輸入 'C'
        let res = next_session.applying(CompositionAction::Insert("C".into()), true, false, &lexicon);
        match res {
            SessionResult::CommitAndContinue(committed, _) => {
                assert_eq!(committed, "C");
            }
            _ => panic!("Expected CommitAndContinue"),
        }
    }

    #[test]
    fn test_prefix_completion_cycle() {
        let lexicon = TechnicalLexicon::default();
        // 輸入 "op"
        let comp = ASCIIComposition::new("op", None, 0, &lexicon);
        // candidates[0] = "op", candidates[1] = "OpenOCD"
        assert_eq!(comp.candidates[0], "op");
        assert!(comp.candidates.contains(&"OpenOCD".to_string()));

        // Cycle 往後一個候選
        let res = comp.applying(CompositionAction::Cycle(1), &lexicon);
        let comp2 = match res {
            CompositionResult::Updated(c) => c,
            _ => panic!("Expected updated"),
        };
        assert!(comp2.candidates.contains(&"OpenOCD".to_string()));

        // 按 Enter 送出選取之詞
        let selected_term = comp2.display().to_string();
        let res = comp2.applying(CompositionAction::Enter, &lexicon);
        assert_eq!(res, CompositionResult::Commit(selected_term));
    }
    #[test]
    fn selected_engineering_unicode_survives_editing() {
        let lexicon = TechnicalLexicon::default();
        let comp = ASCIIComposition::new("sio2", None, 1, &lexicon);
        assert_eq!(comp.display(), "SiO₂");
        let CompositionResult::Updated(edited) = comp.applying(CompositionAction::Insert("X".into()), &lexicon) else { panic!("edit"); };
        assert_eq!(edited.raw, "SiO₂X");
        let CompositionResult::Updated(edited) = edited.applying(CompositionAction::Backspace, &lexicon) else { panic!("delete"); };
        assert_eq!(edited.raw, "SiO₂");
    }

    #[test]
    fn escape_restores_preview_before_canceling_session() {
        let lexicon = TechnicalLexicon::default();
        let session = ShiftLetterSession::new(ASCIIComposition::new("i2c", None, 1, &lexicon), ShiftMode::Word);
        let SessionResult::Updated(next) = session.applying(CompositionAction::Escape, false, false, &lexicon) else { panic!("restore"); };
        assert_eq!(next.composition.display(), "i2c");
        assert_eq!(next.applying(CompositionAction::Escape, false, false, &lexicon), SessionResult::Cancel);
    }

    #[test]
    fn deleting_initial_letter_retains_explicit_english() {
        let lexicon = TechnicalLexicon::default();
        let session = ShiftLetterSession::start_with_letter('M', &lexicon);
        let SessionResult::Updated(next) = session.applying(CompositionAction::Backspace, false, false, &lexicon) else { panic!("edit"); };
        assert_eq!(next.mode, ShiftMode::Word);
        assert_eq!(next.composition.raw, "");
        let SessionResult::Updated(next) = next.applying(CompositionAction::Insert("mm".into()), false, false, &lexicon) else { panic!("edit"); };
        assert_eq!(next.composition.display(), "mm");
    }

}

/// An F gesture owns only its newly entered symbols. Backspace must never delete
/// the application text or the English composition that preceded the gesture.
#[derive(Debug, Clone)]
pub struct FCompositionBuffer {
    pub base: Option<ShiftLetterSession>,
    symbols: String,
}

impl FCompositionBuffer {
    pub fn new(mut base: Option<ShiftLetterSession>, lexicon: &TechnicalLexicon) -> Self {
        if let Some(session) = &mut base {
            session.composition = ASCIIComposition::accepted(session.composition.raw.clone(),
                Some(session.composition.cursor), 0, lexicon);
        }
        Self { base, symbols: String::new() }
    }
    pub fn append(&mut self, symbol: &str) {
        let base_len = self.base.as_ref().map_or(0, |s| s.composition.raw.chars().count());
        if base_len + self.symbols.chars().count() + symbol.chars().count() <= 256
            && symbol.chars().all(|c| c.is_ascii_graphic() || c == '°') {
            self.symbols.push_str(symbol);
        }
    }
    pub fn backspace(&mut self) { self.symbols.pop(); }
    pub fn symbols(&self) -> &str { &self.symbols }
    pub fn preview(&self, lexicon: &TechnicalLexicon) -> ASCIIComposition {
        let base = self.base.as_ref().map(|s| s.composition.clone())
            .unwrap_or_else(|| ASCIIComposition::new("", None, 0, lexicon));
        if self.symbols.is_empty() { return base; }
        match base.applying(CompositionAction::Insert(self.symbols.clone()), lexicon) {
            CompositionResult::Updated(next) => next,
            _ => base,
        }
    }
}

#[cfg(test)]
mod f_buffer_tests {
    use super::*;
    #[test]
    fn held_f_backspace_never_deletes_existing_word() {
        let lexicon = TechnicalLexicon::default();
        let base = ShiftLetterSession::new(ASCIIComposition::new("abc", Some(1), 0, &lexicon), ShiftMode::Word);
        let mut buffer = FCompositionBuffer::new(Some(base), &lexicon);
        buffer.append("+"); buffer.append("8");
        assert_eq!(buffer.preview(&lexicon).raw, "a+8bc");
        buffer.backspace(); assert_eq!(buffer.preview(&lexicon).raw, "a+bc");
        buffer.backspace(); buffer.backspace();
        assert_eq!(buffer.preview(&lexicon).raw, "abc");
        assert_eq!(buffer.preview(&lexicon).cursor, 1);
    }
    #[test]
    fn f_gesture_restores_raw_instead_of_accepting_a_preview() {
        let lexicon = TechnicalLexicon::default();
        let base = ShiftLetterSession::new(ASCIIComposition::new("i2c", None, 1, &lexicon), ShiftMode::Word);
        let mut buffer = FCompositionBuffer::new(Some(base), &lexicon);
        buffer.append("+");
        assert_eq!(buffer.preview(&lexicon).raw, "i2c+");
    }
    #[test]
    fn direct_english_symbols_are_buffered_until_release() {
        let lexicon = TechnicalLexicon::default();
        let mut buffer = FCompositionBuffer::new(None, &lexicon);
        buffer.append("°"); buffer.append("+"); buffer.backspace();
        assert_eq!(buffer.symbols(), "°");
        assert_eq!(buffer.preview(&lexicon).display(), "°");
    }
}

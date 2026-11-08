// SPDX-License-Identifier: MIT
//! Immutable candidate actions for one exact uncommitted composition and epoch.
//! No document access, selection side effects, or text logging occurs here.
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use crate::engineering_lexicon::TechnicalCandidate;
use crate::shared_lexicon::SharedCandidate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateSource { Native, Engineering, Shared }

/// Structural IDs avoid delimiter ambiguity and do not depend on displayed row numbers.
/// Native occurrence distinguishes identical native strings without dropping engine choices.
#[derive(Clone, PartialEq, Eq)]
pub enum CandidateId {
    Native { text: String, occurrence: usize },
    Engineering { alias: String, output: String },
    Shared { entry_id: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplacementScope { WholeComposition { utf16_len: usize } }

#[derive(Clone, PartialEq, Eq)]
pub enum CandidateAction {
    Native { original_index: usize },
    Engineering { entry_id: CandidateId, scope: ReplacementScope },
    Shared { entry_id: CandidateId, scope: ReplacementScope },
}

#[derive(Clone, PartialEq, Eq)]
pub struct Candidate {
    id: CandidateId,
    commit_text: String,
    annotation: Option<String>,
    action: CandidateAction,
    shared_entry_ids: Vec<String>,
}
impl Candidate {
    pub fn id(&self) -> &CandidateId { &self.id }
    pub fn commit_text(&self) -> &str { &self.commit_text }
    pub fn annotation(&self) -> Option<&str> { self.annotation.as_deref() }
    pub fn action(&self) -> &CandidateAction { &self.action }
    pub fn shared_entry_ids(&self) -> &[String] { &self.shared_entry_ids }
    pub fn source(&self) -> CandidateSource {
        match self.action { CandidateAction::Native { .. } => CandidateSource::Native,
            CandidateAction::Engineering { .. } => CandidateSource::Engineering,
            CandidateAction::Shared { .. } => CandidateSource::Shared }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeSelection {
    pub original_index: usize,
    pub page: usize,
    pub slot: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotError { EmptyPageSize }

#[derive(Clone, PartialEq, Eq)]
pub struct CandidateSnapshot {
    generation: u64,
    composition: String,
    page_size: usize,
    items: Vec<Candidate>,
}
impl CandidateSnapshot {
    /// `native` must come from Editor::all_candidates(), never its current-page slice.
    /// Caller advances generation on edit, selector/cursor/focus/mode changes, or cancellation.
    /// Engineering candidates must already be exact alias matches for `composition`.
    pub fn new(generation: u64, composition: &str, native: &[String],
        engineering: &[TechnicalCandidate], page_size: usize) -> Result<Self, SnapshotError> {
        if page_size == 0 { return Err(SnapshotError::EmptyPageSize); }
        let mut items = Vec::new();
        let mut seen = BTreeSet::from([composition.to_string()]);
        let mut occurrences = BTreeMap::<String, usize>::new();
        for (original_index, text) in native.iter().enumerate() {
            let occurrence = occurrences.entry(text.clone()).or_default();
            let id = CandidateId::Native { text: text.clone(), occurrence: *occurrence };
            *occurrence += 1;
            seen.insert(text.clone());
            items.push(Candidate { id, commit_text: text.clone(), annotation: None,
                action: CandidateAction::Native { original_index }, shared_entry_ids: Vec::new() });
        }
        if !composition.is_empty() {
            for entry in engineering {
                if entry.text.is_empty() || !seen.insert(entry.text.clone()) { continue; }
                let id = CandidateId::Engineering { alias: composition.to_string(), output: entry.text.clone() };
                items.push(Candidate { id: id.clone(), commit_text: entry.text.clone(),
                    annotation: entry.annotation.clone(), shared_entry_ids: Vec::new(), action: CandidateAction::Engineering {
                        entry_id: id, scope: ReplacementScope::WholeComposition {
                            utf16_len: composition.encode_utf16().count(),
                        },
                    },
                });
            }
        }
        Ok(Self { generation, composition: composition.to_string(), page_size, items })
    }
    /// Shared entries must already match the full engine reading. Preserve native
    /// and engineering order, with source IDs retained even when output is deduplicated.
    pub fn new_with_shared(generation: u64, composition: &str, native: &[String],
        engineering: &[TechnicalCandidate], shared_entries: &[SharedCandidate<'_>],
        page_size: usize) -> Result<Self, SnapshotError> {
        let mut snapshot = Self::new(generation, composition, native, engineering, page_size)?;
        if composition.is_empty() { return Ok(snapshot); }
        let mut outputs: BTreeMap<String, usize> = snapshot.items.iter().enumerate()
            .map(|(i, item)| (item.commit_text.clone(), i)).collect();
        for entry in shared_entries {
            if entry.text == composition { continue; }
            if let Some(&i) = outputs.get(&entry.text) {
                if !snapshot.items[i].shared_entry_ids.contains(&entry.entry_id) {
                    snapshot.items[i].shared_entry_ids.push(entry.entry_id.clone());
                }
                continue;
            }
            let id = CandidateId::Shared { entry_id: entry.entry_id.clone() };
            outputs.insert(entry.text.clone(), snapshot.items.len());
            snapshot.items.push(Candidate { id: id.clone(), commit_text: entry.text.clone(),
                annotation: entry.annotation.clone(), shared_entry_ids: vec![entry.entry_id.clone()],
                action: CandidateAction::Shared { entry_id: id, scope: ReplacementScope::WholeComposition {
                    utf16_len: composition.encode_utf16().count(),
                } },
            });
        }
        Ok(snapshot)
    }
    pub fn generation(&self) -> u64 { self.generation }
    pub fn items(&self) -> &[Candidate] { &self.items }
    pub fn page_size(&self) -> usize { self.page_size }
    pub fn page_count(&self) -> usize {
        self.items.len() / self.page_size + usize::from(self.items.len() % self.page_size != 0)
    }
    pub fn page_range(&self, page: usize) -> Option<Range<usize>> {
        let start = page.checked_mul(self.page_size)?;
        if start >= self.items.len() { return None; }
        Some(start..start.saturating_add(self.page_size).min(self.items.len()))
    }
    pub fn page(&self, page: usize) -> Option<&[Candidate]> {
        self.page_range(page).map(|range| &self.items[range])
    }
    pub fn global_index(&self, page: usize, slot: usize) -> Option<usize> {
        if slot >= self.page_size { return None; }
        let range = self.page_range(page)?;
        let index = range.start.checked_add(slot)?;
        (index < range.end).then_some(index)
    }
    pub fn is_current(&self, generation: u64, composition: &str) -> bool {
        self.generation == generation && self.composition == composition
    }
    /// Use this guarded resolver for every Enter, selection-key, and accessibility action.
    pub fn resolve(&self, page: usize, slot: usize, generation: u64, composition: &str) -> Option<&Candidate> {
        if !self.is_current(generation, composition) { return None; }
        self.items.get(self.global_index(page, slot)?)
    }
    /// The engine's select(n) is page-local. Synchronize its page with PAGEUP/PAGEDOWN,
    /// never SPACE (which may change the selector's phrase range), before selecting slot.
    pub fn native_selection(&self, page: usize, slot: usize, generation: u64,
        composition: &str) -> Option<NativeSelection> {
        let candidate = self.resolve(page, slot, generation, composition)?;
        match candidate.action {
            CandidateAction::Native { original_index } => Some(NativeSelection {
                original_index, page: original_index / self.page_size, slot: original_index % self.page_size,
            }),
            CandidateAction::Engineering { .. } | CandidateAction::Shared { .. } => None,
        }
    }
    /// Engineering entries cannot become native dictionary deletion requests.
    pub fn native_unlearn_index(&self, page: usize, slot: usize, generation: u64,
        composition: &str) -> Option<usize> {
        self.native_selection(page, slot, generation, composition).map(|selection| selection.original_index)
    }
}

macro_rules! private_debug {
    ($($kind:ty),+) => { $(impl std::fmt::Debug for $kind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(concat!(stringify!($kind), "(<private>)"))
        }
    })+ };
}
private_debug!(CandidateId, CandidateAction, Candidate, CandidateSnapshot);

#[cfg(test)]
mod tests {
    use super::*;
    fn engineering(text: &str) -> TechnicalCandidate {
        TechnicalCandidate { text: text.into(), annotation: Some("只供顯示".into()) }
    }
    #[test]
    fn native_order_and_original_later_page_mapping_survive_engineering_dedup() {
        let native = ["甲", "乙", "丙", "丁", "戊"].map(String::from);
        let extras = [engineering("甲"), engineering("原文"), engineering("CH₄"), engineering("CH₄")];
        let snapshot = CandidateSnapshot::new(9, "原文", &native, &extras, 2).unwrap();
        assert_eq!(snapshot.items().iter().map(Candidate::commit_text).collect::<Vec<_>>(),
            ["甲", "乙", "丙", "丁", "戊", "CH₄"]);
        assert_eq!(snapshot.page_count(), 3);
        assert_eq!(snapshot.native_selection(2, 0, 9, "原文"),
            Some(NativeSelection { original_index: 4, page: 2, slot: 0 }));
        assert_eq!(snapshot.native_unlearn_index(2, 0, 9, "原文"), Some(4));
        assert_eq!(snapshot.native_unlearn_index(2, 1, 9, "原文"), None);
        let chosen = snapshot.resolve(2, 1, 9, "原文").unwrap();
        assert_eq!(chosen.commit_text(), "CH₄");
        assert_eq!(chosen.annotation(), Some("只供顯示"));
        assert_eq!(chosen.source(), CandidateSource::Engineering);
    }
    #[test]
    fn exact_identity_and_epoch_block_stale_actions() {
        let snapshot = CandidateSnapshot::new(5, "甲烷", &["甲烷".into()], &[engineering("CH₄")], 10).unwrap();
        for (generation, composition) in [(6, "甲烷"), (5, "甲烷濃度"), (5, " 甲烷"), (4, "甲烷")] {
            assert!(snapshot.resolve(0, 1, generation, composition).is_none());
            assert!(snapshot.native_unlearn_index(0, 0, generation, composition).is_none());
        }
    }
    #[test]
    fn empty_partial_pages_and_extreme_indices_are_safe() {
        assert_eq!(CandidateSnapshot::new(0, "", &[], &[], 0), Err(SnapshotError::EmptyPageSize));
        let empty = CandidateSnapshot::new(0, "", &[], &[engineering("CH₄")], 10).unwrap();
        assert_eq!(empty.page_count(), 0);
        assert!(empty.page(0).is_none());
        let snapshot = CandidateSnapshot::new(0, "甲", &["甲".into(), "乙".into(), "丙".into()], &[], 2).unwrap();
        assert_eq!(snapshot.page_range(1), Some(2..3));
        for (page, slot) in [(1,1), (0,2), (2,0), (usize::MAX,0), (0,usize::MAX)] {
            assert!(snapshot.global_index(page, slot).is_none());
        }
        assert!(snapshot.page_range(usize::MAX).is_none());
    }
    #[test]
    fn unicode_scope_stable_structural_ids_and_private_debug() {
        let snapshot = CandidateSnapshot::new(2, "𠮷甲", &[], &[engineering("X₂")], 2).unwrap();
        let shifted = CandidateSnapshot::new(3, "𠮷甲", &["原生".into()], &[engineering("X₂")], 2).unwrap();
        assert_eq!(snapshot.items()[0].id(), shifted.items()[1].id());
        assert!(matches!(snapshot.items()[0].action(), CandidateAction::Engineering {
            scope: ReplacementScope::WholeComposition { utf16_len: 3 }, .. }));
        for text in [format!("{:?}", snapshot), format!("{:?}", snapshot.items()[0]),
            format!("{:?}", snapshot.items()[0].id()), format!("{:?}", snapshot.items()[0].action())] {
            assert!(!text.contains("𠮷甲") && !text.contains("X₂") && !text.contains("只供顯示"));
        }
        let duplicate = CandidateSnapshot::new(0, "句", &["同".into(), "同".into()], &[], 2).unwrap();
        assert_ne!(duplicate.items()[0].id(), duplicate.items()[1].id());
        assert_eq!(duplicate.native_selection(0,1,0,"句").unwrap().original_index, 1);
    }
}

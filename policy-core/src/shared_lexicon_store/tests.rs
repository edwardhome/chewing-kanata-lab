use super::*;
use crate::shared_lexicon::{entry_id, LexiconEntry, Review, ReviewStatus, TermType, Usage};

fn document(version: &str, annotation: &str) -> LexiconDocument {
    LexiconDocument { format: "ai-bopomofo.lexicon".into(), schema_version: "1.0.0".into(),
        document_type: "review".into(), dictionary_version: version.into(), entries: vec![LexiconEntry {
            entry_id: entry_id(TermType::TechnicalAlias, "sio2", "SiO₂"), term_type: TermType::TechnicalAlias,
            reading: "sio2".into(), text: "SiO₂".into(), source: "synthetic-test".into(),
            usage: Some(Usage { commit_count: 12, manual_selection_count: 3 }),
            review: Review { status: ReviewStatus::Proposed, user_confirmed: false },
            model_suggestion: None, category: Some("化學".into()), annotation: Some(annotation.into()),
            sources: None, policy: Some("candidate_only".into()),
        }] }
}
fn bytes(version: &str, annotation: &str) -> Vec<u8> { shared_lexicon::encode(&document(version, annotation)).unwrap() }
fn store() -> (tempfile::TempDir, SharedStore) {
    let temp = tempfile::tempdir().unwrap();
    let store = SharedStore::new(temp.path().join("SharedLexicon"));
    (temp, store)
}
fn deploy(store: &SharedStore, version: &str) -> Snapshot {
    let revision = store.status().unwrap().revision;
    let preview = store.prepare(&bytes(version, version), revision.as_deref()).unwrap();
    store.approve(preview).unwrap()
}
fn active_bytes(store: &SharedStore) -> Vec<u8> { fs::read(store.root.join("active.json")).unwrap() }

#[test]
fn preview_and_cancel_never_create_store_or_approve_external_claims() {
    let (_temp, store) = store();
    let mut doc = document("v1", "合成詞");
    doc.document_type = "approved".into();
    doc.entries[0].review = Review { status: ReviewStatus::Approved, user_confirmed: true };
    let prepared = store.prepare(&shared_lexicon::encode(&doc).unwrap(), None).unwrap();
    assert_eq!(prepared.document(), &doc);
    drop(prepared);
    assert!(!store.root.exists());
    assert!(store.load().unwrap().is_none());
}

#[test]
fn approval_creates_verified_snapshot_without_rewriting_exchange_fields() {
    let (_temp, store) = store();
    let original = document("v1", "二氧化矽");
    let preview = store.prepare(&shared_lexicon::encode(&original).unwrap(), None).unwrap();
    let approved = store.approve(preview).unwrap();
    assert_eq!(approved.document, original);
    assert_eq!(store.load().unwrap().unwrap().document, original);
    assert!(approved.enabled);
    assert!(store.root.join("versions").join(&approved.checksum).join("receipt.json").exists());
    assert_eq!(fs::read_dir(store.root.join("staging")).unwrap().count(), 0);
}

#[test]
fn whitespace_and_key_order_do_not_change_content_identity() {
    let (_temp, store) = store();
    let original = document("v1", "合成詞");
    let first = store.approve(store.prepare(&shared_lexicon::encode(&original).unwrap(), None).unwrap()).unwrap();
    let compact = serde_json::to_vec(&serde_json::to_value(&original).unwrap()).unwrap();
    let second = store.approve(store.prepare(&compact, Some(&first.revision)).unwrap()).unwrap();
    assert_eq!(first.checksum, second.checksum);
    assert_eq!(first.revision, second.revision);
}

#[test]
fn same_version_different_content_is_rejected_without_changing_active() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    let before = active_bytes(&store);
    assert_eq!(store.prepare(&bytes("v1", "不同註解"), Some(&first.revision)).unwrap_err(), StoreError::VersionConflict);
    assert_eq!(before, active_bytes(&store));
}

#[test]
fn historical_version_names_cannot_be_reused() {
    let (_temp, store) = store();
    deploy(&store, "v1");
    let second = deploy(&store, "v2");
    assert_eq!(store.prepare(&bytes("v1", "改寫舊版本"), Some(&second.revision)).unwrap_err(), StoreError::VersionConflict);
}

#[test]
fn competing_preview_requires_repreview_even_after_rollback() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    let stale = store.prepare(&bytes("v3", "第三版"), Some(&first.revision)).unwrap();
    let second = deploy(&store, "v2");
    let rollback = store.rollback(Some(&second.revision)).unwrap();
    assert_eq!(first.checksum, rollback.checksum);
    assert_ne!(first.revision, rollback.revision);
    assert_eq!(store.approve(stale).unwrap_err(), StoreError::RevisionChanged);
}

#[test]
fn first_import_race_is_detected() {
    let (_temp, store) = store();
    let stale = store.prepare(&bytes("v1", "第一版"), None).unwrap();
    deploy(&store, "v2");
    assert_eq!(store.approve(stale).unwrap_err(), StoreError::RevisionChanged);
}

#[test]
fn approval_is_bound_to_its_store() {
    let (_a, first) = store();
    let (_b, second) = store();
    let preview = first.prepare(&bytes("v1", "第一版"), None).unwrap();
    assert_eq!(second.approve(preview).unwrap_err(), StoreError::WrongStore);
    assert!(!second.root.exists());
}

#[test]
fn rollback_swaps_verified_versions_and_changes_revision() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    assert_eq!(store.rollback(Some(&first.revision)).unwrap_err(), StoreError::NoPreviousVersion);
    let second = deploy(&store, "v2");
    let result = store.rollback(Some(&second.revision)).unwrap();
    assert_eq!(result.checksum, first.checksum);
    assert_eq!(store.status().unwrap().previous.unwrap().checksum, second.checksum);
    assert_ne!(result.revision, first.revision);
    assert_eq!(store.load().unwrap().unwrap().document.dictionary_version, "v1");
}

#[test]
fn disable_and_reimport_preserve_off_state_and_data() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    let off = store.set_enabled(false, Some(&first.revision)).unwrap().unwrap();
    assert!(!off.enabled);
    assert_eq!(off.checksum, first.checksum);
    let same = store.approve(store.prepare(&bytes("v1", "v1"), Some(&off.revision)).unwrap()).unwrap();
    assert!(!same.enabled);
    assert_eq!(same.revision, off.revision);
    let next = deploy(&store, "v2");
    assert!(!next.enabled);
    let on = store.set_enabled(true, Some(&next.revision)).unwrap().unwrap();
    assert!(on.enabled);
}

#[test]
fn enable_requires_an_approved_version() {
    let (_temp, store) = store();
    assert_eq!(store.set_enabled(true, None).unwrap_err(), StoreError::NoActiveVersion);
    assert!(store.load().unwrap().is_none());
}

#[test]
fn failure_at_every_deployment_checkpoint_keeps_active_and_allows_retry() {
    for failure in [Checkpoint::Staged, Checkpoint::Verified, Checkpoint::Activating, Checkpoint::PointerSwap] {
        let (_temp, store) = store();
        let first = deploy(&store, "v1");
        let before = active_bytes(&store);
        let prepared = store.prepare(&bytes("v2", "v2"), Some(&first.revision)).unwrap();
        assert_eq!(store.approve_impl(prepared, |point| if point == failure { Err(StoreError::StorageFailure) } else { Ok(()) }).unwrap_err(), StoreError::StorageFailure);
        assert_eq!(before, active_bytes(&store));
        assert_eq!(store.load().unwrap().unwrap().checksum, first.checksum);
        assert_eq!(fs::read_dir(store.root.join("staging")).unwrap().count(), 0);
        assert_eq!(deploy(&store, "v2").document.dictionary_version, "v2");
    }
}

#[test]
fn failed_pointer_replace_preserves_active_and_rollback_authority() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    let before = active_bytes(&store);
    // A directory is not a legal pointer destination; failure occurs after version staging.
    fs::remove_file(store.root.join("previous-active.json")).unwrap();
    fs::create_dir(store.root.join("previous-active.json")).unwrap();
    let prepared = store.prepare(&bytes("v2", "v2"), Some(&first.revision)).unwrap();
    assert!(store.approve(prepared).is_err());
    assert_eq!(active_bytes(&store), before);
    assert_eq!(store.load().unwrap().unwrap().checksum, first.checksum);
}

#[test]
fn writer_lock_rejects_concurrent_mutation() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    let prepared = store.prepare(&bytes("v2", "v2"), Some(&first.revision)).unwrap();
    let held = store.writer_lock().unwrap();
    assert_eq!(store.approve(prepared).unwrap_err(), StoreError::StoreBusy);
    assert_eq!(store.set_enabled(false, Some(&first.revision)).unwrap_err(), StoreError::StoreBusy);
    drop(held);
    assert_eq!(store.load().unwrap().unwrap().checksum, first.checksum);
}

#[test]
fn corrupt_payload_or_missing_receipt_disables_extension() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    let version = store.root.join("versions").join(&first.checksum);
    let original = fs::read(version.join("lexicon.json")).unwrap();
    fs::write(version.join("lexicon.json"), b"{}").unwrap();
    assert_eq!(store.load().unwrap_err(), StoreError::InvalidStore);
    fs::write(version.join("lexicon.json"), original).unwrap();
    fs::remove_file(version.join("receipt.json")).unwrap();
    assert_eq!(store.load().unwrap_err(), StoreError::InvalidStore);
}

#[test]
fn receipt_copied_from_another_store_is_not_local_approval() {
    let (_a, first_store) = store();
    let (_b, second_store) = store();
    let first = deploy(&first_store, "v1");
    let second = deploy(&second_store, "v1");
    assert_eq!(first.checksum, second.checksum);
    fs::copy(first_store.root.join("versions").join(&first.checksum).join("receipt.json"),
        second_store.root.join("versions").join(&second.checksum).join("receipt.json")).unwrap();
    assert_eq!(second_store.load().unwrap_err(), StoreError::InvalidStore);
}

#[test]
fn corrupted_active_pointer_is_not_silently_overwritten_or_rolled_back() {
    let (_temp, store) = store();
    deploy(&store, "v1");
    let damaged = b"{invalid pointer";
    fs::write(store.root.join("active.json"), damaged).unwrap();
    assert_eq!(store.load().unwrap_err(), StoreError::InvalidStore);
    assert_eq!(store.prepare(&bytes("v2", "v2"), None).unwrap_err(), StoreError::InvalidStore);
    assert!(store.rollback(None).is_err());
    assert_eq!(active_bytes(&store), damaged);
}

#[test]
fn active_pointer_never_resolves_document_supplied_paths() {
    let (_temp, store) = store();
    deploy(&store, "v1");
    let mut pointer: serde_json::Value = serde_json::from_slice(&active_bytes(&store)).unwrap();
    for traversal in ["../outside", "/absolute", "C:\\private", &"A".repeat(64)] {
        pointer["active"]["checksum"] = traversal.into();
        fs::write(store.root.join("active.json"), serde_json::to_vec(&pointer).unwrap()).unwrap();
        assert_eq!(store.load().unwrap_err(), StoreError::InvalidStore);
    }
}

#[test]
fn malformed_and_overlarge_imports_leave_no_store() {
    let (_temp, store) = store();
    assert_eq!(store.prepare(b"{}", None).unwrap_err(), StoreError::InvalidDocument);
    assert_eq!(store.prepare(&vec![b' '; shared_lexicon::MAX_BYTES + 1], None).unwrap_err(), StoreError::InvalidDocument);
    assert!(!store.root.exists());
}

#[test]
fn invalid_previous_snapshot_cannot_replace_current_version() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    let second = deploy(&store, "v2");
    let before = active_bytes(&store);
    fs::write(store.root.join("versions").join(first.checksum).join("receipt.json"), b"{}").unwrap();
    assert_eq!(store.rollback(Some(&second.revision)).unwrap_err(), StoreError::InvalidStore);
    assert_eq!(before, active_bytes(&store));
    assert_eq!(store.load().unwrap().unwrap().checksum, second.checksum);
}

#[test]
fn mirror_is_not_rollback_authority() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    let second = deploy(&store, "v2");
    fs::write(store.root.join("previous-active.json"), b"null").unwrap();
    assert_eq!(store.rollback(Some(&second.revision)).unwrap().checksum, first.checksum);
}

#[cfg(unix)]
#[test]
fn symlinked_payload_and_store_root_are_rejected() {
    use std::os::unix::fs::symlink;
    let (temp, store) = store();
    let first = deploy(&store, "v1");
    let payload = store.root.join("versions").join(&first.checksum).join("lexicon.json");
    let other = temp.path().join("outside.json");
    fs::copy(&payload, &other).unwrap();
    fs::remove_file(&payload).unwrap();
    symlink(&other, &payload).unwrap();
    assert_eq!(store.load().unwrap_err(), StoreError::InvalidStore);
    let linked = temp.path().join("linked-store");
    symlink(&store.root, &linked).unwrap();
    assert_eq!(SharedStore::new(linked).load().unwrap_err(), StoreError::InvalidStore);
}

#[test]
fn forged_source_checksum_cannot_reuse_a_version_name() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    let receipt_path = store.root.join("versions").join(&first.checksum).join("receipt.json");
    let mut receipt: serde_json::Value = serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    receipt["source_checksum"] = digest(&bytes("v1", "不同內容")).into();
    fs::write(receipt_path, serde_json::to_vec(&receipt).unwrap()).unwrap();
    assert_eq!(store.load().unwrap_err(), StoreError::InvalidStore);
    assert_eq!(store.prepare(&bytes("v1", "不同內容"), Some(&first.revision)).unwrap_err(), StoreError::InvalidStore);
}

#[test]
fn duplicate_null_pointer_fields_are_rejected_before_deserialization() {
    let (_temp, store) = store();
    deploy(&store, "v1");
    let original = String::from_utf8(active_bytes(&store)).unwrap();
    assert!(original.contains("\"previous\": null"));
    let duplicate = original.replace("\"previous\": null", "\"previous\": null, \"previous\": null");
    fs::write(store.root.join("active.json"), duplicate).unwrap();
    assert_eq!(store.load().unwrap_err(), StoreError::InvalidStore);
}

#[test]
fn duplicate_escaped_receipt_fields_are_rejected() {
    let (_temp, store) = store();
    let first = deploy(&store, "v1");
    let receipt_path = store.root.join("versions").join(&first.checksum).join("receipt.json");
    let original = String::from_utf8(fs::read(&receipt_path).unwrap()).unwrap();
    let duplicate = original.replacen('{', "{\"format_\\u0076ersion\": 1,", 1);
    fs::write(receipt_path, duplicate).unwrap();
    assert_eq!(store.load().unwrap_err(), StoreError::InvalidStore);
}

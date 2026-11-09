#![cfg(feature = "unified-sqlite")]
use chewing_policy_core::unified_sqlite::{self as sql, UnifiedLexicon};
use rusqlite::Connection;
const INPUT: &[u8] = include_bytes!("fixtures/unified-v1/unified-v1.sqlite");
fn fixture() -> UnifiedLexicon { serde_json::from_slice(include_bytes!("fixtures/unified-v1/unified-v1.json")).unwrap() }
fn altered(statement: &str) -> Vec<u8> {
    let dir = tempfile::tempdir().unwrap(); let p=dir.path().join("fixture.sqlite");
    std::fs::write(&p,INPUT).unwrap();
    { let db=Connection::open(&p).unwrap();db.execute_batch(statement).unwrap(); }
    std::fs::read(p).unwrap()
}
#[test] fn python_fixture_decodes_without_metadata_loss() { assert_eq!(sql::decode(INPUT).unwrap(),fixture()); }
#[test] fn rust_roundtrip_preserves_relations_and_metadata() {
    let value=fixture();let bytes=sql::encode(&value).unwrap();
    assert_eq!(sql::decode(&bytes).unwrap(),value);
}
#[test] fn approved_source_does_not_rewrite_review_metadata() {
    let mut value=fixture(); value.document.document_type="approved".into();
    assert_eq!(sql::decode(&sql::encode(&value).unwrap()).unwrap(),value);
    assert!(!value.document.entries[0].review.user_confirmed);
}
#[test] fn malformed_and_oversized_files_are_rejected() {
    for bytes in [vec![],b"{}".to_vec(),INPUT[..100].to_vec(),vec![0;sql::MAX_BYTES+1]] {assert!(sql::decode(&bytes).is_err());}
}
#[test] fn schema_injections_are_rejected() {
    for change in ["CREATE VIEW surprise AS SELECT * FROM entries", "CREATE TABLE surprise(x)", "CREATE INDEX surprise ON entries(text)", "CREATE TRIGGER surprise AFTER INSERT ON entries BEGIN DELETE FROM relations; END"] {
        assert!(sql::decode(&altered(change)).is_err());
    }
}
#[test] fn incompatible_versions_and_metadata_are_rejected() {
    for change in ["PRAGMA application_id=0", "PRAGMA user_version=2", "UPDATE metadata SET value='2.0.0' WHERE key='schema_version'", "INSERT INTO metadata VALUES ('unknown','x')", "DELETE FROM metadata WHERE key='document_type'"] {
        assert!(sql::decode(&altered(change)).is_err());
    }
}
#[test] fn payload_and_redundant_fields_must_agree() {
    for change in ["UPDATE entries SET text='different' WHERE position=0", "UPDATE entries SET position=10 WHERE position=0", "UPDATE entries SET payload='{}' WHERE position=0", "UPDATE entries SET reading='bad' WHERE position=0"] {
        assert!(sql::decode(&altered(change)).is_err());
    }
}
#[test] fn duplicate_payload_keys_are_rejected() {
    assert!(sql::decode(&altered("UPDATE entries SET payload=substr(payload,1,length(payload)-1)||',\"source\":\"duplicate\"}' WHERE position=0")).is_err());
}
#[test] fn invalid_relations_are_rejected() {
    for change in ["UPDATE relations SET kind='replace'", "UPDATE relations SET source_id='missing'", "UPDATE relations SET target_id=source_id WHERE target_id=(SELECT target_id FROM relations LIMIT 1)", "UPDATE relations SET source_id=target_id"] {
        assert!(sql::decode(&altered(change)).is_err());
    }
}
#[test] fn encode_rejects_duplicate_entries_and_relations() {
    let mut value=fixture();value.document.entries.push(value.document.entries[0].clone());assert!(sql::encode(&value).is_err());
    let mut value=fixture();value.relations.push(value.relations[0].clone());assert!(sql::encode(&value).is_err());
}
#[test] fn relation_does_not_change_chinese_candidate_output() {
    let value=sql::decode(INPUT).unwrap();
    let index=chewing_policy_core::shared_lexicon::LexiconIndex::new(&value.document).unwrap();
    let candidates=index.chinese("ㄐㄧㄚˇ-ㄨㄢˊ");assert_eq!(candidates.len(),1);assert_eq!(candidates[0].text,"甲烷");
}
#[test] fn empty_dictionary_roundtrips() {
    let mut value=fixture();value.document.entries.clear();value.relations.clear();
    assert_eq!(sql::decode(&sql::encode(&value).unwrap()).unwrap(),value);
}

#![cfg(feature="unified-sqlite")]
use chewing_policy_core::{unified_sqlite as sql,unified_management as manage,shared_lexicon_store::{SharedStore,StoreError}};
fn fixture()->sql::UnifiedLexicon{serde_json::from_slice(include_bytes!("fixtures/unified-v1/unified-v1.json")).unwrap()}
fn setup()->(tempfile::TempDir,SharedStore){let t=tempfile::tempdir().unwrap();let s=SharedStore::new(t.path().join("store"));(t,s)}
fn approve(s:&SharedStore,v:sql::UnifiedLexicon,rev:Option<&str>){let hash=manage::digest(&v).unwrap();s.approve(s.prepare_unified(v,rev,&hash).unwrap()).unwrap();}
#[test]fn preview_cancel_is_read_only_and_source_approved_is_not_local_approval(){
 let(t,s)=setup();let mut v=fixture();v.document.document_type="approved".into();let h=manage::digest(&v).unwrap();let p=s.prepare_unified(v,None,&h).unwrap();drop(p);assert!(!t.path().join("store").exists());assert!(s.load().unwrap().is_none());
}
#[test]fn approval_preserves_relations_clears_foreign_usage_and_exports_full_snapshot(){
 let(_t,s)=setup();approve(&s,fixture(),None);let a=s.load_unified().unwrap().unwrap();
 assert_eq!(a.value.relations.len(),2);assert!(a.value.document.entries.iter().all(|e|e.usage.is_none()&&e.review.user_confirmed));assert!(a.value.document.dictionary_version.starts_with("snapshot-"));assert!(!a.recovered);
 let bytes=sql::encode(&a.value).unwrap();assert_eq!(sql::decode(&bytes).unwrap(),a.value);
}
#[test]fn stale_preview_cannot_overwrite_another_approval(){
 let(_t,s)=setup();let v=fixture();let hash=manage::digest(&v).unwrap();let old=s.prepare_unified(v.clone(),None,&hash).unwrap();approve(&s,v,None);assert_eq!(s.approve(old).unwrap_err(),StoreError::RevisionChanged);
}
#[test]fn primary_corruption_uses_complete_fallback_then_rejects_tampered_fallback(){
 let(t,s)=setup();approve(&s,fixture(),None);let before=s.load_unified().unwrap().unwrap();let d=t.path().join("store/versions").join(&before.snapshot.checksum);
 std::fs::write(d.join("lexicon.sqlite"),b"corrupt").unwrap();let after=s.load_unified().unwrap().unwrap();assert!(after.recovered);assert_eq!(before.value,after.value);
 std::fs::write(d.join("unified.json"),b"{}").unwrap();assert!(s.load().is_err());
}
#[test]fn rollback_restores_relations_and_disabled_state_is_preserved(){
 let(_t,s)=setup();approve(&s,fixture(),None);let first=s.load_unified().unwrap().unwrap();
 let mut next=first.value.clone();next.document.entries.clear();next.relations.clear();approve(&s,next,Some(&first.snapshot.revision));let second=s.load().unwrap().unwrap();
 s.set_enabled(false,Some(&second.revision)).unwrap();let disabled=s.load().unwrap().unwrap();s.rollback(Some(&disabled.revision)).unwrap();let restored=s.load_unified().unwrap().unwrap();assert_eq!(restored.value,first.value);assert!(!restored.snapshot.enabled);
}
#[test]fn json_legacy_is_readable_but_old_editor_cannot_drop_new_relations(){
 let(_t,s)=setup();let v=fixture();let bytes=chewing_policy_core::shared_lexicon::encode(&v.document).unwrap();s.approve(s.prepare(&bytes,None).unwrap()).unwrap();let old=s.load_unified().unwrap().unwrap();assert!(old.value.relations.is_empty());
 approve(&s,v,Some(&old.snapshot.revision));let current=s.load().unwrap().unwrap();assert!(s.prepare(&bytes,Some(&current.revision)).is_err());
}
#[test]fn partial_selection_closes_relationships_and_export_versions_follow_contents(){
 let v=fixture();let id=v.document.entries[1].entry_id.clone();let all=manage::selected(&v,Some(&[id])).unwrap();assert_eq!(all,v);
 let full=manage::export_snapshot(&v,None).unwrap();let empty=manage::export_snapshot(&v,Some(&[])).unwrap();assert_ne!(full.document.dictionary_version,empty.document.dictionary_version);
}
#[test]fn edit_requires_confirmed_alias_reading_and_delete_only_removes_managed_targets(){
 let base=manage::empty();let edit=manage::Edit{term_type:chewing_policy_core::shared_lexicon::TermType::TechnicalAlias,reading:"h2o".into(),text:"H₂O".into(),annotation:"".into(),alias:"水".into(),alias_reading:"".into()};assert!(manage::edit(&base,Some(edit.clone()),None,&[]).is_err());
 let mut edit=edit;edit.alias_reading="ㄕㄨㄟˇ".into();let v=manage::edit(&base,Some(edit),None,&[]).unwrap();assert_eq!(v.document.entries.len(),2);assert_eq!(v.relations.len(),1);
 let id=v.relations[0].target_id.clone();let deleted=manage::edit(&v,None,None,&[id]).unwrap();assert_eq!(deleted.document.entries.len(),1);assert!(deleted.relations.is_empty());
}
#[test]fn partial_reimport_preserves_source_identity_and_does_not_duplicate(){
 let(_t,s)=setup();let v=fixture();let source=manage::digest(&v).unwrap();let first=manage::selected(&v,Some(&[v.document.entries[1].entry_id.clone()])).unwrap();s.approve(s.prepare_unified(first,None,&source).unwrap()).unwrap();let current=s.load_unified().unwrap().unwrap();
 let (merged,_)=manage::merge(&current.value,&v,&[]).unwrap();s.approve(s.prepare_unified(merged,Some(&current.snapshot.revision),&source).unwrap()).unwrap();let after=s.load_unified().unwrap().unwrap();assert_eq!(after.snapshot.revision,current.snapshot.revision);assert_eq!(after.value.document.entries.len(),3);
}

#[test]fn explicit_disable_survives_payload_damage(){
 let(t,s)=setup();approve(&s,fixture(),None);let current=s.load().unwrap().unwrap();s.set_enabled(false,Some(&current.revision)).unwrap();let disabled=s.load().unwrap().unwrap();let dir=t.path().join("store/versions").join(&disabled.checksum);
 std::fs::write(dir.join("lexicon.sqlite"),b"broken").unwrap();std::fs::write(dir.join("unified.json"),b"broken").unwrap();assert!(s.load().is_err());assert_eq!(s.disabled_revision().unwrap().as_deref(),Some(disabled.revision.as_str()));
}

#[test]fn editing_linked_entries_preserves_all_relations_and_deduplicates_aliases(){
 let base=fixture();let old=&base.document.entries[0];
 let change=manage::Edit{term_type:old.term_type,reading:old.reading.clone(),text:"甲完".into(),annotation:"".into(),alias:"".into(),alias_reading:"".into()};
 let updated=manage::edit(&base,Some(change),Some(&old.entry_id),&[]).unwrap();
 assert_eq!(updated.relations.len(),2);assert!(updated.relations.iter().all(|r|r.source_id!=old.entry_id));
 let target=&base.document.entries[1];
 let change=manage::Edit{term_type:target.term_type,reading:target.reading.clone(),text:target.text.clone(),annotation:"更新註解".into(),alias:old.text.clone(),alias_reading:old.reading.clone()};
 let updated=manage::edit(&base,Some(change),Some(&target.entry_id),&[]).unwrap();
 assert_eq!(updated.relations,base.relations);assert_eq!(updated.document.entries.len(),3);
}
#[test]fn linked_type_change_is_rejected_instead_of_dropping_relations(){
 let base=fixture();let old=&base.document.entries[0];let change=manage::Edit{term_type:chewing_policy_core::shared_lexicon::TermType::EnglishTerm,reading:"methane".into(),text:"methane".into(),annotation:"".into(),alias:"".into(),alias_reading:"".into()};
 assert!(manage::edit(&base,Some(change),Some(&old.entry_id),&[]).is_err());
}

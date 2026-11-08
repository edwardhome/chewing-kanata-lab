use chewing_policy_core::shared_lexicon::*;
const FIXTURE:&[u8]=include_bytes!("../../tests/shared-lexicon/fixtures/valid-v1.json");
fn fixture()->LexiconDocument { parse(FIXTURE).unwrap() }
fn reset_id(e:&mut LexiconEntry) { e.entry_id=entry_id(e.term_type,&e.reading,&e.text); }
#[test]
fn fixed_vectors_and_semantic_round_trip() {
    let doc=fixture(); assert_eq!(doc.entries.len(),7);
    for e in &doc.entries { assert_eq!(e.entry_id,entry_id(e.term_type,&e.reading,&e.text)); }
    assert_eq!(doc,parse(&encode(&doc).unwrap()).unwrap());
    assert_ne!(entry_id(TermType::EnglishTerm,"FinFET","FinFET"),entry_id(TermType::EnglishTerm,"finfet","FinFET"));
    assert_ne!(entry_id(TermType::UnicodeMapping,"aa","é"),entry_id(TermType::UnicodeMapping,"aa","e\u{301}"));
}
#[test]
fn duplicate_optional_null_unknown_fields_and_json_structure_rejected() {
    let raw=String::from_utf8(FIXTURE.to_vec()).unwrap();
    for injected in ["\"annotation\":null,\"annotation\":null,", "\"annotation\":null,\"annot\\u0061tion\":\"x\",", "\"zh_aliases\":[],"] {
        let bad=raw.replacen("\"entry_id\":",&format!("{injected}\"entry_id\":"),1);
        assert!(parse(bad.as_bytes()).is_err());
    }
    assert!(parse(raw.replacen("\"status\":", "\"unknown\":1,\"status\":",1).as_bytes()).is_err());
    assert!(parse(&FIXTURE[..FIXTURE.len()/2]).is_err());
    assert!(parse(format!("{raw} null").as_bytes()).is_err());
    let deep=format!("{}0{}","[".repeat(13),"]".repeat(13));
    assert_eq!(parse(deep.as_bytes()).unwrap_err().0,"JSON depth exceeds 12");
    let depth12=format!("{}0{}","[".repeat(12),"]".repeat(12));
    assert_eq!(parse(depth12.as_bytes()).unwrap_err().0,"invalid JSON fields");
    assert!(parse(&vec![b' ';MAX_BYTES+1]).is_err());
}
#[test]
fn matching_is_exact_and_never_promotes_rejected_or_uses_annotations() {
    let mut doc=fixture(); doc.entries[0].annotation=Some("中文別名".into());
    let index=LexiconIndex::new(&doc).unwrap();
    assert_eq!(index.chinese("ㄌㄟˇ-ㄐㄧㄥ").len(),1);
    assert!(index.chinese("中文別名").is_empty());
    assert!(index.chinese("ㄌㄟ-ㄐㄧㄥ").is_empty());
    assert_eq!(index.engineering("FINFET").len(),2);
    assert_eq!(index.engineering("h2o").len(),2);
    doc.entries[0].review.status=ReviewStatus::Rejected;
    let next=LexiconIndex::new(&doc).unwrap();
    assert!(next.chinese("ㄌㄟˇ-ㄐㄧㄥ").is_empty());
    assert_eq!(index.chinese("ㄌㄟˇ-ㄐㄧㄥ").len(),1); // prior snapshot remains immutable
}
#[test]
fn merge_preserves_conflicts_usage_and_distinct_case_identities() {
    let base=fixture(); let mut incoming=base.clone();
    incoming.entries[0].annotation=Some("另有說明".into());
    incoming.entries[1].usage=Some(Usage{commit_count:5,manual_selection_count:2});
    let preview=merge_preview(&base,&incoming).unwrap();
    assert_eq!(preview.entries,base.entries); assert_eq!(preview.added,0);
    assert_eq!(preview.conflicts.len(),2); assert_eq!(preview.identical,5);
    assert_eq!(preview.case_collisions.len(),1);
    assert!(preview.entries[1].usage.is_none());
    incoming.entries.push(incoming.entries[0].clone());
    assert!(incoming.validate().is_ok());
    incoming.entries.last_mut().unwrap().annotation=Some("矛盾".into());
    assert!(incoming.validate().is_err());
}
#[test]
fn unicode_limits_syllables_and_usage_match_mac_contract() {
    let base=fixture();
    for (reading,text,valid) in [("ㄧ˙","一",true),("ㄧ","𠀀",true),("一","一",false),("˙ㄧ","一",false),("ㄧ-","一",false),("ㄅㄆ","一",false)] {
        let mut e=base.entries[0].clone(); e.reading=reading.into();e.text=text.into();reset_id(&mut e);
        assert_eq!(e.validate().is_ok(),valid,"{reading}");
    }
    let mut e=base.entries[1].clone();
    e.annotation=Some("𠀀".repeat(40)); assert!(e.validate().is_ok());
    e.annotation=Some("𠀀".repeat(41)); assert!(e.validate().is_err());
    for unsafe_text in ["a\u{200b}","a\u{2028}","a\n"] { e.annotation=Some(unsafe_text.into());assert!(e.validate().is_err()); }
    e.annotation=None; e.usage=Some(Usage{commit_count:1,manual_selection_count:2});assert!(e.validate().is_err());
}
#[test]
fn maximum_entry_index_retains_all_candidates_and_stable_order() {
    let mut doc=fixture(); let template=doc.entries[1].clone();doc.entries.clear();
    for i in 0..MAX_ENTRIES { let mut e=template.clone();e.text=format!("synthetic-{i}");reset_id(&mut e);doc.entries.push(e); }
    let build_start=std::time::Instant::now();
    let index=LexiconIndex::new(&doc).unwrap();
    let build_elapsed=build_start.elapsed();
    let lookup_start=std::time::Instant::now();
    let found=index.engineering("h2o"); assert_eq!(found.len(),MAX_ENTRIES);
    assert_eq!(found[0].text,"synthetic-0");assert_eq!(found[MAX_ENTRIES-1].text,"synthetic-19999");
    let snapshot=chewing_policy_core::candidate_snapshot::CandidateSnapshot::new_with_shared(
        1,"h2o",&[],&[],&found,10).unwrap();
    assert_eq!(snapshot.items().len(),MAX_ENTRIES);
    assert_eq!(snapshot.page_count(),2000);
    eprintln!("synthetic20k index_build={build_elapsed:?} lookup_and_snapshot={:?}",lookup_start.elapsed());
    doc.entries.push(template);assert!(doc.validate().is_err());
}
#[test]
fn shared_candidate_actions_retain_provenance_and_never_unlearn() {
    use chewing_policy_core::candidate_snapshot::*;
    use chewing_policy_core::engineering_lexicon::TechnicalCandidate;
    let doc=fixture();
    let index=LexiconIndex::new(&doc).unwrap();
    let mut shared=index.engineering("h2o");
    shared.retain(|entry|entry.text=="H₂O");
    shared.extend(index.engineering("sio2"));
    let engineering=vec![TechnicalCandidate{text:"H₂O".into(),annotation:None}];
    let snapshot=CandidateSnapshot::new_with_shared(1,"原文",&["原生".into()],&engineering,&shared,2).unwrap();
    assert_eq!(snapshot.items().len(),3);
    assert_eq!(snapshot.items()[1].shared_entry_ids(), &[doc.entries[1].entry_id.clone()]);
    let selected=snapshot.resolve(1,0,1,"原文").unwrap();
    assert_eq!(selected.source(),CandidateSource::Shared);
    assert!(matches!(selected.action(),CandidateAction::Shared{scope:ReplacementScope::WholeComposition{utf16_len:2},..}));
    assert!(snapshot.native_unlearn_index(1,0,1,"原文").is_none());
    assert!(snapshot.resolve(1,0,2,"原文").is_none());
}
#[test]
fn engineering_shared_snapshot_injection_preserves_original_and_can_be_removed() {
    use chewing_policy_core::technical_lexicon::TechnicalLexicon;
    use std::sync::Arc;
    let mut doc=fixture(); let e=&mut doc.entries[1];e.reading="zzsynthetic".into();reset_id(e);
    let index=Arc::new(LexiconIndex::new(&doc).unwrap());
    let old=TechnicalLexicon::default();
    assert!(old.candidate_items("zzsynthetic").is_empty());
    let new=old.clone().with_shared_index(Some(index));
    assert_eq!(new.candidate_items("zzsynthetic")[0].text,"H₂O");
    assert!(new.with_shared_index(None).candidate_items("zzsynthetic").is_empty());
}

#[test]
fn agy_negative_exchange_fixtures_are_rejected() {
 let cases:serde_json::Value=serde_json::from_str(include_str!("../../tests/shared-lexicon/fixtures/agy-negative-cases.json")).unwrap();
 let cases=cases.as_array().unwrap();assert_eq!(cases.len(),8);
 for case in cases {assert!(parse(case["json"].as_str().unwrap().as_bytes()).is_err(),"case {} unexpectedly accepted",case["name"]);}
}

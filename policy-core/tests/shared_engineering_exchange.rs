use chewing_policy_core::shared_engineering_exchange::*;
use chewing_policy_core::shared_lexicon::{parse,encode,ReviewStatus};
use chewing_policy_core::user_engineering::UserEngineeringTerm;
fn term()->UserEngineeringTerm { UserEngineeringTerm{input:"CH4".into(),output:"CH₄".into(),annotation:"甲烷".into(),zh_aliases:Vec::new()} }
#[test]
fn export_preserves_exact_data_and_never_grants_approval() {
    let rows=vec![term()];let before=rows.clone();
    let preview=preview_engineering_export(&rows,"synthetic-1",false).unwrap();
    let doc=preview.document.unwrap();let e=&doc.entries[0];
    assert_eq!(e.reading,"CH4");assert_eq!(e.text,"CH₄");assert_eq!(e.annotation.as_deref(),Some("甲烷"));
    assert_eq!(e.source,"user_engineering_export");assert_eq!(e.review.status,ReviewStatus::Proposed);assert!(!e.review.user_confirmed);
    assert!(e.usage.is_none());assert_eq!(doc.document_type,"review");
    assert_eq!(parse(&encode(&doc).unwrap()).unwrap(),doc);assert_eq!(rows,before);
}
#[test]
fn aliases_require_explicit_omission_and_report_exact_loss() {
    let mut row=term();row.zh_aliases=vec!["甲烷".into(),"沼氣".into()];let rows=vec![row];let before=rows.clone();
    let blocked=preview_engineering_export(&rows,"v1",false).unwrap();
    assert!(blocked.document.is_none());assert!(blocked.omitted_aliases.is_empty());assert_eq!(blocked.unsupported[0].reason,UnsupportedReason::ChineseAliasesRequireOmissionApproval);
    let accepted=preview_engineering_export(&rows,"v1",true).unwrap();assert!(accepted.document.is_some());
    assert_eq!(accepted.omitted_aliases[0].aliases,rows[0].zh_aliases);assert_eq!(accepted.omitted_aliases[0].index,0);assert_eq!(rows,before);
}
#[test]
fn utf16_annotation_limit_is_reported_not_truncated() {
    let mut row=term();row.annotation="𠀀".repeat(41);
    let mut other=term();other.input="other-code".into();
    let preview=preview_engineering_export(&[row.clone(),other],"v1",false).unwrap();assert!(preview.document.is_none());
    assert_eq!(preview.unsupported[0].reason,UnsupportedReason::AnnotationExceedsUtf16Limit);
    row.annotation="𠀀".repeat(40);assert!(preview_engineering_export(&[row.clone()],"v1",false).unwrap().document.is_some());
    row.annotation.clear();assert!(preview_engineering_export(&[row],"v1",false).unwrap().document.unwrap().entries[0].annotation.is_none());
}
#[test]
fn original_row_and_byte_limits_apply_before_export() {
    let mut rows:Vec<_>=(0..500).map(|i|{let mut t=term();t.input=format!("synthetic{i}");t}).collect();
    assert_eq!(preview_engineering_export(&rows,"v1",false).unwrap().document.unwrap().entries.len(),500);
    let mut extra=term();extra.input="extra".into();rows.push(extra);assert!(preview_engineering_export(&rows,"v1",false).is_err());
    rows.pop();for row in &mut rows { row.output="漢".repeat(128);row.annotation="註".repeat(80); }
    assert!(preview_engineering_export(&rows,"v1",false).is_err());
}

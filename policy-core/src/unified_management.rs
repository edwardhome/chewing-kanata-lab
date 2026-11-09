//! Pure management operations; no writes, model calls, or implicit approval.
use crate::shared_lexicon::{self as lex, LexiconDocument, LexiconEntry, TermType, Review, ReviewStatus, EntryConflict};
use crate::unified_sqlite::{UnifiedLexicon, Relation, ExchangeError};
use serde::{Serialize,Deserialize};
use sha2::{Sha256,Digest};
use std::collections::{HashSet,BTreeSet};
type Result<T> = std::result::Result<T,ExchangeError>;
pub fn empty() -> UnifiedLexicon { UnifiedLexicon {document:LexiconDocument {format:"ai-bopomofo.lexicon".into(),schema_version:"1.0.0".into(),document_type:"review".into(),dictionary_version:"empty-local".into(),entries:vec![]},relations:vec![]} }
pub fn digest(value:&UnifiedLexicon) -> Result<String> { value.validate()?;Ok(format!("{:x}",Sha256::digest(serde_json::to_vec(value)?))) }
pub fn selected(value:&UnifiedLexicon, selected:Option<&[String]>) -> Result<UnifiedLexicon> {
    value.validate()?;
    let Some(selected)=selected else {return Ok(value.clone());};
    let valid:HashSet<_>=value.document.entries.iter().map(|e|e.entry_id.as_str()).collect();
    let mut ids:HashSet<_>=selected.iter().map(String::as_str).collect();
    if ids.len()!=selected.len() || ids.iter().any(|id|!valid.contains(id)) {return Err(ExchangeError("invalid selection"));}
    loop {
        let before=ids.len();
        for r in &value.relations {if ids.contains(r.source_id.as_str()) || ids.contains(r.target_id.as_str()) {ids.insert(&r.source_id);ids.insert(&r.target_id);}}
        if ids.len()==before {break;}
    }
    let mut result=value.clone();
    result.document.entries.retain(|e|ids.contains(e.entry_id.as_str()));
    result.relations.retain(|r|ids.contains(r.source_id.as_str())&&ids.contains(r.target_id.as_str()));
    Ok(result)
}
pub fn merge(base:&UnifiedLexicon,incoming:&UnifiedLexicon,choose_incoming:&[String]) -> Result<(UnifiedLexicon,Vec<EntryConflict>)> {
    base.validate()?;incoming.validate()?;
    let preview=lex::merge_preview(&base.document,&incoming.document)?;
    let allowed:HashSet<_>=preview.conflicts.iter().map(|c|c.incoming.entry_id.as_str()).collect();
    let choice:HashSet<_>=choose_incoming.iter().map(String::as_str).collect();
    if choice.len()!=choose_incoming.len() || choice.iter().any(|id|!allowed.contains(id)) {return Err(ExchangeError("invalid conflict selection"));}
    let mut entries=preview.entries;
    for conflict in &preview.conflicts {if choice.contains(conflict.incoming.entry_id.as_str()) {if let Some(e)=entries.iter_mut().find(|e|e.entry_id==conflict.incoming.entry_id){*e=conflict.incoming.clone();}}}
    let mut value=base.clone();value.document.entries=entries;
    let mut seen=BTreeSet::new();
    value.relations=base.relations.iter().chain(&incoming.relations).filter(|r|seen.insert((r.source_id.clone(),r.target_id.clone(),r.kind.clone()))).cloned().collect();
    value.validate()?;Ok((value,preview.conflicts))
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edit {pub term_type:TermType,pub reading:String,pub text:String,pub annotation:String,pub alias:String,pub alias_reading:String}
fn entry(kind:TermType,reading:String,text:String,annotation:Option<String>)->LexiconEntry {
    LexiconEntry{entry_id:lex::entry_id(kind,&reading,&text),term_type:kind,reading,text,source:"manual-management".into(),usage:None,review:Review{status:ReviewStatus::Proposed,user_confirmed:false},model_suggestion:None,category:None,annotation,sources:None,policy:Some("candidate_only".into())}
}
pub fn edit(base:&UnifiedLexicon,edit:Option<Edit>,old_id:Option<&str>,delete_ids:&[String])->Result<UnifiedLexicon> {
    base.validate()?;let mut value=base.clone();
    let mut remove:HashSet<&str>=delete_ids.iter().map(String::as_str).collect();
    if remove.len()!=delete_ids.len(){return Err(ExchangeError("duplicate deletion"));}
    if let Some(id)=old_id {remove.insert(id);}
    if remove.iter().any(|id|!base.document.entries.iter().any(|e|e.entry_id==*id)){return Err(ExchangeError("stale edit target"));}
    value.document.entries.retain(|e|!remove.contains(e.entry_id.as_str()));
    value.relations.retain(|r|!remove.contains(r.source_id.as_str())&&!remove.contains(r.target_id.as_str()));
    if let Some(edit)=edit {
        if edit.alias.is_empty()!=edit.alias_reading.is_empty(){return Err(ExchangeError("alias requires confirmed reading"));}
        let output=entry(edit.term_type,edit.reading,edit.text,if edit.annotation.is_empty(){None}else{Some(edit.annotation)});
        output.validate()?;
        if value.document.entries.iter().any(|e|e.entry_id==output.entry_id){return Err(ExchangeError("entry already exists"));}
        // Editing a managed term keeps every existing relation, including multiple
        // Chinese names. New alias fields add a relation; deletion is explicit.
        if let Some(old_id)=old_id {
            let old=base.document.entries.iter().find(|e|e.entry_id==old_id).unwrap();
            if old.term_type!=output.term_type && base.relations.iter().any(|r|r.source_id==old_id||r.target_id==old_id) {
                return Err(ExchangeError("linked entry type cannot change"));
            }
            for relation in &base.relations {
                if relation.source_id!=old_id && relation.target_id!=old_id {continue;}
                if delete_ids.iter().any(|id|id==&relation.source_id||id==&relation.target_id){continue;}
                let mut relation=relation.clone();
                if relation.source_id==old_id {relation.source_id=output.entry_id.clone();}
                if relation.target_id==old_id {relation.target_id=output.entry_id.clone();}
                value.relations.push(relation);
            }
        }
        if !edit.alias.is_empty(){
            if edit.term_type==TermType::ZhPhrase {return Err(ExchangeError("Chinese phrase cannot target an alias relation"));}
            let alias=entry(TermType::ZhPhrase,edit.alias_reading,edit.alias,None);alias.validate()?;
            value.relations.push(Relation{source_id:alias.entry_id.clone(),target_id:output.entry_id.clone(),kind:"equivalent".into()});
            if !value.document.entries.iter().any(|e|e.entry_id==alias.entry_id){value.document.entries.push(alias);}
        }
        if let Some(id)=old_id {
            let original_position=base.document.entries.iter().position(|e|e.entry_id==id).unwrap();
            value.document.entries.insert(original_position.min(value.document.entries.len()),output);
        } else {value.document.entries.push(output);}
    }
    let mut seen=BTreeSet::new();
    value.relations.retain(|r|seen.insert((r.source_id.clone(),r.target_id.clone(),r.kind.clone())));
    value.relations.sort_by(|a,b|(&a.source_id,&a.target_id,&a.kind).cmp(&(&b.source_id,&b.target_id,&b.kind)));
    value.validate()?;Ok(value)
}
pub fn export_snapshot(value:&UnifiedLexicon,ids:Option<&[String]>) -> Result<UnifiedLexicon> {
    let mut value=selected(value,ids)?;
    value.document.dictionary_version="snapshot-content".into();
    value.relations.sort_by(|a,b|(&a.source_id,&a.target_id,&a.kind).cmp(&(&b.source_id,&b.target_id,&b.kind)));
    value.document.dictionary_version=format!("snapshot-{}",digest(&value)?);
    Ok(value)
}

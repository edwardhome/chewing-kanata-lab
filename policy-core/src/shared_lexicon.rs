//! Version 1 exchange data only. Source approval is never local deployment authority.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use unicode_general_category::{get_general_category, GeneralCategory};

pub const MAX_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 20_000;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconError(pub &'static str);
impl std::fmt::Display for LexiconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.0) }
}
impl std::error::Error for LexiconError {}
type Result<T> = std::result::Result<T, LexiconError>;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TermType {
    #[serde(rename="zh_phrase")] ZhPhrase,
    #[serde(rename="english_term")] EnglishTerm,
    #[serde(rename="technical_alias")] TechnicalAlias,
    #[serde(rename="unicode_mapping")] UnicodeMapping,
}
impl TermType {
    pub fn as_str(self) -> &'static str { match self { Self::ZhPhrase=>"zh_phrase", Self::EnglishTerm=>"english_term", Self::TechnicalAlias=>"technical_alias", Self::UnicodeMapping=>"unicode_mapping" } }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ReviewStatus { Observed, Proposed, Approved, Rejected }
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage { pub commit_count: i64, pub manual_selection_count: i64 }
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review { pub status: ReviewStatus, pub user_confirmed: bool }
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSuggestion {
    pub suggestion_id: String, pub origin_entry_id: String, pub reason: String, pub model_id: String,
    #[serde(skip_serializing_if="Option::is_none")] pub model_version: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")] pub suggested_annotation: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")] pub suggested_category: Option<String>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexiconEntry {
    pub entry_id: String, pub term_type: TermType, pub reading: String, pub text: String, pub source: String,
    #[serde(skip_serializing_if="Option::is_none")] pub usage: Option<Usage>,
    pub review: Review,
    #[serde(skip_serializing_if="Option::is_none")] pub model_suggestion: Option<ModelSuggestion>,
    #[serde(skip_serializing_if="Option::is_none")] pub category: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")] pub annotation: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")] pub sources: Option<Vec<String>>,
    #[serde(skip_serializing_if="Option::is_none")] pub policy: Option<String>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexiconDocument {
    pub format: String, pub schema_version: String, pub document_type: String,
    pub dictionary_version: String, pub entries: Vec<LexiconEntry>,
}
impl std::fmt::Debug for LexiconDocument {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { f.debug_struct("LexiconDocument").field("entry_count",&self.entries.len()).finish_non_exhaustive() }
}
impl std::fmt::Debug for LexiconEntry {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { f.debug_struct("LexiconEntry").field("term_type",&self.term_type).finish_non_exhaustive() }
}
pub fn entry_id(kind: TermType, reading: &str, text: &str) -> String {
    let mut digest=Sha256::new();
    for value in [kind.as_str(),reading,text] { digest.update((value.len() as u64).to_be_bytes()); digest.update(value.as_bytes()); }
    format!("{:x}",digest.finalize())
}
fn safe(s:&str,max:usize)->bool {
    !s.is_empty() && s.encode_utf16().count()<=max && s.chars().all(|c| !matches!(get_general_category(c),GeneralCategory::Control|GeneralCategory::Format|GeneralCategory::LineSeparator|GeneralCategory::ParagraphSeparator))
}
fn optional_safe(s:&Option<String>,max:usize)->bool { s.as_ref().map_or(true,|s|safe(s,max)) }
fn valid_syllable(s:&str)->bool {
    let mut chars=s.chars().peekable(); let mut phonetic=false;
    for (lo,hi) in [('ㄅ','ㄙ'),('ㄧ','ㄩ'),('ㄚ','ㄦ')] {
        if chars.peek().is_some_and(|c| (lo..=hi).contains(c)) { chars.next(); phonetic=true; }
    }
    if chars.peek().is_some_and(|c| matches!(c,'ˊ'|'ˇ'|'ˋ'|'˙')) { chars.next(); }
    phonetic && chars.next().is_none()
}
impl LexiconEntry {
    pub fn validate(&self)->Result<()> {
        if self.entry_id!=entry_id(self.term_type,&self.reading,&self.text) || !safe(&self.text,128) || !safe(&self.source,80)
            || !optional_safe(&self.annotation,80) || !optional_safe(&self.category,80)
            || !self.sources.as_ref().map_or(true,|v|v.len()<=16 && v.iter().all(|s|safe(s,512)))
            || !self.policy.as_ref().map_or(true,|p|p=="candidate_only")
            || !self.usage.as_ref().map_or(true,|u|u.commit_count>=0 && u.manual_selection_count>=0 && u.manual_selection_count<=u.commit_count) {
            return Err(LexiconError("invalid entry identity or fields"));
        }
        if self.term_type==TermType::ZhPhrase {
            let syllables:Vec<_>=self.reading.split('-').collect();
            if !(1..=8).contains(&syllables.len()) || syllables.len()!=self.text.chars().count()
                || !self.text.chars().all(|c|(0x3400..=0x9fff).contains(&(c as u32)) || (0x20000..=0x323af).contains(&(c as u32)))
                || !syllables.iter().all(|s|valid_syllable(s)) { return Err(LexiconError("invalid Chinese reading or span")); }
        } else if !(2..=64).contains(&self.reading.len()) || !self.reading.bytes().all(|b|(33..=126).contains(&b)) {
            return Err(LexiconError("invalid ASCII reading"));
        }
        if let Some(s)=&self.model_suggestion {
            // Require the published SHA-256 ID shape; Swift currently only counts
            // characters here. This deliberately rejects malformed source IDs.
            if !safe(&s.suggestion_id,80) || s.origin_entry_id.len()!=64 || !s.origin_entry_id.bytes().all(|b|b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || !safe(&s.reason,512) || !safe(&s.model_id,128) || !optional_safe(&s.model_version,128)
                || !optional_safe(&s.suggested_annotation,80) || !optional_safe(&s.suggested_category,80) { return Err(LexiconError("invalid model suggestion")); }
        }
        Ok(())
    }
}
impl LexiconDocument {
    pub fn validate(&self)->Result<()> {
        if self.format!="ai-bopomofo.lexicon" || self.schema_version!="1.0.0" || !["review","approved"].contains(&self.document_type.as_str())
            || !safe(&self.dictionary_version,80) || self.entries.len()>MAX_ENTRIES { return Err(LexiconError("unsupported document or limit")); }
        let mut seen=HashMap::new();
        for entry in &self.entries { entry.validate()?; if let Some(old)=seen.insert(&entry.entry_id,entry) { if old!=entry { return Err(LexiconError("conflicting duplicate entry ID")); } } }
        Ok(())
    }
}
// Bound nesting before allocating a JSON tree, and reject duplicate decoded keys,
// including repeated optional fields whose first value was null.
struct Scanner<'a> { bytes:&'a [u8], pos:usize }
impl Scanner<'_> {
    fn space(&mut self) { while self.bytes.get(self.pos).is_some_and(|b|matches!(b,b' '|b'\n'|b'\r'|b'\t')) { self.pos+=1; } }
    fn require(&mut self,b:u8)->Result<()> { self.space(); if self.bytes.get(self.pos)!=Some(&b) { return Err(LexiconError("invalid JSON structure")); } self.pos+=1; Ok(()) }
    fn string(&mut self)->Result<String> {
        self.space(); let start=self.pos; self.require(b'"')?;
        while let Some(&b)=self.bytes.get(self.pos) { self.pos+=1; if b==b'\\' { self.pos+=1; } else if b==b'"' { return serde_json::from_slice(&self.bytes[start..self.pos]).map_err(|_|LexiconError("invalid JSON string")); } }
        Err(LexiconError("truncated JSON string"))
    }
    fn value(&mut self,depth:usize)->Result<()> {
        if depth>12 { return Err(LexiconError("JSON depth exceeds 12")); } self.space();
        let b=*self.bytes.get(self.pos).ok_or(LexiconError("truncated JSON"))?;
        if b==b'"' { self.string()?; return Ok(()); }
        if b==b'{' || b==b'[' {
            self.pos+=1; let object=b==b'{'; let end=if object { b'}' } else { b']' }; self.space();
            if self.bytes.get(self.pos)==Some(&end) { self.pos+=1; return Ok(()); }
            let mut keys=HashSet::new();
            loop { if object { if !keys.insert(self.string()?) { return Err(LexiconError("duplicate JSON key")); } self.require(b':')?; }
                self.value(depth+1)?; self.space(); if self.bytes.get(self.pos)==Some(&end) { self.pos+=1; return Ok(()); } self.require(b',')?;
            }
        }
        let start=self.pos;
        while self.bytes.get(self.pos).is_some_and(|b|!matches!(b,b' '|b'\n'|b'\r'|b'\t'|b','|b']'|b'}')) { self.pos+=1; }
        if start==self.pos { return Err(LexiconError("invalid JSON value")); } Ok(())
    }
}
/// Bounded duplicate-key/depth preflight. Follow with typed serde decoding to
/// validate scalar syntax, required fields, and unknown-field policy.
pub fn validate_json_structure(bytes:&[u8])->Result<()> {
    if bytes.len()>MAX_BYTES { return Err(LexiconError("JSON exceeds 8 MiB")); }
    let mut scanner=Scanner{bytes,pos:0}; scanner.value(0)?; scanner.space();
    if scanner.pos!=bytes.len() { return Err(LexiconError("trailing JSON data")); }
    Ok(())
}
pub fn parse(bytes:&[u8])->Result<LexiconDocument> {
    validate_json_structure(bytes)?;
    let document:LexiconDocument=serde_json::from_slice(bytes).map_err(|_|LexiconError("invalid JSON fields"))?;
    document.validate()?; Ok(document)
}
pub fn encode(document:&LexiconDocument)->Result<Vec<u8>> {
    document.validate()?; let bytes=serde_json::to_vec_pretty(document).map_err(|_|LexiconError("cannot encode JSON"))?;
    if bytes.len()>MAX_BYTES { return Err(LexiconError("JSON exceeds 8 MiB")); } Ok(bytes)
}
#[derive(Debug, Clone, Serialize)]
pub struct EntryConflict { pub existing:LexiconEntry, pub incoming:LexiconEntry }
#[derive(Debug, Clone, Serialize)]
pub struct MergePreview {
    /// Proposed additive contents. Conflicting IDs keep the existing entry until explicitly resolved.
    pub entries:Vec<LexiconEntry>, pub added:usize, pub identical:usize,
    pub conflicts:Vec<EntryConflict>,
    /// Distinct identities sharing an ASCII-folded engineering reading; never silently merged.
    pub case_collisions:Vec<(String,String)>,
}
pub fn merge_preview(base:&LexiconDocument,incoming:&LexiconDocument)->Result<MergePreview> {
    base.validate()?; incoming.validate()?;
    let mut result=MergePreview{entries:Vec::new(),added:0,identical:0,conflicts:Vec::new(),case_collisions:Vec::new()};
    let mut ids=HashMap::new();
    for e in &base.entries { if !ids.contains_key(&e.entry_id) { ids.insert(e.entry_id.clone(),result.entries.len()); result.entries.push(e.clone()); } }
    let mut seen=HashSet::new();
    for e in &incoming.entries {
        if !seen.insert(&e.entry_id) { result.identical+=1; continue; }
        if let Some(&i)=ids.get(&e.entry_id) { if result.entries[i]==*e { result.identical+=1; } else { result.conflicts.push(EntryConflict{existing:result.entries[i].clone(),incoming:e.clone()}); } }
        else { ids.insert(e.entry_id.clone(),result.entries.len()); result.entries.push(e.clone()); result.added+=1; }
    }
    if result.entries.len()>MAX_ENTRIES { return Err(LexiconError("merged entries exceed limit")); }
    let mut readings:HashMap<String,&LexiconEntry>=HashMap::new();
    for e in &result.entries { if e.term_type!=TermType::ZhPhrase {
        if let Some(first)=readings.get(&e.reading.to_ascii_lowercase()) { if first.reading!=e.reading { result.case_collisions.push((first.entry_id.clone(),e.entry_id.clone())); } }
        else { readings.insert(e.reading.to_ascii_lowercase(),e); }
    } }
    Ok(result)
}
/// Borrowed candidate proven valid by immutable index construction. The private
/// field prevents bypassing validation with a mutable/deserialized entry.
#[derive(Clone, Copy)]
pub struct SharedCandidate<'a>(&'a LexiconEntry);
impl std::ops::Deref for SharedCandidate<'_> {
    type Target = LexiconEntry;
    fn deref(&self) -> &Self::Target { self.0 }
}
/// Immutable candidate-only index. Caller retains original composition first,
/// requires explicit selection, and grants local approval before constructing this index.
#[derive(Clone, PartialEq, Eq)]
pub struct LexiconIndex { entries:Vec<LexiconEntry>, chinese:HashMap<String,Vec<usize>>, engineering:HashMap<String,Vec<usize>> }
impl LexiconIndex {
    pub fn new(document:&LexiconDocument)->Result<Self> {
        document.validate()?;
        let mut index=Self{entries:Vec::new(),chinese:HashMap::new(),engineering:HashMap::new()}; let mut seen=HashSet::new();
        for e in &document.entries { if e.review.status==ReviewStatus::Rejected || !seen.insert(&e.entry_id) { continue; }
            let i=index.entries.len(); let map=if e.term_type==TermType::ZhPhrase { &mut index.chinese } else { &mut index.engineering };
            let key=if e.term_type==TermType::ZhPhrase { e.reading.clone() } else { e.reading.to_ascii_lowercase() };
            map.entry(key).or_default().push(i); index.entries.push(e.clone());
        } Ok(index)
    }
    pub fn chinese(&self,reading:&str)->Vec<SharedCandidate<'_>> { self.lookup(&self.chinese,reading) }
    pub fn engineering(&self,reading:&str)->Vec<SharedCandidate<'_>> { self.lookup(&self.engineering,&reading.to_ascii_lowercase()) }
    fn lookup(&self,map:&HashMap<String,Vec<usize>>,key:&str)->Vec<SharedCandidate<'_>> { map.get(key).into_iter().flatten().map(|&i|SharedCandidate(&self.entries[i])).collect() }
}

impl std::fmt::Debug for LexiconIndex {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { f.debug_struct("LexiconIndex").field("entry_count",&self.entries.len()).finish_non_exhaustive() }
}

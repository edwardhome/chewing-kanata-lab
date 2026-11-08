//! Explicit, all-or-none export preview. Never writes or mutates the private catalog.
use serde::Serialize;
use crate::shared_lexicon::{entry_id, encode, LexiconDocument, LexiconEntry, LexiconError, Review, ReviewStatus, TermType};
use crate::user_engineering::{encode_user_terms, UserEngineeringTerm, UserTermsError};

#[derive(Debug)]
pub enum EngineeringExportError { UserTerms(UserTermsError), Exchange(LexiconError) }
impl std::fmt::Display for EngineeringExportError {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        match self { Self::UserTerms(_) => f.write_str("invalid or oversized private engineering catalog"), Self::Exchange(e)=> e.fmt(f) }
    }
}
impl std::error::Error for EngineeringExportError {}
#[derive(Clone, Serialize)]
pub struct OmittedAliases { pub index:usize, pub input:String, pub aliases:Vec<String> }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all="snake_case")]
pub enum UnsupportedReason { ChineseAliasesRequireOmissionApproval, AnnotationExceedsUtf16Limit, IncompatibleV1Fields }
#[derive(Clone, Serialize)]
pub struct UnsupportedEntry { pub index:usize, pub input:String, pub reason:UnsupportedReason }
#[derive(Clone, Serialize)]
pub struct EngineeringExportPreview {
    /// None whenever any row is unsupported; callers must never export a partial subset.
    pub document:Option<LexiconDocument>,
    pub omitted_aliases:Vec<OmittedAliases>,
    pub unsupported:Vec<UnsupportedEntry>,
}
impl std::fmt::Debug for EngineeringExportPreview {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        f.debug_struct("EngineeringExportPreview").field("exportable",&self.document.is_some())
            .field("omissions",&self.omitted_aliases.len()).field("unsupported",&self.unsupported.len()).finish()
    }
}
pub fn preview_engineering_export(terms:&[UserEngineeringTerm],version:&str,allow_alias_omission:bool)
    ->Result<EngineeringExportPreview,EngineeringExportError> {
    // Preserve both legacy catalog bounds: 500 rows and 256 KiB serialized bytes.
    encode_user_terms(terms).map_err(EngineeringExportError::UserTerms)?;
    let mut document=LexiconDocument{format:"ai-bopomofo.lexicon".into(),schema_version:"1.0.0".into(),
        document_type:"review".into(),dictionary_version:version.into(),entries:Vec::new()};
    document.validate().map_err(EngineeringExportError::Exchange)?;
    let mut result=EngineeringExportPreview{document:None,omitted_aliases:Vec::new(),unsupported:Vec::new()};
    for (index,term) in terms.iter().enumerate() {
        if !term.zh_aliases.is_empty() {
            if allow_alias_omission { result.omitted_aliases.push(OmittedAliases{index,input:term.input.clone(),aliases:term.zh_aliases.clone()}); }
            else { result.unsupported.push(UnsupportedEntry{index,input:term.input.clone(),reason:UnsupportedReason::ChineseAliasesRequireOmissionApproval}); }
        }
        if term.annotation.encode_utf16().count()>80 {
            result.unsupported.push(UnsupportedEntry{index,input:term.input.clone(),reason:UnsupportedReason::AnnotationExceedsUtf16Limit});
            continue;
        }
        let entry=LexiconEntry{entry_id:entry_id(TermType::TechnicalAlias,&term.input,&term.output),term_type:TermType::TechnicalAlias,
            reading:term.input.clone(),text:term.output.clone(),source:"user_engineering_export".into(),usage:None,
            review:Review{status:ReviewStatus::Proposed,user_confirmed:false},model_suggestion:None,category:None,
            annotation:if term.annotation.is_empty(){None}else{Some(term.annotation.clone())},sources:None,policy:Some("candidate_only".into())};
        if entry.validate().is_err() { result.unsupported.push(UnsupportedEntry{index,input:term.input.clone(),reason:UnsupportedReason::IncompatibleV1Fields}); }
        else { document.entries.push(entry); }
    }
    if result.unsupported.is_empty() {
        encode(&document).map_err(EngineeringExportError::Exchange)?;
        result.document=Some(document);
    }
    Ok(result)
}

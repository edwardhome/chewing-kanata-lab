//! Offline exchange only. An external approval flag never authorizes deployment.
use crate::shared_lexicon::{self, LexiconDocument, LexiconEntry, TermType};
use rusqlite::{params, Connection, OpenFlags, limits::Limit};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, HashMap, HashSet}, fs, io::Write, time::Duration};

pub const MAX_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_RELATIONS: usize = 40_000;
pub const APPLICATION_ID: i32 = 0x4149424c;
pub const SCHEMA: &str = include_str!("../../schemas/unified-lexicon-v1.sql");
const TABLES: [(&str, &str); 3] = [
    ("metadata", "CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)"),
    ("entries", "CREATE TABLE entries (entry_id TEXT PRIMARY KEY, position INTEGER NOT NULL UNIQUE, term_type TEXT NOT NULL, reading TEXT NOT NULL, text TEXT NOT NULL, payload TEXT NOT NULL)"),
    ("relations", "CREATE TABLE relations (source_id TEXT NOT NULL, target_id TEXT NOT NULL, kind TEXT NOT NULL, PRIMARY KEY (source_id, target_id, kind))"),
];
#[derive(Debug)]
pub struct ExchangeError(pub &'static str);
impl std::fmt::Display for ExchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.0) }
}
impl std::error::Error for ExchangeError {}
impl From<rusqlite::Error> for ExchangeError { fn from(_: rusqlite::Error) -> Self { Self("invalid SQLite data or query") } }
impl From<std::io::Error> for ExchangeError { fn from(_: std::io::Error) -> Self { Self("temporary exchange file failed") } }
impl From<serde_json::Error> for ExchangeError { fn from(_: serde_json::Error) -> Self { Self("invalid exchange JSON") } }
impl From<shared_lexicon::LexiconError> for ExchangeError { fn from(_: shared_lexicon::LexiconError) -> Self { Self("invalid lexicon content") } }
type Result<T> = std::result::Result<T, ExchangeError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relation { pub source_id: String, pub target_id: String, pub kind: String }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnifiedLexicon { pub document: LexiconDocument, pub relations: Vec<Relation> }
impl UnifiedLexicon {
    pub fn validate(&self) -> Result<()> {
        shared_lexicon::encode(&self.document)?;
        if self.relations.len() > MAX_RELATIONS { return Err(ExchangeError("too many relations")); }
        let mut ids = HashMap::new();
        for e in &self.document.entries {
            if ids.insert(e.entry_id.as_str(), e.term_type).is_some() { return Err(ExchangeError("duplicate entry")); }
        }
        let mut seen = HashSet::new();
        for r in &self.relations {
            if r.kind != "equivalent" || ids.get(r.source_id.as_str()) != Some(&TermType::ZhPhrase)
                || !ids.get(r.target_id.as_str()).is_some_and(|t| *t != TermType::ZhPhrase)
                || !seen.insert((&r.source_id, &r.target_id, &r.kind)) {
                return Err(ExchangeError("invalid or duplicate relation"));
            }
        }
        Ok(())
    }
}
fn configure(db: &Connection, readonly: bool) -> Result<()> {
    db.busy_timeout(Duration::ZERO)?;
    db.set_limit(Limit::SQLITE_LIMIT_LENGTH, 1_048_576);
    db.set_limit(Limit::SQLITE_LIMIT_SQL_LENGTH, 8192);
    db.set_limit(Limit::SQLITE_LIMIT_COLUMN, 32);
    db.execute_batch("PRAGMA trusted_schema=OFF;")?;
    if readonly { db.execute_batch("PRAGMA query_only=ON;")?; }
    Ok(())
}
fn normalized(sql: &str) -> String { sql.chars().filter(|c| !c.is_whitespace()).flat_map(char::to_lowercase).collect() }
fn text(row: &rusqlite::Row<'_>, column: usize) -> rusqlite::Result<String> {
    let value: String = row.get(column)?;
    if value.contains('\0') { return Err(rusqlite::Error::InvalidQuery); }
    Ok(value)
}
/// Copies bounded bytes to a private temporary directory before opening read-only.
/// No SQL stored in the input database is executed or migrated.
pub fn decode(bytes: &[u8]) -> Result<UnifiedLexicon> {
    if bytes.len() > MAX_BYTES || !bytes.starts_with(b"SQLite format 3\0") { return Err(ExchangeError("unsupported SQLite file")); }
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("lexicon.sqlite");
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path)?;
    file.write_all(bytes)?;
    drop(file);
    let db = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
    configure(&db, true)?;
    let app: i32 = db.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    let version: i32 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if app != APPLICATION_ID || version != 1 { return Err(ExchangeError("unsupported SQLite version")); }
    let mut query = db.prepare("SELECT type,name,sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY name LIMIT 4")?;
    let rows = query.query_map([], |r| Ok((text(r,0)?, text(r,1)?, text(r,2)?)))?.collect::<std::result::Result<Vec<_>,_>>()?;
    if rows.len() != 3 { return Err(ExchangeError("unknown schema objects")); }
    for (kind, name, sql) in rows {
        let expected = TABLES.iter().find(|(n,_)| *n == name).ok_or(ExchangeError("unknown schema object"))?;
        if kind != "table" || normalized(&sql) != normalized(expected.1) { return Err(ExchangeError("incompatible schema")); }
    }
    let check: String = db.query_row("PRAGMA quick_check(1)", [], |r| text(r,0))?;
    if check != "ok" { return Err(ExchangeError("SQLite integrity failure")); }
    let mut query = db.prepare("SELECT key,value FROM metadata LIMIT 5")?;
    let meta: BTreeMap<String,String> = query.query_map([], |r| Ok((text(r,0)?,text(r,1)?)))?.collect::<std::result::Result<_,_>>()?;
    let keys: Vec<_> = meta.keys().map(String::as_str).collect();
    if keys != ["dictionary_version", "document_type", "format", "schema_version"]
        || meta["format"] != "ai-bopomofo.sqlite" || meta["schema_version"] != "1.0.0" { return Err(ExchangeError("invalid metadata")); }
    let mut document = LexiconDocument { format:"ai-bopomofo.lexicon".into(), schema_version:"1.0.0".into(), document_type:meta["document_type"].clone(), dictionary_version:meta["dictionary_version"].clone(), entries:Vec::new() };
    let mut query = db.prepare("SELECT entry_id,position,term_type,reading,text,payload FROM entries ORDER BY position LIMIT 20001")?;
    let mut rows = query.query([])?;
    let mut payload_bytes: usize = 0;
    while let Some(row) = rows.next()? {
        let position: i64 = row.get(1)?;
        if document.entries.len() >= shared_lexicon::MAX_ENTRIES || position != document.entries.len() as i64 { return Err(ExchangeError("invalid entry position or count")); }
        let payload = text(row,5)?;
        payload_bytes += payload.len();
        if payload_bytes > shared_lexicon::MAX_BYTES { return Err(ExchangeError("JSON payload limit")); }
        shared_lexicon::validate_json_structure(payload.as_bytes())?;
        let entry: LexiconEntry = serde_json::from_str(&payload)?;
        entry.validate()?;
        if text(row,0)? != entry.entry_id || text(row,2)? != entry.term_type.as_str() || text(row,3)? != entry.reading || text(row,4)? != entry.text { return Err(ExchangeError("payload mismatch")); }
        document.entries.push(entry);
    }
    let mut query = db.prepare("SELECT source_id,target_id,kind FROM relations ORDER BY source_id,target_id,kind LIMIT 40001")?;
    let relations = query.query_map([], |r| Ok(Relation {source_id:text(r,0)?, target_id:text(r,1)?, kind:text(r,2)?}))?.collect::<std::result::Result<Vec<_>,_>>()?;
    let value = UnifiedLexicon { document, relations };
    value.validate()?;
    Ok(value)
}
/// Produces an independent SQLite snapshot, never an active database plus WAL.
pub fn encode(value: &UnifiedLexicon) -> Result<Vec<u8>> {
    value.validate()?;
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("lexicon.sqlite");
    {
        let mut db = Connection::open(&path)?;
        configure(&db, false)?;
        let tx = db.transaction()?;
        tx.execute_batch(SCHEMA)?;
        for (key, val) in [("format", "ai-bopomofo.sqlite"), ("schema_version", "1.0.0"), ("document_type", value.document.document_type.as_str()), ("dictionary_version", value.document.dictionary_version.as_str())] {
            tx.execute("INSERT INTO metadata VALUES (?,?)", params![key,val])?;
        }
        for (position, entry) in value.document.entries.iter().enumerate() {
            tx.execute("INSERT INTO entries VALUES (?,?,?,?,?,?)", params![entry.entry_id, position as i64, entry.term_type.as_str(), entry.reading, entry.text, serde_json::to_string(entry)?])?;
        }
        for r in &value.relations { tx.execute("INSERT INTO relations VALUES (?,?,?)", params![r.source_id,r.target_id,r.kind])?; }
        tx.commit()?;
    }
    let bytes = fs::read(path)?;
    if bytes.len() > MAX_BYTES { return Err(ExchangeError("SQLite file limit")); }
    Ok(bytes)
}

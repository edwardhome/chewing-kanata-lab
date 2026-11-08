// SPDX-License-Identifier: MIT
//! Local, explicitly approved shared-lexicon snapshots. No user-path discovery or network I/O.
//!
//! `prepare` is read-only. The application keeps its non-serializable result in memory
//! until the user confirms. `approve` is the only deployment entry point. The active
//! pointer includes the previous reference so one atomic replacement commits both;
//! `previous-active.json` is a diagnostic mirror, never rollback authority.
use crate::shared_lexicon::{self, LexiconDocument};
use fs2::FileExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const META_LIMIT: usize = 16 * 1024;
const MAX_VERSIONS: usize = 1024;
const FORMAT_VERSION: u32 = 1;
type Result<T> = std::result::Result<T, StoreError>;

/// Static messages deliberately omit file paths and private entry contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    InvalidDocument,
    InvalidStore,
    RevisionChanged,
    VersionConflict,
    NoPreviousVersion,
    NoActiveVersion,
    WrongStore,
    StorageFailure,
    StoreBusy,
    Capacity,
}
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidDocument => "共用詞庫格式或內容不合法。",
            Self::InvalidStore => "共用詞庫版本、指標或本機核准紀錄驗證失敗。",
            Self::RevisionChanged => "共用詞庫已由其他操作變更，請重新預覽。",
            Self::VersionConflict => "相同詞庫版本名稱已有不同內容，請使用新版本名稱。",
            Self::NoPreviousVersion => "沒有可回復的上一個共用詞庫版本。",
            Self::NoActiveVersion => "尚未核准部署共用詞庫。",
            Self::WrongStore => "預覽不屬於這個本機詞庫位置，請重新預覽。",
            Self::StorageFailure => "共用詞庫寫入失敗；目前版本未切換。",
            Self::StoreBusy => "另一個詞庫管理操作正在進行，請稍後再試。",
            Self::Capacity => "本機共用詞庫版本數量已達上限。",
        })
    }
}
impl std::error::Error for StoreError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionInfo {
    pub checksum: String,
    pub dictionary_version: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct StoreStatus {
    pub revision: Option<String>,
    pub enabled: bool,
    pub current: Option<VersionInfo>,
    pub previous: Option<VersionInfo>,
}
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub revision: String,
    pub checksum: String,
    pub document: LexiconDocument,
    pub enabled: bool,
}

/// Cannot be constructed or deserialized outside this module. Dropping cancels it.
pub struct PreparedImport {
    root: PathBuf,
    revision: Option<String>,
    source_checksum: String,
    bytes: Vec<u8>,
    document: LexiconDocument,
    checksum: String,
}
impl std::fmt::Debug for PreparedImport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedImport").field("entry_count", &self.document.entries.len()).finish_non_exhaustive()
    }
}
impl PreparedImport {
    pub fn document(&self) -> &LexiconDocument { &self.document }
    pub fn checksum(&self) -> &str { &self.checksum }
    pub fn base_revision(&self) -> Option<&str> { self.revision.as_deref() }
}
#[derive(Debug, Clone)]
pub struct SharedStore { root: PathBuf }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pointer {
    format_version: u32,
    generation: u64,
    enabled: bool,
    active: VersionInfo,
    previous: Option<VersionInfo>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity { format_version: u32, store_id: String }
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    format_version: u32,
    store_id: String,
    approval_id: String,
    approved_at_unix_ms: u64,
    source_checksum: String,
    checksum: String,
    dictionary_version: String,
}
struct LoadedPointer { pointer: Pointer, revision: String }
struct VerifiedVersion { document: LexiconDocument }

impl SharedStore {
    /// Root is supplied by the platform data-directory resolver, never by a document.
    pub fn new(root: PathBuf) -> Self { Self { root } }

    pub fn status(&self) -> Result<StoreStatus> {
        let loaded = self.read_pointer()?;
        if let Some(state) = loaded {
            let identity = self.read_identity()?;
            self.read_version(&state.pointer.active, &identity)?;
            Ok(StoreStatus {
                revision: Some(state.revision), enabled: state.pointer.enabled,
                current: Some(state.pointer.active), previous: state.pointer.previous,
            })
        } else {
            Ok(StoreStatus { revision: None, enabled: false, current: None, previous: None })
        }
    }

    /// Read-only and lock-free: versions are immutable and pointer replacement is atomic.
    /// The caller disables the shared extension on an error, retaining native input.
    pub fn load(&self) -> Result<Option<Snapshot>> {
        let Some(state) = self.read_pointer()? else { return Ok(None); };
        let identity = self.read_identity()?;
        let version = self.read_version(&state.pointer.active, &identity)?;
        Ok(Some(Snapshot { revision: state.revision, checksum: state.pointer.active.checksum,
            document: version.document, enabled: state.pointer.enabled }))
    }

    /// Parsing and preview cannot write receipts, directories, locks or active pointers.
    pub fn prepare(&self, bytes: &[u8], expected_revision: Option<&str>) -> Result<PreparedImport> {
        let document = shared_lexicon::parse(bytes).map_err(|_| StoreError::InvalidDocument)?;
        let source_bytes = shared_lexicon::encode(&document).map_err(|_| StoreError::InvalidDocument)?;
        let source_checksum = digest(&source_bytes);
        let state = self.read_pointer()?;
        check_revision(state.as_ref(), expected_revision)?;
        if let Some(state) = &state {
            self.read_version(&state.pointer.active, &self.read_identity()?)?;
        }
        // Source approval and usage remain exchange data. Only the separately
        // generated local receipt grants deployment; usage never updates an engine.
        let bytes = shared_lexicon::encode(&document).map_err(|_| StoreError::InvalidDocument)?;
        let prepared = PreparedImport { root: self.root.clone(), revision: expected_revision.map(str::to_owned),
            source_checksum, checksum: digest(&bytes), bytes, document };
        self.check_version_name(&prepared)?;
        Ok(prepared)
    }

    /// Application calls only after the user explicitly confirms this in-memory preview.
    pub fn approve(&self, prepared: PreparedImport) -> Result<Snapshot> {
        self.approve_impl(prepared, |_| Ok(()))
    }

    fn approve_impl(&self, prepared: PreparedImport, checkpoint: impl Fn(Checkpoint) -> Result<()>) -> Result<Snapshot> {
        if prepared.root != self.root { return Err(StoreError::WrongStore); }
        let _lock = self.writer_lock()?;
        let state = self.read_pointer()?;
        check_revision(state.as_ref(), prepared.revision.as_deref())?;
        let identity = self.ensure_identity()?;
        if let Some(state) = &state { self.read_version(&state.pointer.active, &identity)?; }
        self.check_version_name(&prepared)?;
        if let Some(state) = &state {
            if state.pointer.active.checksum == prepared.checksum {
                let version = self.read_version(&state.pointer.active, &identity)?;
                // Reimport does not silently enable a previously disabled extension.
                return Ok(Snapshot { revision: state.revision.clone(), checksum: prepared.checksum,
                    document: version.document, enabled: state.pointer.enabled });
            }
        }
        let versions = self.root.join("versions");
        let staging = self.root.join("staging");
        ensure_directory(&versions)?;
        ensure_directory(&staging)?;
        let stage = tempfile::Builder::new().prefix("operation-").tempdir_in(&staging)
            .map_err(|_| StoreError::StorageFailure)?;
        let operation_id = digest(stage.path().file_name().ok_or(StoreError::InvalidStore)?.to_string_lossy().as_bytes());
        let receipt = Receipt { format_version: FORMAT_VERSION, store_id: identity.store_id.clone(),
            approval_id: operation_id, approved_at_unix_ms: now_ms()?,
            source_checksum: prepared.source_checksum.clone(), checksum: prepared.checksum.clone(),
            dictionary_version: prepared.document.dictionary_version.clone() };
        write_new(&stage.path().join("lexicon.json"), &prepared.bytes)?;
        write_new(&stage.path().join("receipt.json"), &json(&receipt)?)?;
        checkpoint(Checkpoint::Staged)?;
        self.verify_version_directory(stage.path(), &VersionInfo {
            checksum: prepared.checksum.clone(), dictionary_version: prepared.document.dictionary_version.clone(),
        }, &identity)?;
        checkpoint(Checkpoint::Verified)?;
        let destination = versions.join(&prepared.checksum);
        if path_exists(&destination)? {
            self.read_version(&VersionInfo { checksum: prepared.checksum.clone(),
                dictionary_version: prepared.document.dictionary_version.clone() }, &identity)?;
        } else {
            fs::rename(stage.path(), &destination).map_err(|_| StoreError::StorageFailure)?;
        }
        let next = Pointer { format_version: FORMAT_VERSION, generation: next_generation(state.as_ref())?,
            enabled: state.as_ref().map_or(true, |s| s.pointer.enabled),
            active: VersionInfo { checksum: prepared.checksum.clone(), dictionary_version: prepared.document.dictionary_version.clone() },
            previous: state.as_ref().map(|s| s.pointer.active.clone()) };
        checkpoint(Checkpoint::Activating)?;
        let revision = self.commit_pointer_impl(&next, || checkpoint(Checkpoint::PointerSwap))?;
        // After the single commit point there are no fallible operations.
        Ok(Snapshot { revision, checksum: prepared.checksum, document: prepared.document, enabled: next.enabled })
    }

    pub fn rollback(&self, expected_revision: Option<&str>) -> Result<Snapshot> {
        let _lock = self.writer_lock()?;
        let state = self.read_pointer()?;
        check_revision(state.as_ref(), expected_revision)?;
        let state = state.ok_or(StoreError::NoActiveVersion)?;
        let previous = state.pointer.previous.clone().ok_or(StoreError::NoPreviousVersion)?;
        let version = self.read_version(&previous, &self.read_identity()?)?;
        let next = Pointer { format_version: FORMAT_VERSION,
            generation: state.pointer.generation.checked_add(1).ok_or(StoreError::InvalidStore)?,
            enabled: state.pointer.enabled, active: previous.clone(), previous: Some(state.pointer.active) };
        let revision = self.commit_pointer(&next)?;
        Ok(Snapshot { revision, checksum: previous.checksum, document: version.document, enabled: next.enabled })
    }

    pub fn set_enabled(&self, enabled: bool, expected_revision: Option<&str>) -> Result<Option<Snapshot>> {
        let _lock = self.writer_lock()?;
        let state = self.read_pointer()?;
        check_revision(state.as_ref(), expected_revision)?;
        let Some(state) = state else {
            return if enabled { Err(StoreError::NoActiveVersion) } else { Ok(None) };
        };
        let version = self.read_version(&state.pointer.active, &self.read_identity()?)?;
        if state.pointer.enabled == enabled {
            return Ok(Some(Snapshot { revision: state.revision, checksum: state.pointer.active.checksum,
                document: version.document, enabled }));
        }
        let mut next = state.pointer;
        next.enabled = enabled;
        next.generation = next.generation.checked_add(1).ok_or(StoreError::InvalidStore)?;
        let revision = self.commit_pointer(&next)?;
        Ok(Some(Snapshot { revision, checksum: next.active.checksum, document: version.document, enabled }))
    }

    fn writer_lock(&self) -> Result<File> {
        ensure_directory(&self.root)?;
        let path = self.root.join("store.lock");
        if path_exists(&path)? { ensure_regular_file(&path)?; }
        let mut options = OpenOptions::new();
        options.create(true).truncate(false).read(true).write(true);
        restrict_new_file(&mut options);
        let file = options.open(&path).map_err(|_| StoreError::StorageFailure)?;
        FileExt::try_lock_exclusive(&file).map_err(|_| StoreError::StoreBusy)?;
        Ok(file)
    }
    fn read_pointer(&self) -> Result<Option<LoadedPointer>> {
        if !path_exists(&self.root)? { return Ok(None); }
        check_directory(&self.root)?;
        let path = self.root.join("active.json");
        if !path_exists(&path)? { return Ok(None); }
        let bytes = bounded_read(&path, META_LIMIT)?;
        let pointer: Pointer = metadata(&bytes)?;
        if pointer.format_version != FORMAT_VERSION || pointer.generation == 0 { return Err(StoreError::InvalidStore); }
        validate_reference(&pointer.active)?;
        if let Some(previous) = &pointer.previous { validate_reference(previous)?; }
        Ok(Some(LoadedPointer { pointer, revision: digest(&bytes) }))
    }
    fn read_identity(&self) -> Result<Identity> {
        check_directory(&self.root)?;
        let identity: Identity = metadata(&bounded_read(&self.root.join("store-id.json"), META_LIMIT)?)?;
        if identity.format_version != FORMAT_VERSION || !valid_hash(&identity.store_id) { return Err(StoreError::InvalidStore); }
        Ok(identity)
    }
    fn ensure_identity(&self) -> Result<Identity> {
        let path = self.root.join("store-id.json");
        if path_exists(&path)? { return self.read_identity(); }
        let seed = tempfile::Builder::new().prefix("identity-").tempfile_in(&self.root)
            .map_err(|_| StoreError::StorageFailure)?;
        let identity = Identity { format_version: FORMAT_VERSION,
            store_id: digest(format!("{}:{}:{}", seed.path().display(), now_ms()?, std::process::id()).as_bytes()) };
        write_new(&path, &json(&identity)?)?;
        Ok(identity)
    }
    fn read_version(&self, reference: &VersionInfo, identity: &Identity) -> Result<VerifiedVersion> {
        validate_reference(reference)?;
        let versions = self.root.join("versions");
        check_directory(&versions)?;
        self.verify_version_directory(&versions.join(&reference.checksum), reference, identity)
    }
    fn verify_version_directory(&self, directory: &Path, reference: &VersionInfo, identity: &Identity) -> Result<VerifiedVersion> {
        check_directory(directory)?;
        let receipt: Receipt = metadata(&bounded_read(&directory.join("receipt.json"), META_LIMIT)?)?;
        if receipt.format_version != FORMAT_VERSION || receipt.store_id != identity.store_id
            || receipt.checksum != reference.checksum || receipt.dictionary_version != reference.dictionary_version
            || receipt.source_checksum != receipt.checksum || !valid_hash(&receipt.approval_id) || receipt.approved_at_unix_ms == 0 {
            return Err(StoreError::InvalidStore);
        }
        let bytes = bounded_read(&directory.join("lexicon.json"), shared_lexicon::MAX_BYTES)?;
        if digest(&bytes) != reference.checksum { return Err(StoreError::InvalidStore); }
        let document = shared_lexicon::parse(&bytes).map_err(|_| StoreError::InvalidStore)?;
        if document.dictionary_version != reference.dictionary_version {
            return Err(StoreError::InvalidStore);
        }
        Ok(VerifiedVersion { document })
    }
    fn check_version_name(&self, prepared: &PreparedImport) -> Result<()> {
        let versions = self.root.join("versions");
        if !path_exists(&versions)? { return Ok(()); }
        check_directory(&self.root)?;
        check_directory(&versions)?;
        let identity = self.read_identity()?;
        let mut count = 0;
        for entry in fs::read_dir(&versions).map_err(|_| StoreError::InvalidStore)? {
            let entry = entry.map_err(|_| StoreError::InvalidStore)?;
            count += 1;
            if count > MAX_VERSIONS { return Err(StoreError::Capacity); }
            let checksum = entry.file_name().into_string().map_err(|_| StoreError::InvalidStore)?;
            if !valid_hash(&checksum) { return Err(StoreError::InvalidStore); }
            check_directory(&entry.path())?;
            let receipt: Receipt = metadata(&bounded_read(&entry.path().join("receipt.json"), META_LIMIT)?)?;
            if receipt.store_id != identity.store_id || receipt.checksum != checksum
                || receipt.source_checksum != checksum || receipt.format_version != FORMAT_VERSION {
                return Err(StoreError::InvalidStore);
            }
            if receipt.dictionary_version == prepared.document.dictionary_version {
                if checksum != prepared.checksum {
                    return Err(StoreError::VersionConflict);
                }
                self.read_version(&VersionInfo { checksum, dictionary_version: receipt.dictionary_version }, &identity)?;
            }
        }
        if count == MAX_VERSIONS && !path_exists(&versions.join(&prepared.checksum))? { return Err(StoreError::Capacity); }
        Ok(())
    }
    fn commit_pointer(&self, next: &Pointer) -> Result<String> {
        self.commit_pointer_impl(next, || Ok(()))
    }
    fn commit_pointer_impl(&self, next: &Pointer, before_swap: impl FnOnce() -> Result<()>) -> Result<String> {
        let bytes = json(next)?;
        let mirror = json(&next.previous)?;
        // If the final replace fails this mirror may lead the active pointer;
        // readers/rollback exclusively trust active.json's embedded previous.
        atomic_replace(&self.root, "previous-active.json", &mirror)?;
        before_swap()?;
        atomic_replace(&self.root, "active.json", &bytes)?;
        Ok(digest(&bytes))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Checkpoint { Staged, Verified, Activating, PointerSwap }
fn digest(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn metadata<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    shared_lexicon::validate_json_structure(bytes).map_err(|_| StoreError::InvalidStore)?;
    serde_json::from_slice(bytes).map_err(|_| StoreError::InvalidStore)
}
fn json(value: &impl Serialize) -> Result<Vec<u8>> { serde_json::to_vec_pretty(value).map_err(|_| StoreError::InvalidStore) }
fn now_ms() -> Result<u64> {
    u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| StoreError::StorageFailure)?.as_millis())
        .map_err(|_| StoreError::StorageFailure)
}
fn valid_hash(value: &str) -> bool { value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) }
fn validate_reference(reference: &VersionInfo) -> Result<()> {
    // Version labels are never interpolated into a path.
    if !valid_hash(&reference.checksum) || reference.dictionary_version.is_empty()
        || reference.dictionary_version.encode_utf16().count() > 80 { return Err(StoreError::InvalidStore); }
    Ok(())
}
fn check_revision(state: Option<&LoadedPointer>, expected: Option<&str>) -> Result<()> {
    if state.map(|s| s.revision.as_str()) != expected { return Err(StoreError::RevisionChanged); }
    Ok(())
}
fn next_generation(state: Option<&LoadedPointer>) -> Result<u64> {
    state.map_or(Some(1), |s| s.pointer.generation.checked_add(1)).ok_or(StoreError::InvalidStore)
}
fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))] { metadata.file_type().is_symlink() }
}
fn path_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(StoreError::InvalidStore),
    }
}
fn check_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| StoreError::InvalidStore)?;
    if is_link(&metadata) || !metadata.is_dir() { return Err(StoreError::InvalidStore); }
    Ok(())
}
fn ensure_directory(path: &Path) -> Result<()> {
    if !path_exists(path)? {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)] {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(path).map_err(|_| StoreError::StorageFailure)?;
    }
    check_directory(path)
}
fn ensure_regular_file(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| StoreError::InvalidStore)?;
    if is_link(&metadata) || !metadata.is_file() { return Err(StoreError::InvalidStore); }
    Ok(())
}
fn bounded_read(path: &Path, max: usize) -> Result<Vec<u8>> {
    ensure_regular_file(path)?;
    let file = File::open(path).map_err(|_| StoreError::InvalidStore)?;
    if file.metadata().map_err(|_| StoreError::InvalidStore)?.len() > max as u64 { return Err(StoreError::InvalidStore); }
    let mut bytes = Vec::new();
    file.take(max as u64 + 1).read_to_end(&mut bytes).map_err(|_| StoreError::InvalidStore)?;
    if bytes.len() > max { return Err(StoreError::InvalidStore); }
    Ok(bytes)
}
fn restrict_new_file(options: &mut OpenOptions) {
    #[cfg(unix)] {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(not(unix))] { let _ = options; }
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    restrict_new_file(&mut options);
    let mut file = options.open(path).map_err(|_| StoreError::StorageFailure)?;
    file.write_all(bytes).and_then(|_| file.sync_all()).map_err(|_| StoreError::StorageFailure)
}
fn atomic_replace(root: &Path, name: &str, bytes: &[u8]) -> Result<()> {
    let destination = root.join(name);
    if path_exists(&destination)? { ensure_regular_file(&destination)?; }
    let mut temp = tempfile::Builder::new().prefix("pointer-").tempfile_in(root).map_err(|_| StoreError::StorageFailure)?;
    temp.write_all(bytes).and_then(|_| temp.as_file().sync_all()).map_err(|_| StoreError::StorageFailure)?;
    temp.persist(&destination).map_err(|_| StoreError::StorageFailure)?;
    Ok(())
}

#[cfg(test)]
mod tests;

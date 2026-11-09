#[cfg(feature="unified-sqlite")]
fn main() -> Result<(),Box<dyn std::error::Error>> {
    use std::{fs,io::{Read,Write}};
    use chewing_policy_core::unified_sqlite::{decode,encode,MAX_BYTES};
    let a:Vec<_>=std::env::args().collect();
    if a.len()!=4 {return Err("usage: unified_sqlite_roundtrip INPUT OUTPUT_SQLITE OUTPUT_JSON".into());}
    let mut data=Vec::new();fs::File::open(&a[1])?.take((MAX_BYTES+1) as u64).read_to_end(&mut data)?;
    let value=decode(&data)?;let encoded=encode(&value)?;
    let mut sqlite=fs::OpenOptions::new().create_new(true).write(true).open(&a[2])?;
    sqlite.write_all(&encoded)?;
    let mut json=fs::OpenOptions::new().create_new(true).write(true).open(&a[3])?;
    json.write_all(&serde_json::to_vec_pretty(&value)?)?;
    println!("{} entries, {} relations validated and exported",value.document.entries.len(),value.relations.len());Ok(())
}
#[cfg(not(feature="unified-sqlite"))]
fn main(){eprintln!("enable unified-sqlite feature");std::process::exit(2);}

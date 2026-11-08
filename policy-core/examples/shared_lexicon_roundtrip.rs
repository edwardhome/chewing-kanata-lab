//! Headless exchange probe; caller explicitly supplies input and a new output path.
use std::{fs,io::Write};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let args:Vec<String>=std::env::args().collect();
 if args.len()!=3 {return Err("usage: shared_lexicon_roundtrip INPUT OUTPUT".into());}
 let bytes=fs::read(&args[1])?;
 let document=chewing_policy_core::shared_lexicon::parse(&bytes)?;
 let bytes=chewing_policy_core::shared_lexicon::encode(&document)?;
 let mut out=fs::OpenOptions::new().write(true).create_new(true).open(&args[2])?;
 out.write_all(&bytes)?;
 println!("validated and exported {} entries",document.entries.len());Ok(())
}

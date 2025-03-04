// etc/programs.rs

use crate::wadparse::parse;
use std::env::Args;

use crate::etc::options::DebugLumpsOptions;

/// Info entrypoint program
/// subcommand: info
pub fn info_entrypoint(_args: &mut Args) -> Result<u8, String> {
    Err(format!("Not implemented"))
}

/// DebugLumps entrypoint program
/// subcommand: lumps
pub fn debuglumps_entrypoint(args: &mut Args) -> Result<u8, String> {
    let opts = match DebugLumpsOptions::new(args) {
        Ok(o) => o,
        Err(e) => {
            return Err(format!("lumps: {}", e));
        }
    };

    for fname in &opts.files {
        let wad = match parse(fname) {
            Ok(w) => w,
            Err(_) => {
                return Err(format!("???"));
            }
        };

        for lump in &wad.lumps {
            println!("{:?}", lump);
        }
    }

    Ok(0)
}

// end etc/programs.rs

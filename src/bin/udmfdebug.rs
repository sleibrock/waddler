// udmfdebug.rs - debugging tool for the UDMF format
// currently dumps out the entirety of TEXTMAP

use std::env::args;
use std::process::exit;

extern crate waddler;
use waddler::wadparse::parse_into_both;

fn main() {
    let mut arg_iter = args();
    arg_iter.next();

    for fname in arg_iter {
        let (wad, data) = match parse_into_both(&fname) {
            Ok((w, d)) => (w, d),
            Err(e) => panic!("Welp. {}", e),
        };

        for lump in &wad.lumps {
            println!("{:?}", lump);
            if lump.name == "TEXTMAP".to_string() {
                println!("Found textmap");
                let txtmap = &data[lump.start..lump.end];
                let mut i = 0;
                while i < txtmap.len() {
                    print!("{}", (txtmap[i]) as char);
                    i += 1;
                }
            }
        }
    }

    exit(0);
}

// end udmfdebug.rs

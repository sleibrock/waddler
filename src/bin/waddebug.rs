// waddebug.rs - a WAD debugger to inspect headers

extern crate waddler;

use std::env::args;
use std::process::exit;

use waddler::etc::programs::debuglumps_entrypoint;

fn main() {
    let mut arg_iter = args();
    arg_iter.next();

    let _res = match debuglumps_entrypoint(&mut arg_iter) {
        Ok(x) => x,
        Err(e) => panic!("Huh? {}", e),
    };

    exit(0);
}

// end waddebug.rs

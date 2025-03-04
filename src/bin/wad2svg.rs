// wad2svg.rs - convert WAD maps to SVG files

extern crate waddler;

use std::env::args;
use std::process::exit;

use waddler::svgmap::programs::svgmap_entrypoint;

fn main() {
    let mut arg_iter = args();
    arg_iter.next();

    let _res = match svgmap_entrypoint(&mut arg_iter) {
        Ok(x) => x,
        Err(e) => panic!("Huh? {}", e),
    };

    exit(0);
}

// end wad2svg.rs

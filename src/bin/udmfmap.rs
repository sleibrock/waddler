// udmfmap.rs - a tool to dump maps from UDMF format

/*
UDMF is a newer format for Doom wads supported in higher end
source ports like GZDoom. It is a collaborative effort to improve
upon the original DOOM engine by expanding capabilities and data
types to support larger maps with greater levels of complexity and detail.
Notably, the new format is more plaintext format than binary packed,
so data types such as Vertices are now floating-point instead of integer/i16

The goal is to read the contents of TEXTMAP and convert it to an SVG map
as done in other places.

These are the two core structures we need to define an SVG map

vertex // <integer>
{
x = <floating point>;
y = <floating point>;
}

linedef // <number>
{
v1 = <usize>;
v2 = <usize>;
}

sector // <number>
{
/// don't care about sectors for now
}

thing // <number>
{
/// don't care about things for now
}
*/

use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::Write;
use std::process::exit;

extern crate waddler;
use waddler::wadparse::parse_into_both;

// we need two new structs to define the vertices and linedefs
#[derive(Debug)]
struct UVertex {
    x: f64,
    y: f64,
}
impl UVertex {
    fn new() -> UVertex {
        UVertex { x: 0.0, y: 0.0 }
    }
}

#[derive(Debug)]
struct ULinedef {
    v1: usize,
    v2: usize,
}
impl ULinedef {
    fn new() -> ULinedef {
        ULinedef { v1: 0, v2: 0 }
    }
}

// This is the parsing state, holds a mutable value
// to dictate what we are working on
// or holds nothing at all to indicate no struct is being used
enum PState {
    Vert(UVertex),
    Line(ULinedef),
    None,
}

fn slice_to_f64(sl: &str) -> f64 {
    let x: f64 = sl.parse().unwrap();
    return x;
}

fn slice_to_usize(sl: &str) -> usize {
    let x: usize = sl.parse().unwrap();
    return x;
}

fn main() {
    let mut arg_iter = args();
    arg_iter.next();

    for fname in arg_iter {
        println!("Working on {}...", fname);
        let (wad, data) = match parse_into_both(&fname) {
            Ok((w, d)) => (w, d),
            Err(e) => panic!("Welp. {}", e),
        };

        for lump in &wad.lumps {
            if lump.name == "TEXTMAP".to_string() {
                println!("Found textmap");
                let txtmap = &data[lump.start..lump.end];

                // parse the textmap into a list of lines
                let mut start = 0;
                let mut end;
                let max = lump.size - 1;
                let mut pstate = PState::None;
                let mut pid = 0;
                let mut in_struct = true;

                // containers for data
                // for now, linedefs -> vector, vertices -> hashmap
                let mut linedefs: Vec<ULinedef> = Vec::new();
                let mut vertices: HashMap<usize, UVertex> = HashMap::new();

                // svg specific boundaries needed
                let mut max_x = 0.;
                let mut max_y = 0.;
                let mut min_x = 0.;
                let mut min_y = 0.;

                while start < max {
                    end = start;
                    while end != max - 1 && txtmap[end] != 10 {
                        end += 1;
                    }
                    let line: String = txtmap[start..end].iter().map(|x| *x as char).collect();

                    // main parsing state checking
                    // for each type of thing we need to grab it's identifier as well
                    if line.starts_with("vertex // ") {
                        pstate = PState::Vert(UVertex::new());
                        pid = slice_to_usize(&line[10..line.len()]);
                    } else if line.starts_with("linedef") {
                        pstate = PState::Line(ULinedef::new());
                        pid = 0;
                    } else if line.starts_with("{") {
                        in_struct = true;
                    } else if line.starts_with("}") {
                        in_struct = false;
                        // pop the data from the PState(?)
                        match pstate {
                            PState::Vert(v) => {
                                vertices.insert(pid, v);
                            }
                            PState::Line(l) => {
                                linedefs.push(l);
                            }
                            _ => {}
                        }
                        pstate = PState::None;
                    }

                    if in_struct {
                        let lend = line.len() - 1;
                        match pstate {
                            PState::Vert(ref mut v) => {
                                if line.starts_with("x = ") {
                                    let decimal = slice_to_f64(&line[4..lend]);
                                    if decimal > max_x {
                                        max_x = decimal;
                                    } else if decimal < min_x {
                                        min_x = decimal;
                                    }
                                    v.x = decimal;
                                } else if line.starts_with("y = ") {
                                    let decimal = slice_to_f64(&line[4..lend]);
                                    if decimal > max_y {
                                        max_y = decimal;
                                    } else if decimal < min_y {
                                        min_y = decimal;
                                    }
                                    v.y = decimal;
                                }
                            }
                            PState::Line(ref mut l) => {
                                if line.starts_with("v1 = ") {
                                    let decimal = slice_to_usize(&line[5..lend]);
                                    l.v1 = decimal;
                                } else if line.starts_with("v2 = ") {
                                    let decimal = slice_to_usize(&line[5..lend]);
                                    l.v2 = decimal;
                                }
                            }
                            _ => {}
                        }
                    }

                    // bump the start index to the end + 1
                    start = end + 1;
                }
                // end of the big while loop

                // start rendering to an SVG file HERE
                // none of my old SVG code works for floating point types currently
                let svg_width = (max_x as u64) + 1;
                let svg_height = (max_y as u64) + 1;

                let mut f = match File::create(format!("{}.svg", fname)) {
                    Ok(new_file) => new_file,
                    Err(why) => panic!("Couldn't create file. {}", why),
                };

                let svg_header = format!("<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\">", svg_width, svg_height, svg_width, svg_height);
                let _ = f.write(svg_header.as_ref());
                for linedef in linedefs {
                    let a = vertices.get(&linedef.v1).unwrap();
                    let b = vertices.get(&linedef.v2).unwrap();

                    let _ = f.write(format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\" />",
                        a.x,
                        a.y,
                        b.x,
                        b.y,
                        "black",
                        "5").as_ref());
                }
                let _ = f.write(b"</svg>");
                // end SVG rendering

                println!("Created map: {}.svg", fname);
            }
        }
    }

    exit(0);
}

// end udmfmap.rs

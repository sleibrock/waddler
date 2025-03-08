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
sidefront = <usize>;
sideback = <usize>;
twosided = <bool>;
dontdraw = <bool>;
dontpegtop = <bool>;
}

sidedef // <number>
{
sector = <usize>;
offsetx_top = <float>;
offsetx_bottom = <float>;
texturebottom = <string>;
texturetop = <string>;
texturemiddle = <string>;
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
    special: usize,
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    twosided: bool,
}
impl ULinedef {
    fn new() -> ULinedef {
        ULinedef {
            v1: 0,
            v2: 0,
            special: 0,
            arg0: 0,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            arg4: 0,
            twosided: false,
        }
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

fn slice_to_i32(sl: &str) -> i32 {
    let x: i32 = sl.parse().unwrap();
    return x;
}

fn slice_to_bool(sl: &str) -> bool {
    let x: bool = sl.parse().unwrap();
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

        let core_data = &data[wad.header.data_range()];

        for lump in &wad.lumps {
            if lump.name == "TEXTMAP".to_string() {
                println!("Found textmap");
                println!("Textmap length: {}", lump.size);
                let txtmap = &core_data[lump.start..lump.end];

                // parse the textmap into a list of lines
                let mut start = 0;
                let mut end;
                let max = lump.size - 1;
                let mut pstate = PState::None;
                let mut pid = 0;
                let mut in_struct = true;

                // containers for data
                // for now, linedefs -> vector, vertices -> hashmap
                let mut linedefs: HashMap<usize, ULinedef> = HashMap::new();
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
                    } else if line.starts_with("linedef // ") {
                        pstate = PState::Line(ULinedef::new());
                        pid = slice_to_usize(&line[11..line.len()]);
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
                                linedefs.insert(pid, l);
                            }
                            _ => {}
                        }
                        pstate = PState::None;
                    }

                    if in_struct {
                        let lend = line.len() - 1; // no \n character in slices
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
                                } else if line.starts_with("special = ") {
                                    let decimal = slice_to_usize(&line[10..lend]);
                                    l.special = decimal;
                                } else if line.starts_with("arg0 = ") {
                                    let decimal = slice_to_i32(&line[7..lend]);
                                    l.arg0 = decimal;
                                } else if line.starts_with("arg1 = ") {
                                    let decimal = slice_to_i32(&line[7..lend]);
                                    l.arg1 = decimal;
                                } else if line.starts_with("arg2 = ") {
                                    let decimal = slice_to_i32(&line[7..lend]);
                                    l.arg2 = decimal;
                                } else if line.starts_with("arg3 = ") {
                                    let decimal = slice_to_i32(&line[7..lend]);
                                    l.arg3 = decimal;
                                } else if line.starts_with("arg4 = ") {
                                    let decimal = slice_to_i32(&line[7..lend]);
                                    l.arg4 = decimal;
                                } else if line.starts_with("twosided = ") {
                                    let boolean = slice_to_bool(&line[11..lend]);
                                    l.twosided = boolean;
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
                // some vertices may be negative, so we want to shift them
                // into the positive domain for ease
                let svg_max_x = (max_x as i64) + 1;
                let svg_max_y = (max_y as i64) + 1;
                let svg_min_x = (min_x as i64) - 1;
                let svg_min_y = (min_y as i64) - 1;
                let svg_width = svg_min_x.abs() + svg_max_x.abs();
                let svg_height = svg_min_y.abs() + svg_max_y.abs();
                let shift_x = min_x.abs();
                let shift_y = min_y.abs();

                let mut num_portals = 0;

                let mut f = match File::create(format!("{}.svg", fname)) {
                    Ok(new_file) => new_file,
                    Err(why) => panic!("Couldn't create file. {}", why),
                };

                let svg_header = format!("<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\">", svg_width, svg_height, svg_width, svg_height);
                let _ = f.write(svg_header.as_ref());
                let _ = f.write(
                    format!(
                        "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\" />",
                        svg_width, svg_height, "black"
                    )
                    .as_ref(),
                );
                for linedef in linedefs.values() {
                    let a = vertices.get(&linedef.v1).unwrap();
                    let b = vertices.get(&linedef.v2).unwrap();

                    let _ = f.write(format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\" />",
                        a.x + shift_x,
                        a.y + shift_y,
                        b.x + shift_x,
                        b.y + shift_y,
                        match linedef.twosided {
                            true => "grey",
                            _ => "white",
                        },
                        "5").as_ref());

                    if linedef.special == 107 {
                        //println!("Got a Line_SetPortalTarget()");
                        num_portals += 1;
                    }
                    if linedef.special == 156 {
                        /*
                        println!("Got a Line_SetPortal()");
                        println!("arg0: {}", linedef.arg0); // target
                        println!("arg1: {}", linedef.arg1); // thisline (should be 0)
                        println!("arg2: {}", linedef.arg2); // type
                        println!("arg3: {}", linedef.arg3); // plane anchor (0|1)
                        */
                        num_portals += 1;

                        /*
                        if (linedef.arg0 != 0) {
                            // draw a line from current line to target line
                            let index = linedef.arg0 as usize;
                            let target_line = linedefs
                                .get(&index)
                                .expect("Failed to find matching line for this portal");
                            let c = vertices
                                .get(&target_line.v1)
                                .expect("Failed to find matching vertex for this line");

                            let _ = f.write(format!(
                                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\" />",
                                a.x + shift_x,
                                a.y + shift_y,
                                c.x + shift_x,
                                c.y + shift_y,
                                "red",
                                "10"
                            ).as_ref());
                        }
                        */
                    }
                }

                let _ = f.write(b"</svg>");
                // end SVG rendering

                println!("Stats:");
                println!("Number of vertices: {}", &vertices.len());
                println!("Number of linedefs: {}", &linedefs.len());
                println!("Number of portals: {}", num_portals);

                println!("Created map: {}.svg", fname);
            }
        }
    }

    exit(0);
}

// end udmfmap.rs

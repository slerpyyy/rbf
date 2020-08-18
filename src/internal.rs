use std::fmt;
use std::cmp::{max, min};
use std::num::Wrapping;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum IR {
    Start,
    Touch(isize, isize),
    Set(isize, Wrapping<u8>),
    Add(isize, Wrapping<u8>),
    Mul(isize, Wrapping<u8>),
    Move(isize),
    Store(isize),
    Loop(Vec<IR>),
    FixedLoop(Vec<IR>,isize,isize),
    Scan(Wrapping<u8>, isize),
    Fill(isize, Wrapping<u8>, isize),
    Input(isize),
    Output(isize),
}

impl fmt::Display for IR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IR::Start => write!(f, "start"),
            IR::Touch(high, low) => write!(f, "touch {:+} {:+}", high, low),
            IR::Set(off, val) => write!(f, "set {:+} {}", off, val),
            IR::Add(off, val) => write!(f, "add {:+} {}", off, val),
            IR::Mul(off, val) => write!(f, "mul {:+} {}", off, val),
            IR::Move(off) => write!(f, "mov {:+}", off),
            IR::Store(off) => write!(f, "store {:+}", off),
            IR::Loop(_) => write!(f, "loop"),
            IR::FixedLoop(_, high, low) => write!(f, "loop {:+} {:+} (fix)", high, low),
            IR::Scan(val, step) => write!(f, "scan {} {:+}", val, step),
            IR::Fill(off, val, step) => write!(f, "fill {:+} {} {:+}", off, val, step),
            IR::Input(off) => write!(f, "in {:+}", off),
            IR::Output(off) => write!(f, "out {:+}", off),
        }
    }
}

fn write_code(prog: &[IR], ind: u32, lines: &mut Vec<String>) {
    for inst in prog.iter() {
        let padding = "| ".repeat(ind as _);

        match inst {
            IR::Loop(sub) | IR::FixedLoop(sub, _, _) => {
                lines.push(padding);
                lines.push(inst.to_string());

                write_code(sub, ind + 1, lines);
            },

            IR::Fill(_, _, _) => {
                lines.push(padding.replace('|', &"#"));
                lines.push(inst.to_string());
            }

            _ => {
                lines.push(padding);
                lines.push(inst.to_string());
            }
        }
    }
}

pub fn show_code(prog: &[IR], max_width: usize) {
    let mut lines = Vec::new();
    write_code(prog, 0, &mut lines);

    let height = lines.len();
    let width = 6 + lines.iter()
        .map(String::len)
        .max()
        .unwrap_or_default();

    let columns = max(min(max_width / width, height / 24), 1);
    let rows = max(height / columns, 0) + 1;

    for y in 0..rows {
        for x in 0..columns {
            let default = String::new();
            let line = lines
                .get(y + rows * x)
                .unwrap_or(&default);
            print!("{}", line);

            if x < columns - 1 {
                let padding = " ".repeat(width - line.len());
                print!("{}", padding);
            }
        }

        println!();
    }
}

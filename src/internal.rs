use std::fmt;
use std::cmp::*;
use std::num::Wrapping;
use std::iter::*;
use std::string::*;

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
	FixedLoop(Vec<IR>),
	Scan(Wrapping<u8>, isize),
	Fill(isize, Wrapping<u8>, isize),
	Input(isize),
	Output(isize),
}

impl fmt::Display for IR {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			IR::Start        => write!(f, "start"                      ),
			IR::Touch(h,l)   => write!(f, "touch {:+} {:+}"        ,h,l),
			IR::Set(o,v)     => write!(f, "set   {:+} {}"          ,o,v),
			IR::Add(o,v)     => write!(f, "add   {:+} {}"          ,o,v),
			IR::Mul(o,v)     => write!(f, "mul   {:+} {}"          ,o,v),
			IR::Move(o)      => write!(f, "mov   {:+}"               ,o),
			IR::Store(o)     => write!(f, "store {:+}"               ,o),
			IR::Loop(_)      => write!(f, "loop"                       ),
			IR::FixedLoop(_) => write!(f, "loop  (fixed)"              ),
			IR::Scan(v,s)    => write!(f, "scan  {} {:+}"          ,v,s),
			IR::Fill(o,v,s)  => write!(f, "fill  {:+} {} {:+}"   ,o,v,s),
			IR::Input(o)     => write!(f, "in    {:+}"               ,o),
			IR::Output(o)    => write!(f, "out   {:+}"               ,o),
		}
	}
}

fn write_code (prog : &[IR], ind : u32, lines : &mut Vec<String>) {
	for inst in prog.iter() {
		let mut padding = String::new();

		for _ in 0..ind {
			padding.push_str("| ");
		}

		match inst {
			IR::Loop(sub) | IR::FixedLoop(sub) => {
				lines.push(format!("{}{}", padding, inst));

				write_code(sub, ind + 1, lines);
			},

			_ => {
				lines.push(format!("{}{}", padding, inst));
			}
		}
	}
}

pub fn show_code (prog : &[IR], max_width : usize) {
	let mut lines = Vec::new();
	write_code(prog, 0, &mut lines);

	let width = 6 + lines.iter().map(String::len).max().unwrap_or(0);
	let height = lines.len();

	let colums = max(min(max_width / width, height / 24), 1);
	let rows = max(height / colums, 0) + 1;

	for y in 0..rows {
		for x in 0..colums {
			let default = String::new();
			let line = lines.get(y + rows * x).unwrap_or(&default);
			print!("{}", line);

			if x < colums - 1 {
				let padding = " ".repeat(width - line.len());
				print!("{}", padding);
			}
		}

		println!();
	}
}

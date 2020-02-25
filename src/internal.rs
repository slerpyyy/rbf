use std::fmt;
use std::num::Wrapping;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum IR {
	Touch(isize, isize),
	Set(isize, Wrapping<u8>),
	Add(isize, Wrapping<u8>),
	Mul(isize, Wrapping<u8>),
	Move(isize),
	Store(isize),
	Loop(Vec<IR>),
	Scan(Wrapping<u8>, isize),
	Fill(isize, Wrapping<u8>, isize),
	Input(isize),
	Output(isize),
}

impl fmt::Display for IR {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			IR::Touch(high, low) => write!(f, "tch {:+} {:+}", high, low),
			IR::Set(off, val) => write!(f, "set {:+} {}", off, val),
			IR::Add(off, val) => write!(f, "add {:+} {}", off, val),
			IR::Mul(off, val) => write!(f, "mul {:+} {}", off, val),
			IR::Move(off) => write!(f, "mov {:+}", off),
			IR::Store(off) => write!(f, "stn {:+}", off),
			IR::Loop(_) => write!(f, "rep"),
			IR::Scan(val, off) => write!(f, "scn {} {:+}", val, off),
			IR::Fill(off, val, step)
				=> write!(f, "fll {:+} {} {:+}", off, val, step),
			IR::Input(off) => write!(f, "inp {:+}", off),
			IR::Output(off) => write!(f, "out {:+}", off),
		}
	}
}

pub fn show_code (prog : &[IR], ind : u32) {
	for inst in prog.iter() {
		for _ in 0..ind {
			print!("| ");
		}

		match inst {
			IR::Loop(sub) => {
				println!("{}", inst);
				show_code(sub, ind + 1);
			},

			_ => println!("{}", inst),
		}
	}
}

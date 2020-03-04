use std::fmt;
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

pub fn show_code (prog : &[IR], ind : u32) {
	for inst in prog.iter() {
		for _ in 0..ind {
			print!("| ");
		}

		match inst {
			IR::Loop(sub) | IR::FixedLoop(sub) => {
				println!("{}", inst);
				show_code(sub, ind + 1);
			},

			_ => println!("{}", inst),
		}
	}
}

use std::num::Wrapping;

#[derive(Debug)]
pub enum IR {
	Set(isize, Wrapping<u8>),
	Add(isize, Wrapping<u8>),
	Mul(isize, Wrapping<u8>),
	Move(isize),
	Loop(Vec<IR>),
	Store(isize),
	Scan(Wrapping<u8>, isize),
	Fill(isize, Wrapping<u8>, isize),
	Input(isize),
	Output(isize),
}

#[allow(dead_code)]
pub fn show_code (prog : &[IR], ind : u32) {
	for inst in prog.iter() {
		for _ in 0..ind {
			print!(" ");
		}

		match inst {
			IR::Set(off, val) => println!("set {:+} {}", off, val),
			IR::Add(off, val) => println!("add {:+} {}", off, val),
			IR::Mul(off, val) => println!("mul {:+} {}", off, val),

			IR::Move(off) => println!("mov {:+}", off),
			IR::Loop(sub) => {
				println!("loop:");
				show_code(sub, ind + 4);
			},

			IR::Store(off) => println!("str {:+}", off),
			IR::Scan(val, off) => println!("scn {} {:+}", val, off),

			IR::Fill(off, val, step)
				=> println!("fll {:+} {} {:+}", off, val, step),

				IR::Input(off) => println!("in {}", off),
			IR::Output(off) => println!("out {}", off),
		}
	}
}
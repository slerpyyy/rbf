pub enum IR {
	Set(i32,u8),
	Add(i32,u8),
	Mul(i32,u8),
	Move(i32),
	Loop(Vec<IR>),
	Store(i32),
	Scan(u8,i32),
	Fill(i32,u8,i32),
	Input(i32),
	Output(i32),
}

#[allow(dead_code)]
pub fn show_code (prog : &Vec<IR>, ind : u32) {
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
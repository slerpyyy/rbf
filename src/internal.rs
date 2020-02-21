pub enum IR {
	Set(u8,i32),
	Add(u8,i32),
	Mul(u8,i32),
	Move(i32),
	Loop(Vec<IR>),
	Scan(u8,i32),
	Fill(u8,i32,i32),
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
			IR::Set(val, off) => println!("set {} {:+}", val, off),
			IR::Add(val, off) => println!("add {} {:+}", val, off),
			IR::Mul(val, off) => println!("mul {} {:+}", val, off),

			IR::Move(off) => println!("mov {:+}", off),
			IR::Loop(sub) => {
				println!("loop:");
				show_code(sub, ind + 4);
			},

			IR::Scan(val, off) => println!("scan {} {:+}", val, off),
			IR::Fill(val, off, step)
				=> println!("fill {} {:+} {:+}", val, off, step),

			IR::Input(off) => println!("in {}", off),
			IR::Output(off) => println!("out {}", off),
		}
	}
}
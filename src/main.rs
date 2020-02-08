use std::*;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::collections::*;
use std::string::*;

#[derive(Debug)]
enum IR {
	Set(u8,i32),
	Add(u8,i32),
	Mul(u8,i32),
	Move(i32),
	Loop(Vec<IR>),
	Scan(i32),
	Input(i32),
	Output(i32),
}

fn show_code (prog : &Vec<IR>, ind : u32) {
	for inst in prog.iter() {
		for _ in 0..ind {
			print!(" ");
		}

		match inst {
			IR::Set(val, off) => println!("set {} ({})", val, off),
			IR::Add(val, off) => println!("add {} ({})", val, off),
			IR::Mul(val, off) => println!("mul {} ({})", val, off),

			IR::Move(off) => println!("mov {}", off),

			IR::Loop(sub) => {
				println!("loop:");
				show_code(sub, ind + 4);
			},

			IR::Scan(off) => println!("scan {}", off),
			IR::Input(off) => println!("in {}", off),
			IR::Output(off) => println!("out {}", off),
		}
	}
}

fn loop_logic (out_list : &mut Vec<IR>, in_list : Vec<IR>) {
	if in_list.len() == 1 {
		if let Some(IR::Move(val)) = in_list.first() {
			out_list.push(IR::Scan(*val));
			return;
		}
	}

	let is_clear = || {
		if in_list.len() == 1 {
			if let Some(IR::Add(val, 0)) = in_list.first() {
				if *val == 1 || *val == 255 {
					return true;
				}
			}
		}
		false
	};

	if is_clear() {
		loop {
			match out_list.last() {
				Some(IR::Add(_, off)) => if *off == 0 { out_list.pop(); },
				Some(IR::Set(_, off)) => if *off == 0 { out_list.pop(); },
				_ => break,
			}
		}

		out_list.push(IR::Set(0, 0));
		return;
	}

	let flat_loop = || {
		let mut pos = 0;
		let mut sum = 1;

		for x in in_list.iter() {
			match x {
				IR::Move(val) => pos += val,

				IR::Add(val, off) => if (*off + pos) == 0 {
					sum += *val as i32;
					sum &= 0xff;
				},

				_ => return false,
			}
		}

		(pos == 0 && sum == 0)
	};

	if flat_loop() {
		for op in in_list.iter() {
			match *op {
				IR::Add(val, off) => {
					if !(val == 255 && off == 0) {
						out_list.push(IR::Mul(val, off))
					}
				},
				_ => (),
			}
		}

		out_list.push(IR::Set(0, 0));
		return;
	}

	out_list.push(IR::Loop(in_list));
}

fn munch_forward (list : &Vec<u8>, index : &mut usize, inc : u8, dec : u8) -> i32 {
	let mut sum = 0;

	while let Some(byte) = list.get(*index) {
		if *byte == inc { sum += 1; }
		else if *byte == dec { sum -= 1; }
		else { break; }

		*index += 1;
	}

	*index -= 1;
	sum
}

fn parse(code : &Vec<u8>, mut index : usize) -> (Vec<IR>, usize) {
	let mut list = Vec::new();
	let mut off_acc = 0;

	while index < code.len() {
		match *code.get(index).unwrap() as char {
			',' => {
				list.push(IR::Input(off_acc));
			},

			'.' => {
				list.push(IR::Output(off_acc));
			},

			'+' | '-' => {
				let munch = munch_forward(&code, &mut index, '+' as u8, '-' as u8);
				let sum = (munch & 0xff) as u16;

				loop {
					if let Some(IR::Set(val, off)) = list.last_mut() {
						if *off == off_acc {
							*val = (*val as u16 + sum) as u8;
							break;
						}
					}

					if let Some(IR::Add(val, off)) = list.last_mut() {
						if *off == off_acc {
							*val = (*val as u16 + sum) as u8;
							if *val == 0 { list.pop(); }
							break;
						}
					}

					list.push(IR::Add(sum as u8, off_acc));
					break;
				};
			},

			'<' | '>' => {
				off_acc += munch_forward(&code, &mut index, '>' as u8, '<' as u8);
			},

			'[' => {
				if off_acc != 0 {
					list.push(IR::Move(off_acc));
					off_acc = 0;
				}

				let (content, new_index) = parse(&code, index + 1);
				index = new_index;

				loop_logic(&mut list, content);
			},

			']' => break,

			_ => (),
		}

		index += 1;
	}

	if off_acc != 0 {
		list.push(IR::Move(off_acc));
	}

	(list, index)
}

fn touch_on_tape (tape : &mut VecDeque<u8>, index : &mut usize, offset : i32) -> usize{
	let mut target = *index as i32 + offset;

	while target < 0 {
		tape.push_front(0u8);
		target += 1;
		*index += 1;
	}

	while target >= tape.len() as i32 {
		tape.push_back(0u8);
	}

	target as usize
}

fn execute (prog : &Vec<IR>, tape : &mut VecDeque<u8>, mut index : usize) -> (String, usize) {
	let mut out_buffer = String::new();

	for inst in prog.iter() {
		//println!("\t > {:03} {:?}\t\t{:?}", index, tape, inst);

		match inst {
			IR::Set(val, off) => {
				let target = touch_on_tape(tape, &mut index, *off);

				if let Some(cell) = tape.get_mut(target) {
					*cell = *val;
				}
			},

			IR::Add(val, off) => {
				let target = touch_on_tape(tape, &mut index, *off);

				if let Some(cell) = tape.get_mut(target) {
					*cell = (*cell as u32 + *val as u32) as u8;
				}
			},

			IR::Mul(val, off) => {
				let target = touch_on_tape(tape, &mut index, *off);

				let factor = *tape.get(index).unwrap();

				if let Some(cell) = tape.get_mut(target as usize) {
					let adder = *val as u32 * factor as u32;
					*cell = (*cell as u32 + adder) as u8;
				}
			}

			IR::Move(off) => {
				let target = touch_on_tape(tape, &mut index, *off);
				index = target;
			},

			IR::Scan(step) => {
				loop {
					if let Some(0) = tape.get(index) {
						break;
					}

					let target = touch_on_tape(tape, &mut index, *step);
					index = target;
				}
			},

			IR::Input(_) => unimplemented!(),

			IR::Output(off) => {
				let target = touch_on_tape(tape, &mut index, *off);

				if let Some(cell) = tape.get(target) {
					print!("{}", *cell as char);
				}
			},

			IR::Loop(loop_prog) => loop {
				if let Some(n) = tape.get(index) {
					if *n == 0 {
						break;
					}
				}

				let (new_str, new_index) = execute(&loop_prog, tape, index);

				out_buffer.push_str(&new_str);
				index = new_index;
			},
		}
	}

	(out_buffer, index)
}

fn main() -> io::Result<()> {
	let mut file = File::open("test.b")?;
	let mut buffer = Vec::new();
	let index = 0;

	file.read_to_end(&mut buffer)?;

	let (prog, _) = parse(&buffer, index);

	show_code(&prog, 0);

	let mut tape = VecDeque::new();
	tape.push_back(0u8);

	execute(&prog, &mut tape, 0);

	io::Result::Ok(())
}

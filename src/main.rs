use std::*;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::collections::*;
use std::iter::*;
use clap::*;

#[derive(Debug)]
enum IR {
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
fn show_code (prog : &Vec<IR>, ind : u32) {
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
			IR::Fill(val, off, step) => println!("fill {} {:+} {:+}", val, off, step),
			IR::Input(off) => println!("in {}", off),
			IR::Output(off) => println!("out {}", off),
		}
	}
}

#[inline]
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

#[inline]
fn clear_loop (out_list : &mut Vec<IR>, in_list : &Vec<IR>, offset : &mut i32) -> bool {
	if in_list.len() != 1 {
		return false;
	}

	if let Some(IR::Add(val, 0)) = in_list.first() {
		if *val != 1 && *val != 255 {
			return false;
		}
	} else {
		return false;
	}

	loop {
		match out_list.last() {
			Some(IR::Set(_, off)) | Some(IR::Add(_, off)) => {
				if *off == *offset {
					out_list.pop();
				} else {
					break;
				}
			},

			_ => break,
		}
	}

	out_list.push(IR::Set(0, *offset));
	true
}

#[inline]
fn flat_loop (out_list : &mut Vec<IR>, in_list : &Vec<IR>) -> bool {
	let mut pos = 0;
	let mut sum = 1;

	for x in in_list.iter() {
		match *x {
			IR::Move(off) => pos += off,

			IR::Add(val, off) => if (off + pos) == 0 {
				sum += val as i32;
				sum &= 0xff;
			},

			_ => return false,
		}
	}

	if pos != 0 || sum != 0 {
		return false;
	}

	for x in in_list.iter() {
		if let IR::Add(val, off) = *x {
			if !(val == 255 && off == 0) {
				out_list.push(IR::Mul(val, off))
			}
		}
	}

	out_list.push(IR::Set(0, 0));
	true
}

#[inline]
fn scan_loop (out_list : &mut Vec<IR>, in_list : &Vec<IR>) -> bool {
	let mut step = 0;
	let mut set_step = false;
	let mut start_cell = 0u16;
	let mut end_cell = 0u16;

	for x in in_list.iter() {
		match x {
			IR::Move(_) | IR::Add(_,_) => (),
			_ => return false,
		}
	}

	for x in in_list.iter() {
		if let IR::Move(off) = x {
			if set_step {
				return false;
			}

			step = *off;
			set_step = true;
		}
	}

	if !set_step {
		return false;
	}

	for x in in_list.iter() {
		if let IR::Add(val, off) = x {
			if *off == 0 {
				start_cell += *val as u16;
				start_cell &= 0xff;
				continue;
			}

			if *off == step {
				end_cell += *val as u16;
				end_cell &= 0xff;
				continue;
			}

			return false;
		}
	}

	if (start_cell + end_cell) as u8 != 0 {
		return false;
	}

	out_list.push(IR::Scan(start_cell as u8, step));
	true
}

#[inline]
fn fill_loop (out_list : &mut Vec<IR>, in_list : &Vec<IR>) -> bool {
	if in_list.len() != 2 {
		return false;
	}

	if let Some(IR::Set(val, off)) = in_list.first() {
		if let Some(IR::Move(step)) = in_list.last() {
			out_list.push(IR::Fill(*val, *off, *step));
			return true;
		}
	}

	false
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
				let (content, new_index) = parse(&code, index + 1);
				index = new_index;

				loop {
					if clear_loop(&mut list, &content, &mut off_acc) { break; }

					if off_acc != 0 {
						list.push(IR::Move(off_acc));
						off_acc = 0;
					}

					if fill_loop(&mut list, &content) { break; }
					if flat_loop(&mut list, &content) { break; }
					if scan_loop(&mut list, &content) { break; }

					list.push(IR::Loop(content));
					break;
				}
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

#[inline]
fn touch_on_tape (tape : &mut VecDeque<u8>, index : &mut usize, offset : i32) -> usize{
	let target = *index as i32 + offset;

	if target < 0 {
		for _ in 0..(-target) {
			tape.push_front(0u8);
		}

		*index = (*index as i32 - target) as usize;
		return 0;
	}

	let diff = target - tape.len() as i32;

	if diff >= 0 {
		let extra_diff = diff + 32;
		tape.extend(repeat(0u8).take(extra_diff as usize));
	}

	target as usize
}

fn execute (prog : &Vec<IR>, tape : &mut VecDeque<u8>, mut index : usize) -> usize {
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
					*cell = (*cell as u16 + *val as u16) as u8;
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

			IR::Scan(val, step) => {
				if let Some(cell) = tape.get_mut(index) {
					*cell = (*cell as u16 + *val as u16) as u8;
				}

				loop {
					if *(tape.get(index).unwrap()) == *val {
						break;
					}

					let target = touch_on_tape(tape, &mut index, *step);
					index = target;
				}

				if let Some(cell) = tape.get_mut(index) {
					*cell = (*cell as u16 + 0x100 - *val as u16) as u8;
				}
			},

			IR::Fill(val, off, step) => loop {
				if *(tape.get(index).unwrap()) == 0 {
					break;
				}

				let write_target = touch_on_tape(tape, &mut index, *off);
				if let Some(cell) = tape.get_mut(write_target) {
					*cell = *val;
				}

				let step_target = touch_on_tape(tape, &mut index, *step);
				index = step_target;
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

				index = execute(&loop_prog, tape, index);
			},
		}
	}

	index
}

fn main() -> io::Result<()> {

	let matches = App::new("rbf")
		.version("0.1.0")
		.about("A simple optimizing Brainfuck interpreter written in Rust.")
		.arg(Arg::with_name("file")
			.value_name("FILE")
			.help("Sets the input file to use")
			.required(false)
			.index(1))
		.arg(Arg::with_name("command")
			.short("c")
			.long("cmd")
			.value_name("CODE")
			.help("Sets a custom config file")
			.takes_value(true))
		.arg(Arg::with_name("verbosity")
			.short("v")
			.multiple(true)
			.help("Sets the level of verbosity"))
		.get_matches();

	let command = matches.value_of("command");
	let filename = matches.value_of("file");
	let verbosity = matches.occurrences_of("verbosity");

	let mut buffer = Vec::new();

	if let Some(cmd_str) = command {
		buffer = cmd_str.as_bytes().to_vec();
	} else {
		if let Some(filename_str) = filename {
			if filename_str == "-" {
				io::stdin().read_to_end(&mut buffer)?;
			} else {
				let file = File::open(filename_str);

				if let Err(err) = file {
					eprintln!("ERROR : {}", err);
					std::process::exit(-1);
				}

				let mut raw_file = file?;
				raw_file.read_to_end(&mut buffer)?;
			}
		} else {
			eprintln!("ERROR : No input found");
			std::process::exit(-1);
		}
	}


	let (prog, _) = parse(&buffer, 0);

	if verbosity > 0 {
		show_code(&prog, 0);
	}

	let mut tape = VecDeque::with_capacity(0x10000);
	for _ in 0..0x100 { tape.push_back(0u8); }

	execute(&prog, &mut tape, 0);

	io::Result::Ok(())
}

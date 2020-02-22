use std::num::Wrapping;

use crate::internal::*;

#[inline(always)]
fn munch_forward (
	list : &Vec<u8>,
	index : &mut usize,
	inc : char,
	dec : char
) -> i32 {
	let inc_u8 = inc as u8;
	let dec_u8 = dec as u8;
	let mut sum = 0;

	while let Some(&byte) = list.get(*index) {
		match () {
			_ if byte == inc_u8 => sum += 1,
			_ if byte == dec_u8 => sum -= 1,
			_ => break,
		}

		*index += 1;
	}

	sum
}

#[inline]
fn add_inst (out_list : &mut Vec<IR>, sum : Wrapping<u8>, offset : &isize) {
	if let Some(IR::Set(off, val)) = out_list.last_mut() {
		if *off == *offset {
			*val += sum;
			return;
		}
	}

	if let Some(IR::Add(off, val)) = out_list.last_mut() {
		if *off == *offset {
			*val += sum;
			if *val == Wrapping(0u8) { out_list.pop(); }
			return;
		}
	}

	out_list.push(IR::Add(*offset, sum));
}

#[inline]
fn clear_loop (
	out_list : &mut Vec<IR>,
	in_list : &Vec<IR>,
	offset : &mut isize
) -> bool {
	if in_list.len() != 1 {
		return false;
	}

	if let Some(IR::Add(0, val)) = in_list.first() {
		if *val != Wrapping(1u8) && *val != Wrapping(255u8) {
			return false;
		}
	} else {
		return false;
	}

	loop {
		match out_list.last() {
			Some(IR::Set(off, _)) | Some(IR::Add(off, _)) => {
				if *off == *offset {
					out_list.pop();
				} else {
					break;
				}
			},

			_ => break,
		}
	}

	out_list.push(IR::Set(*offset, Wrapping(0u8)));
	true
}

#[inline]
fn flat_loop (
	out_list : &mut Vec<IR>,
	in_list : &Vec<IR>,
	offset : &mut isize
) -> bool {
	let mut sum = Wrapping(1u8);
	let mut pos = 0;

	for x in in_list.iter() {
		match x {
			IR::Move(off) => pos += *off,
			IR::Add(off, val) => if *off + pos == 0 { sum += *val },
			_ => return false,
		}
	}

	if (pos != 0) || (sum.0 != 0) {
		return false;
	}

	out_list.push(IR::Store(*offset));

	for x in in_list.iter() {
		match x {
			IR::Add(0, _) => (),
			IR::Add(off, val) => {
				out_list.push(IR::Mul(*offset + *off, *val));
			}
			_ => (),
		}
	}

	true
}

#[inline]
fn scan_loop (out_list : &mut Vec<IR>, in_list : &Vec<IR>) -> bool {
	let mut step = 0;
	let mut set_step = false;
	let mut start_cell = Wrapping(0u8);
	let mut end_cell = Wrapping(0u8);

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
		if let IR::Add(off, val) = x {
			if *off == 0 {
				start_cell += *val;
				continue;
			}

			if *off == step {
				end_cell += *val;
				continue;
			}

			return false;
		}
	}

	if start_cell + end_cell != Wrapping(0u8) {
		return false;
	}

	out_list.push(IR::Scan(start_cell, step));
	true
}

#[inline]
fn fill_loop (out_list : &mut Vec<IR>, in_list : &Vec<IR>) -> bool {
	if in_list.len() != 2 {
		return false;
	}

	if let Some(IR::Set(off, val)) = in_list.first() {
		if let Some(IR::Move(step)) = in_list.last() {
			out_list.push(IR::Fill(*off, *val, *step));
			return true;
		}
	}

	false
}

pub fn parse(code : &Vec<u8>, mut index : usize) -> (Vec<IR>, usize) {
	let mut prog = Vec::new();
	let mut off_acc = 0isize;

	while index < code.len() {
		match *code.get(index).unwrap() as char {
			',' => {
				prog.push(IR::Input(off_acc));
			},

			'.' => {
				prog.push(IR::Output(off_acc));
			},

			'+' | '-' => {
				let munch = munch_forward(&code, &mut index, '+', '-');
				let sum = Wrapping(munch as u8);

				add_inst(&mut prog, sum, &off_acc);
				continue;
			},

			'<' | '>' => {
				let munch = munch_forward(&code, &mut index, '>', '<');
				off_acc += munch as isize;
				continue;
			},

			'[' => {
				let (content, new_index) = parse(&code, index + 1);
				index = new_index;

				loop {
					if clear_loop(&mut prog, &content, &mut off_acc) { break; }
					if flat_loop(&mut prog, &content, &mut off_acc) { break; }

					if off_acc != 0 {
						prog.push(IR::Move(off_acc));
						off_acc = 0;
					}

					if fill_loop(&mut prog, &content) { break; }
					if scan_loop(&mut prog, &content) { break; }

					prog.push(IR::Loop(content));
					break;
				}
			},

			']' => break,

			_ => (),
		}

		index += 1;
	}

	if off_acc != 0 {
		prog.push(IR::Move(off_acc));
	}

	(prog, index)
}
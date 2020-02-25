use std::iter::*;
use std::num::Wrapping;

use crate::internal::*;

pub fn check_valid (bytes : &[u8]) -> bool {
	let mut sum = 0;

	// returns true for 'Y', '[', ']' or '_'
	let f = |k : &&u8| -> bool { **k & 0xf9 == 0x59 };

	for b in bytes.iter().filter(f) {
		match b {
			b'[' => sum += 1,
			b']' => {
				sum -= 1;
				if sum < 0 { return false; }
			},
			_ => (),
		}
	}

	(sum == 0)
}

#[inline]
fn munch_forward (
	bytes : &[u8],
	index : &mut usize,
	inc : u8,
	dec : u8
) -> i32 {
	let mut sum = 0;

	while let Some(&byte) = bytes.get(*index) {
		match () {
			_ if byte == inc => sum += 1,
			_ if byte == dec => sum -= 1,
			_ => break,
		}

		*index += 1;
	}

	sum
}

#[inline]
fn add_inst (out_list : &mut Vec<IR>, sum : Wrapping<u8>, offset : isize) {
	match out_list.last_mut() {
		Some(IR::Set(off, val)) if *off == offset => {
			*val += sum;
		},

		Some(IR::Add(off, val)) if *off == offset => {
			*val += sum;
			if *val == Wrapping(0u8) {
				out_list.pop();
			}
		},

		_ => if sum != Wrapping(0u8) {
			out_list.push(IR::Add(offset, sum))
		},
	}
}

#[inline]
fn move_inst (out_list : &mut Vec<IR>, offset : &mut isize) -> bool {
	if *offset != 0 {
		out_list.push(IR::Move(*offset));
		*offset = 0;
		return true;
	}

	false
}

#[inline]
fn clear_loop (
	out_list : &mut Vec<IR>,
	in_list : &[IR],
	offset : &mut isize
) -> bool {
	if in_list.len() != 2 {
		return false;
	}

	if let Some(IR::Add(0, val)) = in_list.last() {
		if *val != Wrapping(1u8) && *val != Wrapping(255u8) {
			return false;
		}
	} else {
		return false;
	}

	while let Some(IR::Set(off, _))
			| Some(IR::Add(off, _)) = out_list.last() {
		if *off == *offset {
			out_list.pop();
		} else {
			break;
		}
	}

	out_list.push(IR::Set(*offset, Wrapping(0u8)));
	true
}

#[inline]
fn flat_loop (
	out_list : &mut Vec<IR>,
	in_list : &[IR],
	offset : &mut isize
) -> bool {
	let mut sum = Wrapping(1u8);

	for x in in_list.iter() {
		match x {
			IR::Add(0, val) => { sum += *val },
			IR::Touch(_, _) | IR::Add(_, _) => (),
			_ => return false,
		}
	}

	if sum != Wrapping(0u8) {
		return false;
	}

	out_list.push(IR::Store(*offset));

	for x in in_list.iter() {
		match x {
			IR::Add(0, _) => (),
			IR::Add(off, val) => {
				out_list.push(IR::Mul(*offset + *off, *val))
			},
			_ => (),
		}
	}

	true
}

#[inline]
fn scan_loop (
	out_list : &mut Vec<IR>,
	in_list : &[IR],
	offset : &mut isize
) -> bool {
	let mut step = 0;
	let mut set_step = false;
	let mut start_cell = Wrapping(0u8);
	let mut end_cell = Wrapping(0u8);

	for x in in_list.iter() {
		match x {
			IR::Move(_) | IR::Add(_,_) | IR::Touch(_,_) => (),
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
			match *off {
				k if k == 0 => start_cell += *val,
				k if k == step => end_cell += *val,
				_ => return false,
			}
		}
	}

	if start_cell + end_cell != Wrapping(0u8) {
		return false;
	}

	add_inst(out_list, start_cell, *offset);
	move_inst(out_list, offset);

	out_list.push(IR::Scan(start_cell, step));
	out_list.push(IR::Touch(0, 0));

	add_inst(out_list, end_cell, 0);

	true
}

#[inline]
fn fill_loop (out_list : &mut Vec<IR>, in_list : &[IR]) -> bool {
	if in_list.len() != 3 {
		return false;
	}

	if let Some(IR::Set(off, val)) = in_list.get(1) {
		if let Some(IR::Move(step)) = in_list.get(2) {
			out_list.push(IR::Fill(*off, *val, *step));
			out_list.push(IR::Touch(0, 0));
			return true;
		}
	}

	false
}

#[inline]
fn fixed_loop (
	out_list : &mut Vec<IR>,
	in_list : &[IR],
) -> bool {
	let mut pos = 0;

	for x in in_list.iter() {
		match x {
			IR::Move(off) => pos += *off,

			IR::Loop(_) | IR::Scan(_,_) | IR::Fill(_,_,_)
				=> return false,

			_ => (),
		}
	}

	if pos != 0 {
		return false;
	}

	for i in (0..out_list.len()).rev() {
		if let Some(IR::Touch(high, low)) = out_list.get_mut(i) {
			if let Some(IR::Touch(new_high, new_low)) = in_list.first() {
				if *new_high > *high { *high = *new_high; }
				if *new_low < *low { *low = *new_low; }

				let tail : Vec<IR> = in_list.to_vec()
					.into_iter().skip(1).collect::<Vec<IR>>();

				out_list.push(IR::FixedLoop(tail));
				return true;
			}

			return false;
		}
	}

	false
}

#[inline]
fn loop_inst (
	out_list : &mut Vec<IR>,
	in_list : Vec<IR>,
	offset : &mut isize
) {
	if clear_loop(out_list, &in_list, offset) { return; }
	if flat_loop(out_list, &in_list, offset) { return; }
	if scan_loop(out_list, &in_list, offset) { return; }

	move_inst(out_list, offset);

	if fill_loop(out_list, &in_list) { return; }
	if fixed_loop(out_list, &in_list) { return; }

	out_list.push(IR::Loop(in_list));
}

pub fn parse(code : &[u8], mut index : &mut usize) -> Vec<IR> {
	let mut prog = Vec::new();
	let mut off_acc = 0isize;

	prog.push(IR::Touch(0,0));

	while *index < code.len() {
		match *code.get(*index).unwrap() {
			b',' => {
				prog.push(IR::Input(off_acc));
			},

			b'.' => {
				prog.push(IR::Output(off_acc));
			},

			b'+' | b'-' => {
				let munch = munch_forward(&code, &mut index, b'+', b'-');
				let sum = Wrapping(munch as u8);

				add_inst(&mut prog, sum, off_acc);
				continue;
			},

			b'<' | b'>' => {
				let munch = munch_forward(&code, &mut index, b'>', b'<');
				off_acc += munch as isize;
				continue;
			},

			b'[' => {
				*index += 1;
				let content = parse(&code, index);

				loop_inst(&mut prog, content, &mut off_acc);
			},

			b']' => break,

			_ => (),
		}

		*index += 1;
	}

	let mut upper = 0;
	let mut lower = 0;

	for i in (0..prog.len()).rev() {
		match prog.get_mut(i).unwrap() {
			IR::Touch(high, low) => {
				if upper > *high { *high = upper; }
				if lower < *low { *low = lower; }

				upper = 0;
				lower = 0;
			}

			IR::Set(off,_) | IR::Add(off,_) | IR::Mul(off,_) |
			IR::Store(off) | IR::Input(off) | IR::Output(off) => {
				if *off > upper { upper = *off; }
				if *off < lower { lower = *off; }
			}

			IR::Move(off) => {
				let shifted_upper = *off + upper;
				let shifted_lower = *off + lower;

				if shifted_upper > upper { upper = shifted_upper; }
				if shifted_lower < lower { lower = shifted_lower; }
			},

			_ => (),
		}
	}

	if off_acc != 0 {
		prog.push(IR::Move(off_acc));
	}

	prog
}

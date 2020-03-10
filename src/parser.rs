use std::cmp::*;
use std::iter::*;
use std::num::Wrapping;

use crate::internal::*;

mod helper;
use helper::*;

pub fn check_valid (bytes : &[u8]) -> bool {
	let mut sum = 0;

	// returns true for 'Y', '[', ']' and '_'
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
fn set_touch_inst (inout_list : &mut Vec<IR>) {
	let mut upper = 0;
	let mut lower = 0;

	for i in (0..inout_list.len()).rev() {
		match inout_list.get_mut(i).unwrap() {
			IR::Touch(high, low) => {
				*high = max(*high, upper);
				*low = min(*low, lower);

				upper = 0;
				lower = 0;
			}

			IR::FixedLoop(_, high, low) => {
				upper = max(*high, upper);
				lower = min(*low, lower);
			},

			IR::Set(off,_) | IR::Add(off,_) | IR::Mul(off,_) |
			IR::Store(off) | IR::Input(off) | IR::Output(off) => {
				upper = max(upper, *off);
				lower = min(lower, *off);
			}

			IR::Move(off) => {
				upper += max(0, *off);
				lower += min(0, *off);
			},

			_ => (),
		}
	}
}

pub fn parse_recursive (
	code : &[u8],
	mut index : &mut usize,
	root : bool
) -> Vec<IR> {
	let mut prog = Vec::new();
	let mut off_acc = 0isize;

	if root {
		prog.push(IR::Start);
	}

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
				let content = parse_recursive(&code, index, false);

				loop_inst(&mut prog, content, &mut off_acc);
			},

			b']' => break,

			_ => (),
		}

		*index += 1;
	}

	set_touch_inst(&mut prog);

	if !root {
		move_inst(&mut prog, &mut off_acc);
	}

	prog
}

pub fn parse (code : &[u8]) -> Vec<IR> {
	let mut index = 0;
	parse_recursive(code, &mut index, true)
}

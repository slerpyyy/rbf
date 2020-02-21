use std::*;
use std::io::prelude::*;
use std::collections::*;
use std::iter::*;

use crate::internal::*;

#[inline]
fn touch_on_tape (
	tape : &mut VecDeque<u8>,
	index : &mut usize,
	offset : i32
) -> usize {
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

pub fn eval (
	prog : &Vec<IR>,
	input : &mut dyn Read,
	output : &mut dyn Write,
	tape : &mut VecDeque<u8>,
	mut index : usize
) -> io::Result<usize> {
	for inst in prog.iter() {
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
					if tape.get(index).unwrap() == val {
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

			IR::Input(off) => {
				let target = touch_on_tape(tape, &mut index, *off);

				let mut buffer = [0u8];
				if let Err(_) = input.read_exact(&mut buffer) {
					continue;
				}

				if let Some(cell) = tape.get_mut(target) {
					*cell = buffer[0];
				}
			},

			IR::Output(off) => {
				let target = touch_on_tape(tape, &mut index, *off);

				if let Some(cell) = tape.get(target) {
					output.write(&[*cell])?;
				}
			},

			IR::Loop(loop_prog) => loop {
				if let Some(n) = tape.get(index) {
					if *n == 0 {
						break;
					}
				}

				index = eval(&loop_prog, input, output, tape, index)?;
			},
		}
	}

	io::Result::Ok(index)
}
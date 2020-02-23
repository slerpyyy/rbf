use std::*;
use std::io::prelude::*;
use std::num::Wrapping;
use std::collections::*;
use std::iter::*;

use crate::internal::*;

#[inline]
fn touch_on_tape (
	tape : &mut VecDeque<Wrapping<u8>>,
	index : &mut isize,
	offset : isize
) -> isize {
	const MARGIN : isize = 32;
	let target = *index as isize + offset;

	if target < 0 {
		for _ in target..MARGIN {
			tape.push_front(Wrapping(0u8));
		}

		*index += MARGIN - target;
		return MARGIN;
	}

	let diff = target - tape.len() as isize;

	if diff >= 0 {
		let extra_diff = (diff + MARGIN) as usize;
		tape.extend(repeat(Wrapping(0u8)).take(extra_diff));
	}

	target
}

#[inline]
fn tape_cell (
	tape : &VecDeque<Wrapping<u8>>,
	index : isize,
	offset : isize
) -> Wrapping<u8> {
	let target = index + offset;

	if target < 0 {
		return Wrapping(0u8)
	}

	*tape.get(target as usize).unwrap_or(&Wrapping(0u8))
}

pub fn eval (
	prog : &[IR],
	input : &mut dyn Read,
	output : &mut dyn Write,
	tape : &mut VecDeque<Wrapping<u8>>,
	mut index : isize
) -> io::Result<isize> {
	let mut register = Wrapping(0u8);

	for inst in prog.iter() {
		match inst {
			IR::Set(off, val) => {
				let target = touch_on_tape(tape, &mut index, *off);
				if let Some(cell) = tape.get_mut(target as usize) {
					*cell = *val;
				}
			},

			IR::Add(off, val) => {
				let target = touch_on_tape(tape, &mut index, *off);
				if let Some(cell) = tape.get_mut(target as usize) {
					*cell += *val;
				}
			},

			IR::Mul(off, val) => {
				let term = *val * register;
				let target = touch_on_tape(tape, &mut index, *off);
				if let Some(cell) = tape.get_mut(target as usize) {
					*cell += term;
				}
			}

			IR::Move(off) => index += *off,

			IR::Store(off) => {
				let target = touch_on_tape(tape, &mut index, *off);
				if let Some(cell) = tape.get_mut(target as usize) {
					register = *cell;
					*cell = Wrapping(0u8);
				}
			},

			IR::Scan(val, step) => {
				touch_on_tape(tape, &mut index, 0);
				if let Some(cell) = tape.get_mut(index as usize) {
					*cell += *val;
				}

				loop {
					if tape_cell(tape, index, 0) == *val { break; }
					index += *step;
				}

				touch_on_tape(tape, &mut index, 0);
				if let Some(cell) = tape.get_mut(index as usize) {
					*cell -= *val;
				}
			},

			IR::Fill(off, val, step) => loop {
				if tape_cell(tape, index, 0) == Wrapping(0u8) {
					break;
				}

				let target = touch_on_tape(tape, &mut index, *off);
				if let Some(cell) = tape.get_mut(target as usize) {
					*cell = *val;
				}

				index += *step;
			},

			IR::Input(off) => {
				let mut buffer = [0u8];
				if input.read_exact(&mut buffer).is_err() {
					continue;
				}

				let target = touch_on_tape(tape, &mut index, *off);
				if let Some(cell) = tape.get_mut(target as usize) {
					*cell = Wrapping(buffer[0]);
				}
			},

			IR::Output(off) => {
				let cell = tape_cell(tape, index, *off);
				output.write_all(&[cell.0])?;
			},

			IR::Loop(loop_prog) => loop {
				if tape_cell(tape, index, 0) == Wrapping(0u8) {
					break;
				}

				index = eval(&loop_prog, input, output, tape, index)?;
			},
		}
	}

	io::Result::Ok(index)
}

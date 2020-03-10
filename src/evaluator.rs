use std::io;
use std::cmp::*;
use std::num::Wrapping;
use std::collections::*;
use std::iter::*;

use crate::internal::*;

#[inline]
fn touch_range (
	tape : &mut VecDeque<Wrapping<u8>>,
	index : &mut isize,
	high : isize,
	low : isize
) {
	const MARGIN : usize = 16;
	let upper = *index as isize + high + 1;
	let lower = *index as isize + low;

	let upper_diff = max(0, upper - tape.len() as isize) as usize;
	let lower_diff = max(0, -lower) as usize;
	let total_diff = upper_diff + lower_diff;

	if total_diff > 0 {
		let ext = total_diff + MARGIN * 2;
		let rot = lower_diff + MARGIN;

		tape.extend(repeat(Wrapping(0u8)).take(ext));
		tape.rotate_right(rot);

		*index += rot as isize;
	}
}

#[inline]
fn touch_cell (
	tape : &mut VecDeque<Wrapping<u8>>,
	index : &mut isize,
	offset : isize
) -> isize {
	const MARGIN : isize = 16;
	let target = *index as isize + offset;

	if target < 0 {
		let n = (MARGIN - target) as usize;
		tape.extend(repeat(Wrapping(0u8)).take(n));
		tape.rotate_right(n);

		*index += n as isize;
		return MARGIN;
	}

	let diff = target - tape.len() as isize;

	if diff >= 0 {
		let n = (MARGIN + diff) as usize;
		tape.extend(repeat(Wrapping(0u8)).take(n));
	}

	target
}

macro_rules! cell {
	(read, $tape:ident, $index:expr) => {
		$tape.get($index as usize).unwrap_or(&Wrapping(0u8))
	};

	(write, $tape:ident, $index:expr) => {
		$tape.get_mut($index as usize).unwrap()
	};
}


pub fn eval (
	prog : &[IR],
	input : &mut dyn io::Read,
	output : &mut dyn io::Write,
	tape : &mut VecDeque<Wrapping<u8>>,
	mut index : isize
) -> io::Result<isize> {
	let mut register = Wrapping(0u8);

	for inst in prog.iter() {
		match inst {
			IR::Touch(high, low) => {
				touch_range(tape, &mut index, *high, *low);
			},

			IR::Set(off, val) => {
				*cell!(write, tape, index + off) = *val;
			},

			IR::Add(off, val) => {
				*cell!(write, tape, index + off) += *val;
			},

			IR::Mul(off, val) => {
				let term = *val * register;
				*cell!(write, tape, index + off) += term;
			}

			IR::Move(off) => {
				index += *off
			},

			IR::Store(off) => {
				let cell = cell!(write, tape, index + off);

				register = *cell;
				*cell = Wrapping(0u8);
			},

			IR::Scan(val, step) => loop {
				if *cell!(read, tape, index) == *val {
					break;
				}

				index += *step;
			},

			IR::Fill(off, val, step) => loop {
				if *cell!(read, tape, index) == Wrapping(0u8) {
					break;
				}

				let target = touch_cell(tape, &mut index, *off);
				*cell!(write, tape, target) = *val;

				index += *step;
			},

			IR::Input(off) => {
				let mut buffer = [0u8];
				if input.read_exact(&mut buffer).is_err() {
					continue;
				}

				let val = Wrapping(buffer[0]);
				*cell!(write, tape, index + off) = val;
			},

			IR::Output(off) => {
				let cell = *cell!(read, tape, index + off);
				output.write_all(&[cell.0])?;
			},

			IR::Loop(loop_prog) | IR::FixedLoop(loop_prog,_,_) => loop {
				if *cell!(read, tape, index) == Wrapping(0u8) {
					break;
				}

				index = eval(&loop_prog, input, output, tape, index)?;
			},

			_ => (),
		}
	}

	io::Result::Ok(index)
}

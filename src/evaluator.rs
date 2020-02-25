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

macro_rules! cell_read {
	($tape:ident, $index:expr) => {
		$tape.get($index as usize).unwrap_or(&Wrapping(0u8))
	};

	($tape:ident, $index:expr, $offset:expr) => {
		cell_read!($tape, $index + $offset)
	};
}

macro_rules! cell_write {
	($tape:ident, $index:expr) => {
		$tape.get_mut($index as usize).unwrap()
	};

	($tape:ident, $index:expr, $offset:expr) => {
		cell_write!($tape, $index + $offset)
	};
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
			IR::Touch(high, low) => {
				touch_on_tape(tape, &mut index, *high + 1);
				touch_on_tape(tape, &mut index, *low);
			},

			IR::Set(off, val) => {
				*cell_write!(tape, index, off) = *val;
			},

			IR::Add(off, val) => {
				*cell_write!(tape, index, off) += *val;
			},

			IR::Mul(off, val) => {
				let term = *val * register;
				*cell_write!(tape, index, off) += term;
			}

			IR::Move(off) => {
				index += *off
			},

			IR::Store(off) => {
				let cell = cell_write!(tape, index, off);

				register = *cell;
				*cell = Wrapping(0u8);
			},

			IR::Scan(val, step) => loop {
				if *cell_read!(tape, index) == *val { break; }
				index += *step;
			},

			IR::Fill(off, val, step) => loop {
				if *cell_read!(tape, index) == Wrapping(0u8) {
					break;
				}

				let target = touch_on_tape(tape, &mut index, *off);
				*cell_write!(tape, target) = *val;

				index += *step;
			},

			IR::Input(off) => {
				let mut buffer = [0u8];
				if input.read_exact(&mut buffer).is_err() {
					continue;
				}

				let val = Wrapping(buffer[0]);
				*cell_write!(tape, index, off) = val;
			},

			IR::Output(off) => {
				let cell = *cell_read!(tape, index, off);
				output.write_all(&[cell.0])?;
			},

			IR::Loop(loop_prog) | IR::FixedLoop(loop_prog) => loop {
				if *cell_read!(tape, index) == Wrapping(0u8) {
					break;
				}

				index = eval(&loop_prog, input, output, tape, index)?;
			},
		}
	}

	io::Result::Ok(index)
}

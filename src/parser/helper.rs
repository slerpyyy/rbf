use std::iter::*;
use std::num::Wrapping;

use crate::internal::*;

#[inline]
pub fn set_inst (out_list : &mut Vec<IR>, sum : Wrapping<u8>, offset : isize) {
	let mut i = out_list.len() - 1;

	while let Some(IR::Set(off, _))
			| Some(IR::Add(off, _))
			| Some(IR::Mul(off, _)) = out_list.get_mut(i) {
		if *off == offset {
			out_list.remove(i);
		}

		i -= 1;
	}

	out_list.push(IR::Set(offset, sum))
}

#[inline]
pub fn add_inst (out_list : &mut Vec<IR>, sum : Wrapping<u8>, offset : isize) {
	if sum == Wrapping(0u8) {
		return;
	}

	for i in (0..out_list.len()).rev() {
		match out_list.get_mut(i) {
			Some(IR::Set(off, val)) => if *off == offset {
				*val += sum;
				return;
			},

			Some(IR::Add(off, val)) => if *off == offset {
				*val += sum;
				if *val == Wrapping(0u8) {
					out_list.remove(i);
				}
				return;
			},

			Some(IR::Touch(_,_)) => (),

			Some(IR::FixedLoop(_, high, low)) => {
				if offset <= *high && offset >= *low { break; }
			},

			Some(IR::Start) => {
				out_list.push(IR::Set(offset, sum));
				return;
			},

			_ => break,
		}
	}

	out_list.push(IR::Add(offset, sum))
}

#[inline]
pub fn move_inst (out_list : &mut Vec<IR>, offset : &mut isize) {
	if *offset != 0 {
		out_list.push(IR::Move(*offset));
		*offset = 0;
	}
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

	set_inst(out_list, Wrapping(0u8), *offset);
	true
}

#[inline]
fn flat_loop (
	out_list : &mut Vec<IR>,
	in_list : &[IR],
	offset : &mut isize
) -> bool {
	let mut sum = Wrapping(1u8);

	for x in in_list.iter().skip(1) {
		match x {
			IR::Add(0, val) => { sum += *val },
			IR::Add(_, _) => (),
			_ => return false,
		}
	}

	if sum != Wrapping(0u8) {
		return false;
	}

	let mut imm = None;

	for x in out_list.iter().rev() {
		match x {
			IR::Set(off, val) if *off == *offset => {
				imm = Some(*val);
				break;
			},

			IR::Add(off, _) if *off == *offset => break,
			IR::Add(_,_) | IR::Set(_,_) => (),
			_ => break,
		}
	}

	if imm.is_none() {
		out_list.push(IR::Store(*offset));
	}

	for x in in_list.iter() {
		match x {
			IR::Add(0, _) => (),
			IR::Add(off, val) => {
				let new_off = *offset + *off;

				if let Some(factor) = imm {
					add_inst(out_list, *val * factor, new_off);
				} else {
					out_list.push(IR::Mul(new_off, *val))
				}
			},
			_ => (),
		}
	}

	if imm.is_some() {
		set_inst(out_list, Wrapping(0u8), *offset)
	}

	true
}

#[inline]
fn scan_loop (
	out_list : &mut Vec<IR>,
	in_list : &[IR],
	offset : &mut isize
) -> bool {
	let mut start_cell = Wrapping(0u8);
	let mut end_cell = Wrapping(0u8);
	let mut set_step = false;
	let mut step = 0;

	for x in in_list.iter().skip(1) {
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

	if let Some(IR::Touch(high, low)) = in_list.first() {
		let tail : Vec<IR> = in_list.to_vec()
			.into_iter().skip(1).collect::<Vec<IR>>();

		out_list.push(IR::FixedLoop(tail, *high, *low));
		return true;
	}

	false
}

#[inline]
pub fn loop_inst (
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
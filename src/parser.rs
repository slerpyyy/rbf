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
fn add_inst (out_list : &mut Vec<IR>, sum : u16, offset : &i32) {
	if let Some(IR::Set(off, val)) = out_list.last_mut() {
		if *off == *offset {
			*val = (*val as u16 + sum) as u8;
			return;
		}
	}

	if let Some(IR::Add(off, val)) = out_list.last_mut() {
		if *off == *offset {
			*val = (*val as u16 + sum) as u8;
			if *val == 0 { out_list.pop(); }
			return;
		}
	}

	out_list.push(IR::Add(*offset, sum as u8));
}

#[inline]
fn clear_loop (
	out_list : &mut Vec<IR>,
	in_list : &Vec<IR>,
	offset : &mut i32
) -> bool {
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

	out_list.push(IR::Set(*offset, 0));
	true
}

#[inline]
fn flat_loop (out_list : &mut Vec<IR>, in_list : &Vec<IR>, offset : &mut i32) -> bool {
	let mut pos = 0;
	let mut sum = 1;

	for x in in_list.iter() {
		match *x {
			IR::Move(off) => pos += off,

			IR::Add(off, val) => if (off + pos) == 0 {
				sum += val as i32;
				sum &= 0xff;
			},

			_ => return false,
		}
	}

	if pos != 0 || sum != 0 {
		return false;
	}

	out_list.push(IR::Store(*offset));

	for x in in_list.iter() {
		if let IR::Add(off, val) = *x {
			if val == 255 && off == 0 {
				continue;
			}

			out_list.push(IR::Mul(*offset + off, val));
		}
	}

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
		if let IR::Add(off, val) = x {
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

	if let Some(IR::Set(off, val)) = in_list.first() {
		if let Some(IR::Move(step)) = in_list.last() {
			out_list.push(IR::Fill(*off, *val, *step));
			return true;
		}
	}

	false
}

pub fn parse(code : &Vec<u8>, mut index : usize) -> (Vec<IR>, usize) {
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
				let munch = munch_forward(&code, &mut index, '+', '-');
				let sum = (munch & 0xff) as u16;

				add_inst(&mut list, sum, &off_acc);
				continue;
			},

			'<' | '>' => {
				off_acc += munch_forward(&code, &mut index, '>', '<');
				continue;
			},

			'[' => {
				let (content, new_index) = parse(&code, index + 1);
				index = new_index;

				loop {
					if clear_loop(&mut list, &content, &mut off_acc) { break; }
					if flat_loop(&mut list, &content, &mut off_acc) { break; }

					if off_acc != 0 {
						list.push(IR::Move(off_acc));
						off_acc = 0;
					}

					if fill_loop(&mut list, &content) { break; }
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
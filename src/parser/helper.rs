use std::num::Wrapping;

use crate::internal::*;

#[inline]
pub fn set_inst(out_list: &mut Vec<IR>, sum: Wrapping<u8>, offset: isize) {
    for (ind, inst) in out_list.clone().iter().enumerate().rev() {
        match inst {
            IR::Set(off, _) |
            IR::Add(off, _) |
            IR::Mul(off, _) => {
                if *off == offset {
                    out_list.remove(ind);
                }
            }
            _ => break,
        }
    }

    out_list.push(IR::Set(offset, sum))
}

#[inline]
pub fn add_inst(out_list: &mut Vec<IR>, sum: Wrapping<u8>, offset: isize) {
    if sum == Wrapping(0u8) {
        return;
    }

    for (ind, inst) in out_list.iter_mut().enumerate().rev() {
        match inst {
            IR::Set(off, val) => if *off == offset {
                *val += sum;
                return;
            },

            IR::Add(off, val) => if *off == offset {
                *val += sum;
                if *val == Wrapping(0u8) {
                    out_list.remove(ind);
                }
                return;
            },

            IR::Mul(off, _) => if *off == offset {
                break;
            }

            IR::Store(off) => if *off == offset {
                out_list.push(IR::Set(offset, sum));
                return;
            }

            IR::Touch(_, _) => (),

            IR::FixedLoop(_, high, low) => {
                if offset >= *low && offset <= *high { break; }
            },

            IR::Start => {
                out_list.push(IR::Set(offset, sum));
                return;
            },

            _ => break,
        }
    }

    out_list.push(IR::Add(offset, sum))
}

#[inline]
pub fn move_inst(out_list: &mut Vec<IR>, offset: &mut isize) {
    if *offset != 0 {
        out_list.push(IR::Move(*offset));
        *offset = 0;
    }
}

#[inline]
fn clear_loop(out_list: &mut Vec<IR>, in_list: &[IR], offset: &mut isize) -> bool {
    if let [IR::Touch(_, _), IR::Add(0, Wrapping(val))] = in_list {
        if *val & 1 == 1 {
            set_inst(out_list, Wrapping(0u8), *offset);
            return true;
        }
    }

    false
}

#[inline]
fn flat_loop(out_list: &mut Vec<IR>, in_list: &[IR], offset: &mut isize) -> bool {
    let mut sum = Wrapping(1u8);

    for inst in in_list.iter().skip(1) {
        match inst {
            IR::Add(0, val) => { sum += *val },
            IR::Add(_, _) => (),
            _ => return false,
        }
    }

    if sum != Wrapping(0u8) {
        return false;
    }

    let mut imm = None;

    for inst in out_list.iter().rev() {
        match inst {
            IR::Set(off, val) if *off == *offset => {
                imm = Some(*val);
                break;
            },

            IR::Add(off, _) if *off == *offset => break,
            IR::Add(_, _) | IR::Set(_, _) => (),
            _ => break,
        }
    }

    if imm.is_none() {
        out_list.push(IR::Store(*offset));
    }

    for inst in in_list.iter() {
        match inst {
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
fn scan_loop(out_list: &mut Vec<IR>, in_list: &[IR], offset: &mut isize) -> bool {
    let mut start_cell = Wrapping(0u8);
    let mut end_cell = Wrapping(0u8);
    let mut set_step = false;
    let mut step = 0;

    for inst in in_list.iter().skip(1) {
        match inst {
            IR::Move(_) | IR::Add(_, _) => (),
            _ => return false,
        }
    }

    for inst in in_list.iter() {
        if let IR::Move(off) = inst {
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

    for inst in in_list.iter() {
        if let IR::Add(off, val) = inst {
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
fn fill_loop(out_list: &mut Vec<IR>, in_list: &[IR]) -> bool {
    if let [IR::Touch(_, _), IR::Set(off, val), IR::Move(step)] = in_list {
        out_list.push(IR::Fill(*off, *val, *step));
        out_list.push(IR::Touch(0, 0));
        return true;
    }

    false
}

#[inline]
fn fixed_loop(out_list: &mut Vec<IR>, in_list: &[IR]) -> bool {
    let mut pos = 0;

    for inst in in_list.iter() {
        match inst {
            IR::Move(off) => pos += *off,

            IR::Loop(_) | IR::Scan(_, _) | IR::Fill(_, _, _)
                => return false,

            _ => (),
        }
    }

    if pos != 0 {
        return false;
    }

    if let Some(IR::Touch(high, low)) = in_list.first() {
        let tail: Vec<IR> = in_list.to_vec()
            .into_iter().skip(1).collect::<Vec<IR>>();

        out_list.push(IR::FixedLoop(tail, *high, *low));
        return true;
    }

    false
}

#[inline]
pub fn loop_inst(out_list: &mut Vec<IR>, in_list: Vec<IR>, offset: &mut isize) {
    for inst in out_list.iter().rev() {
        if let IR::Set(off, _) | IR::Add(off, _) | IR::Mul(off, _)
             | IR::Store(off) | IR::Input(off) | IR::Output(off) = inst {
            if *off != *offset { continue; }
        }

        match inst {
            IR::Start => return,

            IR::Set(_, Wrapping(0)) => return,
            IR::Set(_, _) => break,
            IR::Store(_) => return,

            IR::FixedLoop(_, high, low) => {
                if *offset >= *low && *offset <= *high { break; }
            },

            IR::Touch(_, _) => (),

            _ => break,
        }
    }

    if clear_loop(out_list, &in_list, offset) { return; }
    if flat_loop(out_list, &in_list, offset) { return; }
    if scan_loop(out_list, &in_list, offset) { return; }

    move_inst(out_list, offset);

    if fill_loop(out_list, &in_list) { return; }
    if fixed_loop(out_list, &in_list) { return; }

    out_list.push(IR::Loop(in_list));
}

#[cfg(test)]
mod test {
    use std::num::Wrapping;
    use std::collections::HashSet;
    use crate::internal::IR;
    use super::*;

    #[test]
    fn set_inst_simple() {
        let mut out_list = Vec::new();
        set_inst(&mut out_list, Wrapping(2), 1);
        set_inst(&mut out_list, Wrapping(1), 2);
        set_inst(&mut out_list, Wrapping(3), 0);
        set_inst(&mut out_list, Wrapping(4), 1);
        set_inst(&mut out_list, Wrapping(5), 2);
        assert_eq!(out_list, vec![IR::Set(0, Wrapping(3)), IR::Set(1, Wrapping(4)), IR::Set(2, Wrapping(5))]);
    }

    #[test]
    fn clear_loop_wrapping() {
        for step in (0..=255u8).map(Wrapping) {
            let mut visited = HashSet::new();
            let mut current = Wrapping(0u8);

            let mut should_clear = true;
            for _ in 0..256 {
                if visited.contains(&current) {
                    should_clear = false;
                    break;
                }

                visited.insert(current);
                current += step;
            }

            let in_list = vec![IR::Touch(0, 0), IR::Add(0, step)];
            let mut out_list = Vec::new();
            let mut offset = 0;
            let does_clear = clear_loop(&mut out_list, &in_list, &mut offset);

            assert_eq!(should_clear, does_clear);
        }
    }
}

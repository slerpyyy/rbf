use std::cmp::{max, min};
use std::num::Wrapping;

use crate::internal::*;

mod helper;
use helper::*;

pub fn check_valid(bytes: &[u8]) -> bool {
    let mut sum = 0;

    for byte in bytes.iter() {
        match byte {
            b'[' => sum += 1,
            b']' => {
                sum -= 1;
                if sum < 0 { return false; }
            },
            _ => (),
        }
    }

    sum == 0
}

#[inline]
fn munch_forward(
    bytes: &[u8],
    index: &mut usize,
    inc: u8,
    dec: u8
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
fn set_touch_inst(inout_list: &mut Vec<IR>) {
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

            IR::Set(off, _) | IR::Add(off, _) | IR::Mul(off, _) |
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

pub fn parse_recursive(
    code: &[u8],
    mut index: &mut usize,
    root: bool
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

pub fn parse(code: &[u8]) -> Vec<IR> {
    let mut index = 0;
    parse_recursive(code, &mut index, true)
}

#[cfg(test)]
mod test {
    use crate::parser::*;

    #[test]
    fn check_valid_good() {
        let code = b"he+ll+o t+he--r.e";
        assert!(check_valid(code));

        let code = b"+++[>++<-]++.[-]--.";
        assert!(check_valid(code));

        let code = b"+[++[--->++[-]]]";
        assert!(check_valid(code));
    }

    #[test]
    fn check_valid_bad() {
        let code = b"aaaaa+a+a-a++]";
        assert!(!check_valid(code));

        let code = b"+++[>++<-]++.-]--.";
        assert!(!check_valid(code));
    }

    #[test]
    fn munch_forward_simple() {
        let text = b".....aababba...";
        let mut index = 5;

        assert_eq!(munch_forward(text, &mut index, b'a', b'b'), 1);
        assert_eq!(index, 12)
    }

    #[test]
    fn parse_simple() {
        let code = b"+++>>++>--,.";
        let prog = parse(code);
        let expected = vec![
            IR::Start,
            IR::Touch(3, 0),
            IR::Set(0, Wrapping(3)),
            IR::Set(2, Wrapping(2)),
            IR::Set(3, -Wrapping(2)),
            IR::Input(3),
            IR::Output(3),
        ];

        assert_eq!(prog, expected);
    }

    #[test]
    fn parse_flat_loop() {
        let code = b"+++[>++++<-]>.";
        let prog = parse(code);
        let expected = vec![
            IR::Start,
            IR::Touch(1, 0),
            IR::Set(1, Wrapping(12)),
            IR::Set(0, Wrapping(0)),
            IR::Output(1),
        ];

        assert_eq!(prog, expected);
    }
}

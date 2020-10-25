use std::io::{Read, Write, Error};
use std::cmp::max;
use std::num::Wrapping;
use std::collections::VecDeque;
use std::iter::repeat;

use crate::internal::*;

#[inline]
fn touch_range(
    tape : &mut VecDeque<Wrapping<u8>>,
    index : &mut isize,
    high : isize,
    low : isize
) {
    let upper = *index as isize + high + 1;
    let lower = *index as isize + low;

    let upper_diff = max(0, upper - tape.len() as isize) as usize;
    let lower_diff = max(0, -lower) as usize;
    let total_diff = upper_diff + lower_diff;

    if total_diff > 0 {
        tape.extend(repeat(Wrapping(0u8)).take(total_diff));
        tape.rotate_right(lower_diff);

        *index += lower_diff as isize;
    }
}

#[inline]
fn touch_cell(
    tape : &mut VecDeque<Wrapping<u8>>,
    index : &mut isize,
    offset : isize
) -> isize {
    let target = *index as isize + offset;

    if target < 0 {
        let n = -target as _;
        tape.extend(repeat(Wrapping(0u8)).take(n));
        tape.rotate_right(n);

        *index -= target;
        return 0;
    }

    let diff = target - tape.len() as isize;

    if diff >= 0 {
        let n = diff as _;
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


pub fn eval_recursive<R,W>(
    prog: &[IR],
    input: &mut R,
    output: &mut W,
    tape: &mut VecDeque<Wrapping<u8>>,
    mut index: isize,
    buffer: &mut [u8; 1]
) -> Result<isize, Error>
where R: Read, W: Write {
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
                if input.read_exact(buffer).is_err() {
                    buffer[0] = 0u8;
                }

                let val = Wrapping(buffer[0]);
                *cell!(write, tape, index + off) = val;
            },

            IR::Output(off) => {
                buffer[0] = cell!(read, tape, index + off).0;
                output.write_all(buffer)?;
            },

            IR::Loop(loop_prog) |
            IR::FixedLoop(loop_prog, _, _) => loop {
                if *cell!(read, tape, index) == Wrapping(0u8) {
                    break;
                }

                index = eval_recursive(&loop_prog, input, output, tape, index, buffer)?;
            },

            _ => (),
        }
    }

    Ok(index)
}

pub fn eval<R,W>(
    prog: &[IR],
    input: &mut R,
    output: &mut W,
    tape: &mut VecDeque<Wrapping<u8>>,
    index: isize
)
where R: Read, W: Write {
    let mut buffer = [0u8];
    eval_recursive(prog, input, output, tape, index, &mut buffer).unwrap();
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;
    use std::io::{empty, sink};

    use crate::evaluator::*;
    use crate::parser::*;

    #[test]
    fn eval_simple() {
        let code = b"+++>--<[>++++<-]++";
        let prog = parse(code);
        let mut tape = VecDeque::new();
        eval(&prog, &mut empty(), &mut sink(), &mut tape, 0);
        assert_eq!(tape, vec![Wrapping(2), Wrapping(10)]);
    }

    #[test]
    fn eval_hello() {
        let code = b"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.";
        let prog = parse(code);
        let mut tape = VecDeque::new();
        let mut output = Vec::new();
        eval(&prog, &mut empty(), &mut output, &mut tape, 0);
        assert_eq!(output, b"Hello, World!");
    }

    #[test]
    fn eval_cat() {
        let code = b",[+++.,]";
        let prog = parse(code);
        let mut tape = VecDeque::new();
        let mut input: &[u8] = b"abc";
        let mut output = Vec::new();
        eval(&prog, &mut input, &mut output, &mut tape, 0);
        assert_eq!(output, b"def");
    }
}

#![warn(clippy::all)]

use std::env::args;
use std::io::{Read, stdin, stdout};
use std::fs::File;
use std::num::Wrapping;
use std::collections::VecDeque;
use std::iter::repeat;
use getopts::Options;

mod internal;
mod parser;
mod evaluator;

use internal::*;
use parser::*;
use evaluator::*;

fn print_usage(program: &str, opts: &Options) {
    println!("Usage: {} FILE [options]\n", program);
    print!("A simple optimizing Brainfuck interpreter written in Rust.");
    println!("{}", opts.usage(""));
}

fn main() {
    let args: Vec<String> = args().collect();
    let program = args.get(0).unwrap();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help message");
    opts.optopt("c", "cmd", "run code from command", "CODE");
    opts.optopt("i", "input", "use string as input", "TEXT");
    opts.optflag("f", "force", "allow invalid code to run");
    opts.optflag("s", "static", "only print the generated code");

    let matches = opts.parse(&args[1..]).expect("failed to parse args");

    if matches.opt_present("h") {
        print_usage(program, &opts);
        return;
    }

    let mut code: Vec<u8> = Vec::new();

    if let Some(command) = matches.opt_str("c") {
        code = command.as_bytes().to_vec();
    } else if let Some(file_name) = matches.free.get(0) {
        if file_name == "-" {
            stdin()
                .read_to_end(&mut code)
                .expect("failed to read stdin");
        } else {
            File::open(file_name)
                .expect("failed to open file")
                .read_to_end(&mut code)
                .expect("failed to read file");
        }
    } else {
        print_usage(&program, &opts);
        return;
    };

    let prog = parse(&code);

    if matches.opt_present("s") {
        show_code(&prog, 120);
        return;
    }

    if !matches.opt_present("f") && !check_valid(&code) {
        panic!("invalid code");
    }

    let mut tape = VecDeque::with_capacity(0x2000);
    tape.extend(repeat(Wrapping(0u8)).take(0x1000));

    if let Some(input) = matches.opt_str("i") {
        eval(&prog, &mut input.as_bytes(), &mut stdout().lock(), &mut tape, 0x400);
    } else {
        eval(&prog, &mut stdin().lock(), &mut stdout().lock(), &mut tape, 0x400);
    }
}

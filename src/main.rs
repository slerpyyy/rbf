#![warn(clippy::all)]

use std::*;
use std::io;
use std::io::prelude::*;
use std::num::Wrapping;
use std::fs::File;
use std::collections::*;
use std::iter::*;
use string::*;
use stringreader::*;
use getopts::*;

mod internal;
mod parser;
mod evaluator;

use internal::*;
use parser::*;
use evaluator::*;

fn print_usage(program : &str, opts : &Options) {
	println!("Usage: {} FILE [options]\n", program);
	print!("A simple optimizing Brainfuck interpreter written in Rust.");
	println!("{}", opts.usage(""));
}

fn main() -> io::Result<()> {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();

	let mut opts = Options::new();
	opts.optflag("h", "help", "print this help message");
	opts.optopt("c", "cmd", "run code from command", "CODE");
	opts.optopt("i", "input", "use string as input", "TEXT");
	opts.optflag("f", "force", "allow invalid code to run");
	opts.optflagmulti("v", "verbose", "increase level of verbosity");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m },
		Err(fail) => {
			eprintln!("ERROR : {}", fail.to_string());
			process::exit(-1);
		},
	};

	if matches.opt_present("h") {
		print_usage(&program, &opts);
		return Ok(());
	}

	let verbosity = matches.opt_count("v");

	let mut code : Vec<u8> = Vec::new();

	if let Some(command) = matches.opt_str("c") {
		code = command.as_bytes().to_vec();
	} else if let Some(file_name) = matches.free.get(0) {
		if file_name == "-" {
			io::stdin().read_to_end(&mut code)?;
		} else {
			let file = File::open(file_name);

			if let Err(err) = file {
				eprintln!("ERROR : {}", err);
				process::exit(-1);
			}

			let mut raw_file = file?;
			raw_file.read_to_end(&mut code)?;
		}
	} else {
		print_usage(&program, &opts);
		return Ok(());
	};

	if !matches.opt_present("f") && !check_valid(&code) {
		eprintln!("ERROR : invalid code");
		process::exit(-1);
	}

	let stdin = std::io::stdin();
	let stdout = std::io::stdout();

	let mut input : &mut dyn Read = &mut stdin.lock();
	let output : &mut dyn Write = &mut stdout.lock();

	let in_string = matches.opt_str("i");
	let in_string_content = in_string.clone().unwrap_or_default();
	let mut str_input = StringReader::new(&in_string_content[..]);

	if in_string.is_some() {
		input = &mut str_input;
	}

	let mut index = 0usize;
	let prog = parse(&code, &mut index);
	if verbosity > 0 { show_code(&prog, 0); }

	let mut tape = VecDeque::with_capacity(0x10000);
	tape.extend(repeat(Wrapping(0u8)).take(0x1000));

	eval(&prog, input, output, &mut tape, 0x400)?;

	Ok(())
}

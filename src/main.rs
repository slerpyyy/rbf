use std::*;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::collections::*;
use clap::*;

mod internal;
mod parser;
mod evaluator;

use internal::*;
use parser::*;
use evaluator::*;

fn main() -> io::Result<()> {

	let matches = App::new("rbf")
		.version("0.1.0")
		.about("A simple optimizing Brainfuck interpreter written in Rust.")
		.arg(Arg::with_name("file")
			.value_name("FILE")
			.help("Sets the input file to use")
			.required(false)
			.index(1))
		.arg(Arg::with_name("command")
			.short("c")
			.long("cmd")
			.value_name("CODE")
			.help("Sets a custom config file")
			.takes_value(true))
		.arg(Arg::with_name("verbosity")
			.short("v")
			.multiple(true)
			.help("Sets the level of verbosity"))
		.get_matches();

	let command = matches.value_of("command");
	let filename = matches.value_of("file");
	let verbosity = matches.occurrences_of("verbosity");

	let mut buffer = Vec::new();

	if let Some(cmd_str) = command {
		buffer = cmd_str.as_bytes().to_vec();
	} else {
		if let Some(filename_str) = filename {
			if filename_str == "-" {
				io::stdin().read_to_end(&mut buffer)?;
			} else {
				let file = File::open(filename_str);

				if let Err(err) = file {
					eprintln!("ERROR : {}", err);
					std::process::exit(-1);
				}

				let mut raw_file = file?;
				raw_file.read_to_end(&mut buffer)?;
			}
		} else {
			eprintln!("ERROR : No input found");
			std::process::exit(-1);
		}
	}


	let (prog, _) = parse(&buffer, 0);

	if verbosity > 0 {
		show_code(&prog, 0);
	}

	let mut tape = VecDeque::with_capacity(0x10000);
	for _ in 0..0x100 { tape.push_back(0u8); }

	let input = std::io::stdin();
	let output = std::io::stdout();

	eval(&prog, &mut input.lock(), &mut output.lock(), &mut tape, 0)?;

	io::Result::Ok(())
}

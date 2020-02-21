use std::*;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::collections::*;
use string::*;
use stringreader::*;
use getopts::*;

mod internal;
mod parser;
mod evaluator;

use internal::*;
use parser::*;
use evaluator::*;

fn print_usage(program : &str, opts : Options) {
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
	opts.optopt("i", "input", "use string as input", "STRING");
	opts.optflagmulti("v", "verbose", "increase level of verbosity");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m },
		Err(f) => { panic!(f.to_string()) },
	};

	if matches.opt_present("h") {
		print_usage(&program, opts);
		return Ok(());
	}

	let verbosity = matches.opt_count("v");

	let mut code : Vec<u8> = Vec::new();

	if let Some(command) = matches.opt_str("c") {
		code = command.as_bytes().to_vec();
	} else {
		if let Some(file_name) = matches.free.get(0) {
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
			print_usage(&program, opts);
			return Ok(());
		}
	};

	let stdin = std::io::stdin();
	let stdout = std::io::stdout();

	let mut input : &mut dyn Read = &mut stdin.lock();
	let output : &mut dyn Write = &mut stdout.lock();

	let in_string_default = "".to_string();
	let in_string = matches.opt_str("i").unwrap_or(in_string_default.clone());
	let mut str_input = StringReader::new(&in_string[..]);

	if in_string != in_string_default {
		input = &mut str_input;
	}

	let (prog, _) = parse(&code, 0);
	if verbosity > 0 { show_code(&prog, 0); }

	let mut tape = VecDeque::with_capacity(0x10000);
	for _ in 0..0x100 { tape.push_back(0u8); }

	eval(&prog, input, output, &mut tape, 0)?;

	Ok(())
}

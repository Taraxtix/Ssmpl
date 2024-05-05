mod annotation;
mod compiler;
mod lexer;
mod parser;
mod report;
mod simulator;
mod type_checker;

use std::{env::args, path::Path, process::Command};

use lexer::Lexer;
use report::{Level, Reporter};

enum Mode {
	Com,
	Sim,
}

impl TryFrom<Option<String>> for Mode {
	type Error = String;

	fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
		if value.is_none() {
			return Err("No mode where specified".to_string());
		}
		match value.unwrap().as_str() {
			| "com" => Ok(Mode::Com),
			| "sim" => Ok(Mode::Sim),
			| _ => Err("Unknown mode".to_string()),
		}
	}
}

struct Cli<S: Into<String>> {
	program_path: S,
	input_path:   S,
	output_path:  S,
	mode:         Mode,
	debug:        bool,
	help:         bool,
	rounding:     bool,
	run:          bool,
}

fn usage(program_path: &String) -> String {
	format!("Usage: {} <mode> <input> [options]\n", program_path)
		+ "Modes:\n"
		+ "    com: Compile the program to elf64 asm.\n"
		+ "         If output is not specified, it will be a.out.\n"
		+ "    sim: Simulate the program.\n"
		+ "         Output is ignored if specified\n."
		+ "Options:\n"
		+ "    -o <path>: Specify the output path for the compilation (`com`) mode.\n"
		+ "               As no effect in simulation (`sim`) mode.\n"
		+ "    -d, --debug: Enable debug mode.\n"
		+ "                 Don't remove temporary files (.o and .asm)\n"
		+ "                 As no effect in simulation (`sim`) mode.\n"
		+ "    -h --help: Show this help message.\n"
		+ "    -r --run: Run the program after compilation.\n"
		+ "		   	     As no effect in simulation (`sim`) mode.\n"
		+ "    --rounding: Rounds double values for dumping.\n"
		+ "		   			   As no effect in simulation (`sim`) mode.\n"
}

fn retrieve_cli(reporter: &mut report::Reporter) -> Cli<String> {
	let mut args: Vec<String> = args().rev().collect();

	let program_path = args.pop().expect("Program path not found (Should never happen)");
	if args.is_empty() {
		println!("{}", usage(&program_path));
		std::process::exit(0);
	}
	let mode = args.pop().try_into().unwrap_or_else(|e| {
		reporter.add_error(e);
		Mode::Sim
	});
	let input_path = args.pop().map(|str| str.to_string()).unwrap_or("".to_string());

	let (output_path, debug, help, rounding, run) = retrieve_options(&mut args, reporter);
	Cli { program_path, input_path, mode, output_path, debug, help, rounding, run }
}

fn retrieve_options(
	args: &mut Vec<String>,
	reporter: &mut report::Reporter,
) -> (String, bool, bool, bool, bool) {
	let mut output_path = "a.out".to_string();
	let mut debug = false;
	let mut help = false;
	let mut rounding = false;
	let mut run = false;
	while !args.is_empty() {
		match args.pop().unwrap().as_str() {
			| "-o" => {
				output_path = args.pop().unwrap_or_else(|| {
					reporter.add_error(
						"-o option requires a path to be specified".to_string(),
					);
					"a.out".to_string()
				})
			},
			| "-d" | "--debug" => debug = true,
			| "-h" | "--help" => help = true,
			| "--rounding" => rounding = true,
			| "-r" | "--run" => run = true,
			| other => {
				reporter.add_error(format!("Unknown option: {}", other));
			},
		}
	}
	(output_path, debug, help, rounding, run)
}
fn main() {
	let mut reporter = Reporter::new(Level::Info);
	let cli = retrieve_cli(&mut reporter);

	if cli.help {
		println!("{}", usage(&cli.program_path));
		reporter.exit_if(Level::Error, 0);
	}

	if Path::new(&cli.output_path).file_name().is_none() {
		reporter.add_error(format!("Invalid output path: {}", cli.output_path));
	}

	let input = std::fs::read_to_string(&cli.input_path)
		.unwrap_or_else(|e| {
			reporter.add_error(format!("Failed to read input file: {}", e));
			"".to_string()
		})
		.chars()
		.collect::<Vec<_>>();

	if reporter.has(Level::Error) {
		eprintln!("{}", usage(&cli.program_path))
	}
	reporter.exit_if(Level::Error, 1);

	let mut program = Lexer::new(input.as_slice(), &cli.input_path, reporter.clone())
		.parse()
		.type_check();
	match cli.mode {
		| Mode::Com => {
			program.compile(&cli).unwrap_or_else(|e| {
				reporter.add_error(format!("Failed to compile: {}", e)).exit(1)
			});
			program.reporter.flush();
			if cli.run {
				Command::new(format!("./{}", &cli.output_path))
					.spawn()
					.unwrap()
					.wait()
					.unwrap();
			}
		},
		| Mode::Sim => {
			program.reporter.flush().exit_if(Level::Error, 1);
			program.simulate()
		},
	}
}

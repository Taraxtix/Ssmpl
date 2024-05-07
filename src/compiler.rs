//#region Imports
use std::{
	collections::HashMap,
	fs::OpenOptions,
	io::{self, BufWriter, Error, Write},
	path::Path,
	process::Command,
};

use crate::{
	annotation::Type,
	parser::{Op, Program},
	Cli,
};
//#endregion

//#region ASM
const ASM_HEADER: &str = r#"extern dump_i
extern dump_f
extern dump_f_rounded
extern i64tof64

global write
write:
    mov 	rax, 1
    syscall
    ret

dump_b:
	pop 	rbp
    mov 	rdi, 1
	mov 	rdx, 5
	mov 	rsi, true_str
	pop 	rax
	test	rax, rax
	mov 	rax, false_str
	cmovz	rsi, rax
	call	write
	mov 	rsi, new_line
	mov 	rdx, 1
	call 	write
	push 	rbp
	ret

test_xmm0:
	add 	rsp, 8
	xor 	r15, r15
	movq 	rax, xmm0
	test 	rax, rax
	mov 	rax, 1
	cmove 	rax, r15
	mov 	qword[rsp], rax
	sub 	rsp, 8
	ret

global _start
_start:
	pop		rax
	mov		qword[argc], rax
	pop		rax
	mov		qword[argv], rax

"#;

const ASM_EXIT_DATA: &str = "
    mov     rax, 60             ;Syscall code for exit
    mov     rdi, 0              ;Param: exit code
    syscall                     ;Calling exit syscall

section .data
argc: dq 0
argv: dq 0
true_str: db 'true', 0
false_str: db 'false', 0
new_line: db 10
";
//#endregion

const SYSCALL_REGS: [&str; 6] = ["rdi", "rsi", "rdx", "r10", "r8", "r9"];

fn escape_string(lit: &str) -> String {
	lit.replace('\n', "\\n")
		.replace('\t', "\\t")
		.replace('\r', "\\r")
		.replace('\0', "\\0")
		.replace('`', "\\`")
}

impl Program<'_> {
	pub fn compile(&mut self, cli: &Cli<String>) -> Result<(), io::Error> {
		let output_path = Path::new(cli.output_path.as_str());
		let dir = output_path.parent().unwrap_or(Path::new("/"));
		let output_path_str = output_path.file_name().unwrap().to_str().unwrap();
		let base_path = output_path_str
			.split('.')
			.next()
			.expect("Split should have returned at least 0 element");
		let asm_file_path = dir.join(format!("{}.asm", base_path));
		let aux_o_file_path = &dir.join("aux.o");
		let o_file_path = dir.join(format!("{}.o", base_path));

		self.add_info(format!(
			"Copying auxiliary data in {} ...",
			aux_o_file_path.display()
		));
		self.write_aux_o(aux_o_file_path)?;
		self.add_info(format!(
			"Compiling program to asm in {} ...",
			&asm_file_path.display()
		));
		self.write_asm(&asm_file_path, cli)?;

		if !self.command(format!("nasm -felf64 {}", asm_file_path.display())) {
			self.add_error("Failed to compile assembly with `nasm`".to_string()).exit(1);
		};

		if !self.command(format!(
			"ld {} {} -o {}",
			o_file_path.display(),
			aux_o_file_path.display(),
			output_path.display()
		)) {
			self.add_error("Failed to link object with `ld`".to_string()).exit(1);
		}

		if !cli.debug
			&& !self.command(format!(
				"rm {} {} {}",
				aux_o_file_path.display(),
				asm_file_path.display(),
				o_file_path.display()
			)) {
			self.add_error("Failed remove temporary files with `rm`".to_string()).exit(1);
		}

		Ok(())
	}

	fn command(&mut self, cmd: String) -> bool {
		self.add_info(format!("Running `{}` ...", cmd.clone()));
		let args_vec = cmd.split(' ').collect::<Vec<_>>();
		let args = args_vec.as_slice();
		let name = args[0];
		let mut command = Command::new(name);
		for args in args[1..].iter() {
			command.arg(args);
		}
		command
			.spawn()
			.unwrap_or_else(|err| self.fail_spawn("ld", err))
			.wait()
			.unwrap_or_else(|err| self.fail_run("ld", err))
			.success()
	}

	fn fail_spawn(&mut self, name: &str, err: Error) -> ! {
		self.add_error(format!("Failed to spawn {} child process: {}", name, err)).exit(1)
	}

	fn fail_run(&mut self, name: &str, err: Error) -> ! {
		self.add_error(format!("Failed to run {}: {}", name, err)).exit(1)
	}

	fn write_asm(
		&mut self,
		asm_file_path: &Path,
		cli: &Cli<String>,
	) -> Result<(), std::io::Error> {
		let mut buf = BufWriter::new(
			OpenOptions::new()
				.write(true)
				.create(true)
				.truncate(true)
				.open(asm_file_path)
				.unwrap_or_else(|err| {
					self.add_error(format!(
						"Failed to open {} for writing: {}",
						asm_file_path.display(),
						err
					))
					.exit(1)
				}),
		);
		buf.write_all(ASM_HEADER.as_bytes())?;
		let mut labels = HashMap::<String, i64>::new();
		for op in self.ops.iter() {
			buf.write_all(op.to_asm(cli, &mut labels, &self.strings).as_bytes())?;
		}
		buf.write_all(ASM_EXIT_DATA.as_bytes())?;
		for (idx, lit) in self.strings.iter().enumerate() {
			buf.write_all(
				format!("STR_LIT_{}: db `{}`, 0\n", idx, escape_string(lit)).as_bytes(),
			)?;
		}
		Ok(())
	}

	fn write_aux_o(&mut self, aux_o_file_path: &Path) -> Result<(), std::io::Error> {
		let mut buf = BufWriter::new(
			OpenOptions::new()
				.write(true)
				.create(true)
				.truncate(true)
				.open(aux_o_file_path)
				.unwrap_or_else(|err| {
					self.add_error(format!(
						"Failed to open {} for writing: {}",
						aux_o_file_path.display(),
						err
					))
					.exit(1)
				}),
		);
		let bytes = include_bytes!("./resources/aux.o");
		buf.write_all(bytes)
	}
}

impl Op<'_> {
	fn to_asm(
		&self,
		cli: &Cli<String>,
		labels: &mut HashMap<String, i64>,
		strings: &[String],
	) -> String {
		match self {
			| Op::PushI(i, _) => format!(";PUSH {}\n\tpush\t{}\n", i, i),
			| Op::PushF(f, _) => {
				format!(";PUSH {}\n\tmov \trax, __float64__({:?})\n\tpush\trax\n", f, f)
			}
			| Op::PushB(b, _) => {
				format!(";PUSH {}\n\tpush\t{}\n\tpush\trax\n", b, *b as u8)
			}
			| Op::PushStr(s, _) => {
				let idx = strings
					.iter()
					.enumerate()
					.find_map(|(idx, lit)| if s == lit { Some(idx) } else { None })
					.unwrap();
				format!(";PUSH {s}\n\tmov \trax, STR_LIT_{idx}\n\tpush\trax\n")
			}
			| Op::Dump(a) => {
				match a.get_type().unwrap() {
					| Type::F64 => {
						";DUMP_F\n\tpop \trdi\n\tmovq\txmm0, rdi\n\tcall\t".to_string()
							+ if cli.rounding { "dump_f_rounded\n" } else { "dump_f\n" }
					}
					| Type::I64 | Type::Ptr => {
						";DUMP_I\n\tpop \trdi\n\tcall\tdump_i\n".to_string()
					}
					| Type::Bool => ";DUMP_B\n\tcall\tdump_b\n".to_string(),
				}
			}
			| Op::Add(a, b) => {
				";ADD\n".to_string()
					+ match (a.get_type().unwrap(), b.get_type().unwrap()) {
						| (Type::I64, Type::F64) => {
							"\tpop \trdi\n\tcall\ti64tof64\n\tmovq\txmm1, \
							 [rsp]\n\taddsd\txmm1, xmm0\n\tmovq\t[rsp], xmm1\n"
						}
						| (Type::F64, Type::I64) => {
							"\tmov \trdi, qword[rsp+8]\n\tcall\ti64tof64\n\taddsd\txmm0, \
							 [rsp]\n\tmovq\t[rsp+8], xmm0\n\tadd \trsp, 8\n"
						}
						| (Type::F64, Type::F64) => {
							"\tmovq\txmm0, [rsp+8]\n\taddsd\txmm0, \
							 [rsp]\n\tmovq\t[rsp+8], xmm0\n\tadd \trsp, 8\n"
						}
						| (..) => "\tpop \trdi\n\tadd \t[rsp], rdi\n",
					}
			}
			| Op::Sub(a, b) => {
				";SUB\n".to_string()
					+ match (a.get_type().unwrap(), b.get_type().unwrap()) {
						| (Type::I64, Type::F64) => {
							"\tpop \trdi\n\tcall\ti64tof64\n\tmovq\txmm1, \
							 [rsp]\n\tsubsd\txmm1, xmm0\n\tmovq\t[rsp], xmm1\n"
						}
						| (Type::F64, Type::I64) => {
							"\tmov \trdi, qword[rsp+8]\n\tcall\ti64tof64\n\tsubsd\txmm0, \
							 [rsp]\n\tmovq\t[rsp+8], xmm0\n\tadd \trsp, 8\n"
						}
						| (Type::F64, Type::F64) => {
							"\tmovq\txmm0, [rsp+8]\n\tsubsd\txmm0, \
							 [rsp]\n\tmovq\t[rsp+8], xmm0\n\tadd \trsp, 8\n"
						}
						| (..) => "\tpop \trdi\n\tsub \t[rsp], rdi\n",
					}
			}
			| Op::Mul(a, b) => {
				";MUL\n".to_string()
					+ match (a.get_type().unwrap(), b.get_type().unwrap()) {
						| (Type::I64, Type::F64) => {
							"\tpop \trdi\n\tcall\ti64tof64\n\tmovq\txmm1, \
							 [rsp]\n\tmulsd\txmm1, xmm0\n\tmovq\t[rsp], xmm1\n"
						}
						| (Type::F64, Type::I64) => {
							"\tmov \trdi, qword[rsp+8]\n\tcall\ti64tof64\n\tmulsd\txmm0, \
							 [rsp]\n\tmovq\t[rsp+8], xmm0\n\tadd \trsp, 8\n"
						}
						| (Type::F64, Type::F64) => {
							"\tmovq\txmm0, [rsp+8]\n\tmulsd\txmm0, \
							 [rsp]\n\tmovq\t[rsp+8], xmm0\n\tadd \trsp, 8\n"
						}
						| (..) => {
							"\tpop \trdi\n\tpop \trax\n\timul \trax, rdi\n\tpush\t rax\n"
						}
					}
			}
			| Op::Div(a, b) => {
				";ADD\n".to_string()
					+ match (a.get_type().unwrap(), b.get_type().unwrap()) {
						| (Type::I64, Type::F64) => {
							"\tpop \trdi\n\tcall\ti64tof64\n\tmovq\txmm1, \
							 [rsp]\n\tdivsd\txmm1, xmm0\n\tmovq\t[rsp], xmm1\n"
						}
						| (Type::F64, Type::I64) => {
							"\tmov \trdi, qword[rsp+8]\n\tcall\ti64tof64\n\tdivsd\txmm0, \
							 [rsp]\n\tmovq\t[rsp+8], xmm0\n\tadd \trsp, 8\n"
						}
						| (Type::F64, Type::F64) => {
							"\tmovq\txmm0, [rsp+8]\n\tdivsd\txmm0, \
							 [rsp]\n\tmovq\t[rsp+8], xmm0\n\tadd \trsp, 8\n"
						}
						| (..) => {
							"\tpop \trdi\n\tpop \trax\n\tcqo\n\tidiv \trdi\n\tpush\trax\n"
						}
					}
			}
			| Op::Mod(..) => {
				";MOD\n\tpop \trdi\n\tpop \trax\n\tcqo\n\tidiv \trdi\n\tpush\trdx\n"
					.into()
			}
			| Op::Increment(a) => {
				";INC\n".to_string()
					+ match a.get_type().unwrap() {
						| Type::F64 => {
							"\tmov\trax, __float64__(1.0)\n\tmovq\txmm0, \
							 rax\n\taddsd\txmm0, [rsp]\n\tmovq\t[rsp], xmm0\n"
						}
						| _ => "\tinc qword[rsp]\n",
					}
			}
			| Op::Decrement(a) => {
				";DEC\n".to_string()
					+ match a.get_type().unwrap() {
						| Type::F64 => {
							"\tmov\trax, __float64__(1.0)\n\tmovq\txmm0, \
							 rax\n\tsubsd\txmm0, [rsp]\n\tmovq\t[rsp], xmm0\n"
						}
						| _ => "\tdec qword[rsp]\n",
					}
			}
			| Op::Drop(n, _) => format!(";DROP{n}\n\tadd \trsp, {}\n", n * 8),
			| Op::Swap(_) => {
				";SWAP\n\tpop \trax\n\tpop \trbx\n\tpush\trax\n\tpush \trbx\n".to_string()
			}
			| Op::Over(n, _) => format!(";OVER{n}\n\tpush\tqword[rsp+{}]\n", 8 * n),
			| Op::Dup(n, _) => {
				if *n < 6 {
					(0..*n).fold(format!(";DUP{n}"), |acc, _| {
						acc + format!(";DUP\n\tpush\tqword[rsp+{}]\n", 8 * (n - 1))
							.as_str()
					})
				} else {
					labels
						.insert("DUMP_L".into(), labels.get("DUMP_L").unwrap_or(&-1) + 1);
					format!(
						";DUP\n\tmov \trcx, \
						 {n}\n\tDUMP_L{}:\n\tpush\tqword[rsp+{}]\n\tdec \trcx\n\tjnz \
						 \tDUMP_L{}\n\ttest \trcx, rcx\n",
						labels.get("DUMP_L").unwrap(),
						8 * (n - 1),
						labels.get("DUMP_L").unwrap()
					)
				}
			}
			| Op::If(..) => ";IF\n".into(),
			| Op::Then(label, else_, _) => {
				format!(
					";THEN\n\tpop \trax\n\ttest\trax, rax\n\tjz \t{}\n",
					if *else_ { format!("ELSE_{label}") } else { format!("END_{label}") }
				)
			}
			| Op::Else(label, _) => {
				format!(";ELSE\n\tjmp \tEND_{label}\nELSE_{label}:\n")
			}
			| Op::End(label, while_, _) => {
				format!(
					";END\n{}END_{}{label}:\n",
					if *while_ { format!("\tjmp \tWHILE_{label}\n") } else { "".into() },
					if *while_ { "WHILE_" } else { "" }
				)
			}
			| Op::While(label, _) => format!("WHILE_{label}:\n"),
			| Op::Do(label, _) => {
				format!(";DO\n\tpop \trax\n\ttest\trax, rax\n\tjz \tEND_WHILE_{label}\n")
			}
			| Op::Eq(annot_l, annot_r) => {
				";Eq\n\t".to_string()
					+ match (annot_l.get_type().unwrap(), annot_r.get_type().unwrap()) {
						| (Type::F64, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmovq\txmm0, \
							 qword[rsp]\n\tcmppd\txmm0, xmm1, 0\n\tcall\ttest_xmm0\n"
						}
						| (_, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmov \trdi, \
							 qword[rsp]\n\tcall\ti64tof64\n\tcmppd\txmm0, xmm1, \
							 0\n\tcall\ttest_xmm0\n"
						}
						| (Type::F64, _) => {
							"pop \trdi\n\tcall\ti64tof64\n\tmovq\txmm1, \
							 xmm0\n\tmovq\txmm0, qword[rsp]\n\tcmppd\txmm0, xmm1, \
							 0\n\tcall\ttest_xmm0\n"
						}
						| (..) => {
							"pop \trbx\n\tmov \trax, qword[rsp]\n\tcmp \trax, rbx\n\tmov \
							 \tqword[rsp], 0\n\tsete\t[rsp]\n"
						}
					}
			}
			| Op::Neq(annot_l, annot_r) => {
				";Neq\n\t".to_string()
					+ match (annot_l.get_type().unwrap(), annot_r.get_type().unwrap()) {
						| (Type::F64, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmovq\txmm0, \
							 qword[rsp]\n\tcmppd\txmm0, xmm1, 4\n\tcall\ttest_xmm0\n"
						}
						| (_, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmov \trdi, \
							 qword[rsp]\n\tcall\ti64tof64\n\tcmppd\txmm0, xmm1, \
							 0\n\tcall\ttest_xmm0\n"
						}
						| (Type::F64, _) => {
							"pop \trdi\n\tcall\ti64tof64\n\tmovq\txmm1, \
							 xmm0\n\tmovq\txmm0, qword[rsp]\n\tcmppd\txmm0, xmm1, \
							 4\n\tcall\ttest_xmm0\n"
						}
						| (..) => {
							"pop \trbx\n\tmov \trax, qword[rsp]\n\tcmp \trax, rbx\n\tmov \
							 \tqword[rsp], 0\n\tsetne\t[rsp]\n"
						}
					}
			}
			| Op::Lt(annot_l, annot_r) => {
				";Lt\n\t".to_string()
					+ match (annot_l.get_type().unwrap(), annot_r.get_type().unwrap()) {
						| (Type::F64, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmovq\txmm0, \
							 qword[rsp]\n\tcmppd\txmm0, xmm1, 1\n\tcall\ttest_xmm0\n"
						}
						| (_, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmov \trdi, \
							 qword[rsp]\n\tcall\ti64tof64\n\tcmppd\txmm0, xmm1, \
							 0\n\tcall\ttest_xmm0\n"
						}
						| (Type::F64, _) => {
							"pop \trdi\n\tcall\ti64tof64\n\tmovq\txmm1, \
							 xmm0\n\tmovq\txmm0, qword[rsp]\n\tcmppd\txmm0, xmm1, \
							 1\n\tcall\ttest_xmm0\n"
						}
						| (..) => {
							"pop \trbx\n\tmov \trax, qword[rsp]\n\tcmp \trax, rbx\n\tmov \
							 \tqword[rsp], 0\n\tsetl\t[rsp]\n"
						}
					}
			}
			| Op::Gt(annot_l, annot_r) => {
				";Gt\n\t".to_string()
					+ match (annot_l.get_type().unwrap(), annot_r.get_type().unwrap()) {
						| (Type::F64, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmovq\txmm0, \
							 qword[rsp]\n\tcmppd\txmm0, xmm1, 0Eh\n\tcall\ttest_xmm0\n"
						}
						| (_, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmov \trdi, \
							 qword[rsp]\n\tcall\ti64tof64\n\tcmppd\txmm0, xmm1, \
							 0\n\tcall\ttest_xmm0\n"
						}
						| (Type::F64, _) => {
							"pop \trdi\n\tcall\ti64tof64\n\tmovq\txmm1, \
							 xmm0\n\tmovq\txmm0, qword[rsp]\n\tcmppd\txmm0, xmm1, \
							 0Eh\n\tcall\ttest_xmm0\n"
						}
						| (..) => {
							"pop \trbx\n\tmov \trax, qword[rsp]\n\tcmp \trax, rbx\n\tmov \
							 \tqword[rsp], 0\n\tsetg\t[rsp]\n"
						}
					}
			}
			| Op::Lte(annot_l, annot_r) => {
				";Lte\n\t".to_string()
					+ match (annot_l.get_type().unwrap(), annot_r.get_type().unwrap()) {
						| (Type::F64, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmovq\txmm0, \
							 qword[rsp]\n\tcmppd\txmm0, xmm1, 2\n\tcall\ttest_xmm0\n"
						}
						| (_, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmov \trdi, \
							 qword[rsp]\n\tcall\ti64tof64\n\tcmppd\txmm0, xmm1, \
							 0\n\tcall\ttest_xmm0\n"
						}
						| (Type::F64, _) => {
							"pop \trdi\n\tcall\ti64tof64\n\tmovq\txmm1, \
							 xmm0\n\tmovq\txmm0, qword[rsp]\n\tcmppd\txmm0, xmm1, \
							 2\n\tcall\ttest_xmm0\n"
						}
						| (..) => {
							"pop \trbx\n\tmov \trax, qword[rsp]\n\tcmp \trax, rbx\n\tmov \
							 \tqword[rsp], 0\n\tsetle\t[rsp]\n"
						}
					}
			}
			| Op::Gte(annot_l, annot_r) => {
				";Gte\n\t".to_string()
					+ match (annot_l.get_type().unwrap(), annot_r.get_type().unwrap()) {
						| (Type::F64, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmovq\txmm0, \
							 qword[rsp]\n\tcmppd\txmm0, xmm1, 0Dh\n\tcall\ttest_xmm0\n"
						}
						| (_, Type::F64) => {
							"movq\txmm1, qword[rsp]\n\tadd \trsp, 8\n\tmov \trdi, \
							 qword[rsp]\n\tcall\ti64tof64\n\tcmppd\txmm0, xmm1, \
							 0\n\tcall\ttest_xmm0\n"
						}
						| (Type::F64, _) => {
							"pop \trdi\n\tcall\ti64tof64\n\tmovq\txmm1, \
							 xmm0\n\tmovq\txmm0, qword[rsp]\n\tcmppd\txmm0, xmm1, \
							 0Dh\n\tcall\ttest_xmm0\n"
						}
						| (..) => {
							"pop \trbx\n\tmov \trax, qword[rsp]\n\tcmp \trax, rbx\n\tmov \
							 \tqword[rsp], 0\n\tsetge\t[rsp]\n"
						}
					}
			}
			| Op::Syscall(syscode, argc, _) => {
				(0..*argc).rev().fold(";Syscall\n\t".to_string(), |acc, idx| {
					format!("{acc}pop \t{}\n\t", SYSCALL_REGS[idx])
				}) + format!("mov \trax, {syscode}\n\tsyscall\n\tpush\trax\n").as_str()
			}
			| Op::Argc(_) => ";Argc\n\tpush\tqword[argc]\n".into(),
			| Op::Argv(_) => ";Argv\n\tpush\tqword[argv]\n".into(),
			| Op::Deref(_) => {
				";Deref\n\tmov \trax, qword[rsp]\n\tmov\tqword[rsp], qword[rax]\n".into()
			}
			| Op::Nop => unreachable!(),
		}
	}
}

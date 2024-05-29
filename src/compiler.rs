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
	parser::{Op, OpType, Program},
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
	mov 	rdx, 6
	mov 	rsi, true_str
	pop 	rax
	test	rax, rax
	mov 	rax, false_str
	cmovz	rsi, rax
	call	write
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
	mov		qword[argv], rsp

"#;

const ASM_EXIT_DATA: &str = "
    mov     rax, 60             ;Syscall code for exit
    mov     rdi, 0              ;Param: exit code
    syscall                     ;Calling exit syscall

section .data
argc: dq 0
argv: dq 0
true_str: db 'true', 10, 0
false_str: db 'false', 10, 0
";

const ASM_BSS: &str = "
section .bss
MEM_BUILTIN_FREE_: resb 1024
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

impl Program {
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
			"ld {} {} -o {} --no-warn-execstack",
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
		buf.write_all(ASM_BSS.as_bytes())?;
		for name in self.memory_regions_order.iter() {
			let size = self.memory_regions.get(name).unwrap();
			buf.write_all(format!("MEM_{name}: resb {size}\n").as_bytes())?;
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

impl Op {
	fn to_asm(
		&self,
		cli: &Cli<String>,
		labels: &mut HashMap<String, i64>,
		strings: &[String],
	) -> String {
		use OpType::*;
		match self.typ.clone() {
			| PushI(i) => format!(";PUSH {}\n\tpush\t{}\n", i, i),
			| PushF(f) => {
				format!(";PUSH {}\n\tmov \trax, __float64__({:?})\n\tpush\trax\n", f, f)
			}
			| PushB(b) => {
				format!(";PUSH {}\n\tpush\t{}\n\tpush\trax\n", b, b as u8)
			}
			| PushStr(s) => {
				let idx = strings
					.iter()
					.enumerate()
					.find_map(|(idx, lit)| if s == *lit { Some(idx) } else { None })
					.unwrap();
				format!(";PUSH {s}\n\tmov \trax, STR_LIT_{idx}\n\tpush\trax\n")
			}
			| Dump(typ) => {
				match typ {
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
			| Add(a, b) => {
				";ADD\n".to_string()
					+ match (a, b) {
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
			| Sub(a, b) => {
				";SUB\n".to_string()
					+ match (a, b) {
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
			| Mul(a, b) => {
				";MUL\n".to_string()
					+ match (a, b) {
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
			| Div(a, b) => {
				";ADD\n".to_string()
					+ match (a, b) {
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
			| Mod(..) => {
				";MOD\n\tpop \trdi\n\tpop \trax\n\tcqo\n\tidiv \trdi\n\tpush\trdx\n"
					.into()
			}
			| Increment(typ) => {
				";INC\n".to_string()
					+ match typ {
						| Type::F64 => {
							"\tmov\trax, __float64__(1.0)\n\tmovq\txmm0, \
							 rax\n\taddsd\txmm0, [rsp]\n\tmovq\t[rsp], xmm0\n"
						}
						| _ => "\tinc qword[rsp]\n",
					}
			}
			| Decrement(typ) => {
				";DEC\n".to_string()
					+ match typ {
						| Type::F64 => {
							"\tmov\trax, __float64__(1.0)\n\tmovq\txmm0, \
							 rax\n\tsubsd\txmm0, [rsp]\n\tmovq\t[rsp], xmm0\n"
						}
						| _ => "\tdec qword[rsp]\n",
					}
			}
			| Drop(n) => format!(";DROP{n}\n\tadd \trsp, {}\n", n * 8),
			| Swap => {
				";SWAP\n\tpop \trax\n\tpop \trbx\n\tpush\trax\n\tpush \trbx\n".to_string()
			}
			| Over(n) => format!(";OVER{n}\n\tpush\tqword[rsp+{}]\n", 8 * n),
			| Dup(n) => {
				if n < 6 {
					(0..n).fold(format!(";DUP{n}"), |acc, _| {
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
			| If(..) => ";IF\n".into(),
			| Then(label, else_) => {
				format!(
					";THEN\n\tpop \trax\n\ttest\trax, rax\n\tjz \t{}\n",
					if else_ { format!("ELSE_{label}") } else { format!("END_{label}") }
				)
			}
			| Else(label) => format!(";ELSE\n\tjmp \tEND_{label}\nELSE_{label}:\n"),
			| End(label, while_) => {
				format!(
					";END\n{}END_{}{label}:\n",
					if while_ { format!("\tjmp \tWHILE_{label}\n") } else { "".into() },
					if while_ { "WHILE_" } else { "" }
				)
			}
			| While(label) => format!("WHILE_{label}:\n"),
			| Do(label) => {
				format!(";DO\n\tpop \trax\n\ttest\trax, rax\n\tjz \tEND_WHILE_{label}\n")
			}
			| Eq(type_l, type_r) => {
				";Eq\n\t".to_string()
					+ match (type_l, type_r) {
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
			| Neq(type_l, type_r) => {
				";Neq\n\t".to_string()
					+ match (type_l, type_r) {
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
			| Lt(type_l, type_r) => {
				";Lt\n\t".to_string()
					+ match (type_l, type_r) {
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
			| Gt(type_l, type_r) => {
				";Gt\n\t".to_string()
					+ match (type_l, type_r) {
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
			| Lte(type_l, type_r) => {
				";Lte\n\t".to_string()
					+ match (type_l, type_r) {
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
			| Gte(type_l, type_r) => {
				";Gte\n\t".to_string()
					+ match (type_l, type_r) {
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
			| Syscall(syscode, argc) => {
				(0..argc).rev().fold(";Syscall\n\t".to_string(), |acc, idx| {
					format!("{acc}pop \t{}\n\t", SYSCALL_REGS[idx])
				}) + format!("mov \trax, {syscode}\n\tsyscall\n\tpush\trax\n").as_str()
			}
			| Argc => ";Argc\n\tpush\tqword[argc]\n".into(),
			| Argv => ";Argv\n\tpush\tqword[argv]\n".into(),
			| Load8 => {
				";Load8\n\tpop \trax\n\tmovzx\trax, byte[rax]\n\tpush\trax\n".into()
			}
			| Load16 => {
				";Load16\n\tpop \trax\n\tmovzx\trax, word[rax]\n\tpush\trax\n".into()
			}
			| Load32 => {
				";Load32\n\tpop \trax\n\tmovzx\trax, dword[rax]\n\tpush\trax\n".into()
			}
			| Load64 => ";Load64\n\tpop \trax\n\tpush\tqword[rax]\n".into(),
			| Store8 => {
				";Store8\n\tpop \trbx\n\tpop \trax\n\tmov\tbyte[rax], bl\n".into()
			}
			| Store16 => {
				";Store16\n\tpop \trbx\n\tpop \trax\n\tmov\tword[rax], bx\n".into()
			}
			| Store32 => {
				";Store32\n\tpop \trbx\n\tpop \trax\n\tmov\tdword[rax], ebx\n".into()
			}
			| Store64 => {
				";Store64\n\tpop \trbx\n\tpop \trax\n\tmov\tqword[rax], rbx\n".into()
			}
			| Cast(Type::Bool) => {
				";Cast(Bool)\n\tcmp \tqword[rsp], 0\n\tsetne\t[rsp]\n".into()
			}
			| Cast(_) => "".into(),
			| ShiftR => ";ShiftR\n\tpop \trcx\n\tshr \tqword[rsp], cl\n".into(),
			| ShiftL => ";ShiftL\n\tpop \trcx\n\tshl \tqword[rsp], cl\n".into(),
			| BitAnd => ";BitAnd\n\tpop \trax\n\tand \tqword[rsp], rax\n".into(),
			| BitOr => ";BitOr\n\tpop \trax\n\tor \tqword[rsp], rax\n".into(),
			| And => {
				";And\n\tpop \trax\n\tand \tqword[rsp], rax\n\tmov \trax, 1\n\txor \
				 \tr15, r15\n\tcmp \tqword[rsp], 0\n\tcmove\trax, r15\n\tmov \
				 \tqword[rsp], rax\n"
					.into()
			}
			| Or => {
				";Or\n\tpop \trax\n\tor  \tqword[rsp], rax\n\tmov \trax, 1\n\txor \tr15, \
				 r15\n\tcmp \tqword[rsp], 0\n\tcmove\trax, r15\n\tmov \tqword[rsp], rax\n"
					.into()
			}
			| Not => ";Not\n\tnot \tqword[rsp]\n".into(),
			| Mem(name) => {
				match name {
					| Some(name) => format!("push MEM_{name}\n"),
					| None => "push MEM_BUILTIN_FREE_\n".into(),
				}
			}
			| Nop => unreachable!(),
		}
	}
}

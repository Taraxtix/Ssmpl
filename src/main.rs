
use std::{fs::{OpenOptions, self}, process::{exit, Command}, env::args, io::Write};

use lexer::op::Op;

use crate::lexer::Lexer;

pub mod lexer;

const ASM_HEADER: &'static str =r#"
global _start

dump:
    sub     rsp, 40
    xorps   xmm0, xmm0
    mov     ecx, 1
    mov     r9d, 10
    movups  [rsp+16], xmm0
    mov     r8, rsp
    mov     rsi, rsp
    mov     BYTE [rsp+31], 10
    movups  [rsp], xmm0
.L2:
    mov     rax, rdi
    xor     edx, edx
    mov     r10, rdi
    dec     r8
    div     r9
    add     edx, 48
    mov     rdi, rax
    mov     BYTE [r8+31], dl
    mov     edx, ecx
    inc     ecx
    cmp     r10, 9
    ja      .L2
    mov     eax, 31
    add     edx, 2
    mov     edi, 1
    sub     eax, ecx
    movsx   rdx, edx
    cdqe
    add     rsi, rax
    mov     rax, 1              ;Syscall code for write
    syscall                     ;Calling write syscall
    add     rsp, 40
    ret

_start:
"#;

const ASM_FOOTER: &'static str = "
    mov     rax, 60             ;Syscall code for exit
    mov     rdi, 0              ;Param: exit code
    syscall                     ;Calling exit syscall
";

fn usage(program: &String) -> String{
    format!("{program} <option> [filepath] [output-filepath]
option :
\tsim\t\tsimulate the given program within rust
\tcom\t\tcompile the given program to native elf64 executable
\t\t\tFor both `sim` and `com` option, filepath is mandatory !
\t\t\tIf output-filepath is not provided for the `com` option, the output will be automatically named a.out
\thelp\tprint this help message")
}

fn main() {
    let args = args().collect::<Vec<_>>();
    assert!(args.len() >= 1);

    if args.len() < 2 {
        eprintln!("ERROR: Not enough argument for the program");
        eprintln!("{}", usage(args.get(0).unwrap()));
        exit(1);
    }

    let option = args.get(1).unwrap().as_str();
    let filepath :String;
    match option {
        "sim"|"com" => {
            if args.len() < 3 {
                eprintln!("ERROR: No filepath were provided");
                eprintln!("{}", usage(args.get(0).unwrap()));
                exit(1);
            }
            filepath = args.get(2).unwrap().to_string();
        }
        "help" => {
            println!("ERROR: Unknown command");
            println!("{}", usage(args.get(0).unwrap()));
            exit(0);
        }
        _ => {
            eprintln!("ERROR: Unknown command");
            eprintln!("{}", usage(args.get(0).unwrap()));
            exit(1);
        }
    }

    let file_content: Vec<_> =
        match fs::read_to_string(filepath.clone()){
            Ok(content) => content,
            Err(err) => {
                eprintln!("ERROR: Could not read file {}: {err}", filepath);
                exit(1);
            },
        }.chars().collect();

    let ops: Vec<Op> = Lexer::new(filepath.clone(), file_content.as_slice()).map(|token| token.to_op()).collect();


    match option {
        "sim" => {
            let mut stack = vec![];
            for op in ops {
                op.simulate(&mut stack)
            }
        }
        "com" => {
            let dot_index = filepath.find('.');
            let file_basename :String;
            let output_filepath;

            if let Some(idx) = dot_index {
                file_basename = filepath.split_at(idx).0.to_string();
            } else {
                file_basename = filepath;
            }

            if let Some(output) = args.get(3) {
                output_filepath = output.to_string();
            }else{
                output_filepath = "a.out".to_string();
            }

            if !Command::new("rm")
                .arg(format!("{file_basename}.asm"))
                .status().expect("ERROR: Cannot execute rm").success(){
                    eprintln!("ERROR: rm exited unsuccesfully");
                    exit(1);
                }

            let mut output_asm = match OpenOptions::new().create_new(true).append(true).open(format!("{file_basename}.asm")){
                Ok(file) => file,
                Err(err) => {
                    eprintln!("ERROR: Could not create assembly file : {err}");
                    exit(1);
                },
            };

            output_asm.write(ASM_HEADER.as_bytes()).expect("ERROR: Could not write to file");
            for op in ops {
                op.compile(&mut output_asm).expect("ERROR: Could not write to file");
            }
            output_asm.write(ASM_FOOTER.as_bytes()).expect("ERROR: Could not write to file");

            println!("INFO: Running `nasm -f elf64 {file_basename}.asm` ...");
            if !Command::new("nasm")
                .arg("-f elf64")
                .arg(format!("{file_basename}.asm"))
                .status().expect("ERROR: Cannot execute nasm").success(){
                    eprintln!("ERROR: nasm exited unsuccesfully");
                    exit(1);
                }

            println!("INFO: Runnning ld {file_basename}.o -o {output_filepath} ...");
            if !Command::new("ld")
                .arg(format!("{file_basename}.o"))
                .arg("-o")
                .arg(format!("{output_filepath}"))
                .status().expect("ERROR: Cannot execute ld").success(){
                    eprintln!("ERROR: ld exited unsuccesfully");
                    exit(1);
                }

            println!("INFO: Runnning rm {file_basename}.o ...");
            if !Command::new("rm")
                .arg(format!("{file_basename}.o"))
                .status().expect("ERROR: Cannot execute rm").success(){
                    eprintln!("ERROR: rm exited unsuccesfully");
                    exit(1);
                }
        }
        _ => unreachable!()
    }
}

use std::{fs, process::exit};

use lexer::{op::Op};

use crate::lexer::Lexer;

pub mod lexer;

fn main() {
    let file_path = "test.ssmpl";
    let file_content =
        match fs::read_to_string(file_path){
            Ok(content) => content,
            Err(err) => {
                eprintln!("ERROR: Could not read file {file_path}: {err}");
                exit(1);
            },
        }.chars().collect::<Vec<_>>();

    let ops: Vec<Op> = Lexer::new(file_path, file_content.as_slice()).map(|token| token.to_op()).collect();
    let mut stack = vec![];
    for op in ops {
        op.simulate(&mut stack)
    }
}

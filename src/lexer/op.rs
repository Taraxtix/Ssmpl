use std::{fmt::Display, process::exit};

pub struct Loc{
    pub file_path: &'static str,
    pub line: usize,
    pub col: usize
}

impl Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("{}:{}:{}", self.file_path, self.line, self.col).fmt(f)
    }
}

pub enum OpType {
    Push(u64),
    Dump
}

impl Display for OpType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpType::Push(_) => "Push".fmt(f),
            OpType::Dump => "Dump".fmt(f),
        }
    }
}

pub struct Op {
    pub op_type: OpType,
    pub loc: Loc
}

impl Op {
    fn pop(&self, stack: &mut Vec<u64>) -> u64{
        match stack.pop() {
            Some(val) => val,
            None => {
                eprintln!("{}: Not enough argument on the stack for op: `{}`", self.loc, self.op_type);
                exit(1);
            },
        }
    }

    pub fn simulate(&self, stack: &mut Vec<u64>) {
        match self.op_type {
            OpType::Push(val) => stack.push(val),
            OpType::Dump => println!("{}", self.pop(stack)),
        }
    }
}

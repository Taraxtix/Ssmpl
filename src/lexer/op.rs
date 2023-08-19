use std::{fmt::Display, process::exit, io::{Error, Write}};

pub struct Loc{
    pub file_path: String,
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
    Dump,
    Minus,
    Plus,
    Mult,
    Div,
    Inc,
    Dec,
    Drop,
    Drop2,
    Swap,
    Over,
    Over2,
    Dup,
}

impl Display for OpType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpType::Push(_) => "Push".fmt(f),
            OpType::Dump => "Dump".fmt(f),
            OpType::Plus => "Plus".fmt(f),
            OpType::Minus => "Minus".fmt(f),
            OpType::Mult => "Mult".fmt(f),
            OpType::Div => "Div".fmt(f),
            OpType::Inc => "Inc".fmt(f),
            OpType::Dec => "Dec".fmt(f),
            OpType::Drop => "Drop".fmt(f),
            OpType::Drop2 => "2Drop".fmt(f),
            OpType::Swap => "Swap".fmt(f),
            OpType::Over => "Over".fmt(f),
            OpType::Over2 => "2Over".fmt(f),
            OpType::Dup => "Dup".fmt(f),
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
            OpType::Minus => {
                let b = self.pop(stack);
                let a = self.pop(stack);
                stack.push(a - b);
            }
            OpType::Plus => {
                let b = self.pop(stack);
                let a = self.pop(stack);
                stack.push(a + b);
            }
            OpType::Mult => {
                let b = self.pop(stack);
                let a = self.pop(stack);
                stack.push(a * b);
            }
            OpType::Div => {
                let b = self.pop(stack);
                let a = self.pop(stack);
                stack.push(a / b);
                stack.push(a % b);
            }
            OpType::Inc => {
                let a = self.pop(stack);
                stack.push(a + 1);
            }
            OpType::Dec => {
                let a = self.pop(stack);
                stack.push(a - 1);
            }
            OpType::Drop => {
                self.pop(stack);
            }
            OpType::Drop2 => {
                self.pop(stack);
                self.pop(stack);
            }
            OpType::Swap => {
                let a = self.pop(stack);
                let b = self.pop(stack);
                stack.push(a);
                stack.push(b);
            }
            OpType::Over => {
                let a = self.pop(stack);
                let b = self.pop(stack);
                stack.push(b);
                stack.push(a);
                stack.push(b);
            }
            OpType::Over2 => {
                let a = self.pop(stack);
                let b = self.pop(stack);
                let c = self.pop(stack);
                stack.push(c);
                stack.push(b);
                stack.push(a);
                stack.push(c);
            },
            OpType::Dup => {
                let a = self.pop(stack);
                stack.push(a);
                stack.push(a);
            }
        }
    }

    pub fn compile(&self, output_asm: &mut std::fs::File) -> Result<usize, Error> {
        match self.op_type {
            OpType::Push(val) => {
                output_asm.write(format!("\t;; Pushing {val}\n").as_bytes())?;
                output_asm.write(format!("\tpush\t{val}\n").as_bytes())
            }
            OpType::Dump => {
                output_asm.write("\t;; Calling Dump\n".as_bytes())?;
                output_asm.write("\tpop \trdi\n".as_bytes())?;
                output_asm.write("\tcall\tdump\n".as_bytes())
            }
            OpType::Minus => {
                output_asm.write("\t;; Minus\n".as_bytes())?;
                output_asm.write("\tpop \trbx\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\tsub \trax, rbx\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())
            }
            OpType::Plus => {
                output_asm.write("\t;; Plus\n".as_bytes())?;
                output_asm.write("\tpop \trbx\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\tadd \trax, rbx\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())
            }
            OpType::Mult => {
                output_asm.write("\t;; Mult\n".as_bytes())?;
                output_asm.write("\tpop \trbx\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\timul\trax, rbx\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())
            }
            OpType::Div => {
                output_asm.write("\t;; Div\n".as_bytes())?;
                output_asm.write("\tpop \trbx\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\tcqo\n".as_bytes())?;
                output_asm.write("\tidiv\trbx\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())?;
                output_asm.write("\tpush\trdx\n".as_bytes())
            }
            OpType::Inc => {
                output_asm.write("\t;; Inc\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\tinc \trax\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())
            }
            OpType::Dec => {
                output_asm.write("\t;; Dec\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\tdec \trax\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())
            }
            OpType::Drop => {
                output_asm.write("\t;; Drop\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())
            }
            OpType::Drop2 => {
                output_asm.write("\t;; 2Drop\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())
            }
            OpType::Swap => {
                output_asm.write("\t;; Swap\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\tpop \trbx\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())?;
                output_asm.write("\tpush\trbx\n".as_bytes())
            }
            OpType::Over => {
                output_asm.write("\t;; Over\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\tpop \trbx\n".as_bytes())?;
                output_asm.write("\tpush\trbx\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())?;
                output_asm.write("\tpush\trbx\n".as_bytes())
            }
            OpType::Over2 => {
                output_asm.write("\t;; 2Over\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\tpop \trbx\n".as_bytes())?;
                output_asm.write("\tpop \trcx\n".as_bytes())?;
                output_asm.write("\tpush\trcx\n".as_bytes())?;
                output_asm.write("\tpush\trbx\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())?;
                output_asm.write("\tpush\trcx\n".as_bytes())
            }
            OpType::Dup => {
                output_asm.write("\t;; Dup\n".as_bytes())?;
                output_asm.write("\tpop \trax\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())?;
                output_asm.write("\tpush\trax\n".as_bytes())
            }
        }
    }
}

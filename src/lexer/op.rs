use std::{fmt::Display, process::exit, io::{Error, Write}};

#[derive(Clone)]
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

#[derive(Clone)]
pub enum OpType {
    Push(u64),
    Dump,
    Minus,
    Plus,
    Mul,
    Div,
    Inc,
    Dec,
    Drop,
    Drop2,
    Swap,
    Over,
    Over2,
    Dup,
    If,
    While,
    Do(u64),
    Else(u64),
    End(u64),
    Equal,
    NEqual,
    Greater,
    GreaterE,
    Less,
    LessE,
}

impl Display for OpType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpType::Push(_) => "Push".fmt(f),
            OpType::Dump => "Dump".fmt(f),
            OpType::Plus => "Plus".fmt(f),
            OpType::Minus => "Minus".fmt(f),
            OpType::Mul => "Mult".fmt(f),
            OpType::Div => "Div".fmt(f),
            OpType::Inc => "Inc".fmt(f),
            OpType::Dec => "Dec".fmt(f),
            OpType::Drop => "Drop".fmt(f),
            OpType::Drop2 => "2Drop".fmt(f),
            OpType::Swap => "Swap".fmt(f),
            OpType::Over => "Over".fmt(f),
            OpType::Over2 => "2Over".fmt(f),
            OpType::Dup => "Dup".fmt(f),
            OpType::If => "If".fmt(f),
            OpType::While => "While".fmt(f),
            OpType::Do(_) => "Do".fmt(f),
            OpType::Else(_) => "Else".fmt(f),
            OpType::End(_) => "End".fmt(f),
            OpType::Equal => "Equal".fmt(f),
            OpType::NEqual => "NEqual".fmt(f),
            OpType::Greater => "Greater".fmt(f),
            OpType::GreaterE => "GreaterE".fmt(f),
            OpType::Less => "Less".fmt(f),
            OpType::LessE => "LessE".fmt(f),
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
}

#[derive(Clone)]
enum DataType {
    Int,
    Ptr,
    Bool,
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Int => "INT".fmt(f),
            DataType::Ptr => "PTR".fmt(f),
            DataType::Bool => "BOOL".fmt(f),
        }
    }
}

fn wrong_arg(op: &Op, expected :&str, got :Vec<Option<DataType>>) -> ! {
    eprintln!("ERROR: {}: Wrong argument for {}.", op.loc, op.op_type);
    eprintln!("Expected: {expected}");
    eprint!("Got     : ");
    for dt in got.as_slice()[..expected.len()-1].iter(){
        if let Some(dt) = dt {
            eprint!("{dt}, ");
        } else{
            eprint!("None, ")
        }
    }
    if let Some(dt) = got.last().unwrap() {
        eprint!("{dt}");
    } else{
        eprint!("None")
    }
    eprintln!();
    exit(1)
}

pub fn type_check(ops: &[Op]){
    let mut ip: usize = 0;
    let mut stack: Vec<DataType> = vec![];
    let mut stack_snapshot: Vec<Vec<DataType>> = vec![];
    while let Some(op) = ops.get(ip){
        match op.op_type.clone() {
            OpType::Push(_) => stack.push(DataType::Int),
            OpType::Dump => {
                match stack.pop() {
                    Some(_) => (),
                    a => wrong_arg(op, "*", vec![a]),
                }
            }
            OpType::Minus => {
                let b = stack.pop();
                let a = stack.pop();
                match b {
                    Some(DataType::Int) => match a {
                        Some(DataType::Int) => stack.push(DataType::Int),
                        Some(DataType::Ptr) => stack.push(DataType::Ptr),
                        _ => wrong_arg(op, "INT, INT|PTR", vec![b, a]),
                    },
                    _ => wrong_arg(op, "INT, INT|PTR", vec![b, a]),
                }
            }
            OpType::Plus => {
                let b = stack.pop();
                let a = stack.pop();
                match b {
                    Some(DataType::Int) => match a {
                        Some(DataType::Int) => stack.push(DataType::Int),
                        Some(DataType::Ptr) => stack.push(DataType::Ptr),
                        _ => wrong_arg(op, "INT, INT|PTR", vec![b, a]),
                    },
                    _ => wrong_arg(op, "INT, INT|PTR", vec![b, a]),
                }
            }
            OpType::Mul => {
                let b = stack.pop();
                let a = stack.pop();
                match b {
                    Some(DataType::Int) => match a {
                        Some(DataType::Int) => stack.push(DataType::Int),
                        _ => wrong_arg(op, "INT, INT", vec![b, a]),
                    },
                    _ => wrong_arg(op, "INT, INT", vec![b, a]),
                }
            }
            OpType::Div => {
                let b = stack.pop();
                let a = stack.pop();
                match b {
                    Some(DataType::Int) => match a {
                        Some(DataType::Int) => stack.push(DataType::Int),
                        _ => wrong_arg(op, "INT, INT", vec![b, a]),
                    },
                    _ => wrong_arg(op, "INT, INT", vec![b, a]),
                }
            }
            OpType::Inc => {
                let a = stack.pop();
                match a {
                    Some(DataType::Int) => stack.push(DataType::Int),
                    Some(DataType::Ptr) => stack.push(DataType::Ptr),
                    _ => wrong_arg(op, "INT|PTR", vec![a]),
                }
            }
            OpType::Dec => {
                let a = stack.pop();
                match a {
                    Some(DataType::Int) => stack.push(DataType::Int),
                    Some(DataType::Ptr) => stack.push(DataType::Ptr),
                    _ => wrong_arg(op, "INT|PTR", vec![a]),
                }
            }
            OpType::Drop => {
                let a = stack.pop();
                match a {
                    Some(_) => { stack.pop(); }
                    _ => wrong_arg(op, "*", vec![a]),
                }
            }
            OpType::Drop2 => {
                let b = stack.pop();
                let a = stack.pop();
                match a {
                    Some(_) => match b {
                        Some(_) => { stack.pop(); }
                        _ => wrong_arg(op, "*, *", vec![a, b]),
                    },
                    _ => wrong_arg(op, "*, *", vec![a, b]),
                }
            }
            OpType::Swap => {
                let b = stack.pop();
                let a = stack.pop();
                match a {
                    Some(_) => match b {
                        Some(_) => {
                            stack.push(b.unwrap());
                            stack.push(a.unwrap());
                        }
                        _ => wrong_arg(op, "*, *", vec![a, b]),
                    },
                    _ => wrong_arg(op, "*, *", vec![a, b]),
                }
            }
            OpType::Over => {
                let b = stack.pop();
                let a = stack.pop();
                match a {
                    Some(_) => match b {
                        Some(_) => {
                            stack.push(a.clone().unwrap());
                            stack.push(b.unwrap());
                            stack.push(a.unwrap());
                        }
                        _ => wrong_arg(op, "*, *", vec![a, b]),
                    },
                    _ => wrong_arg(op, "*, *", vec![a, b]),
                }
            },
            OpType::Over2 => {
                let c = stack.pop();
                let b = stack.pop();
                let a = stack.pop();
                match a {
                    Some(_) => match b {
                        Some(_) => match b {
                        Some(_) => {
                            stack.push(a.clone().unwrap());
                            stack.push(b.unwrap());
                            stack.push(c.unwrap());
                            stack.push(a.unwrap());
                        }
                        _ => wrong_arg(op, "*, *, *", vec![a, b, c]),
                    },
                        _ => wrong_arg(op, "*, *, *", vec![a, b, c]),
                    },
                    _ => wrong_arg(op, "*, *, *", vec![a, b, c]),
                }
            },
            OpType::Dup => {
                let a = stack.pop();
                match a {
                    Some(_) => stack.push(a.unwrap()),
                    _ => wrong_arg(op, "INT|PTR", vec![a]),
                }
            }
            OpType::If | OpType::While => {
                let mut snapshot = stack.clone();
                snapshot.push(DataType::Bool);
                stack_snapshot.push(snapshot);
            }
            OpType::Do(_) => {
                let snapshot = stack_snapshot.pop().unwrap();
                if stack != snapshot{
                    eprintln!("ERROR: {}: Condition block should only add a boolean to the stack without altering it.", op.loc);
                    eprint!("Expected: ");
                    for dt in snapshot.as_slice()[..snapshot.len()-1].iter() {
                        eprint!("{dt}, ");
                    }
                    eprintln!("{}", snapshot.last().unwrap());
                    eprint!("Expected: ");
                    for dt in stack.as_slice()[..stack.len()-1].iter() {
                        eprint!("{dt}, ");
                    }
                    eprintln!("{}", stack.last().unwrap());
                }else{
                    stack.pop();
                }
            }
            OpType::Else(_) => {
                stack_snapshot.pop();
                stack_snapshot.push(stack.clone());
            }
            OpType::End(_) => {
                let snapshot = stack_snapshot.pop().unwrap();
                if stack != snapshot{
                    eprintln!("ERROR: {}: Conditional block should not alter the stack.", op.loc);
                    eprint!("Expected: ");
                    for dt in snapshot.as_slice()[..snapshot.len()-1].iter() {
                        eprint!("{dt}, ");
                    }
                    eprintln!("{}", snapshot.last().unwrap());
                    eprint!("Expected: ");
                    for dt in stack.as_slice()[..stack.len()-1].iter() {
                        eprint!("{dt}, ");
                    }
                    eprintln!("{}", stack.last().unwrap());
                }else{
                    stack.pop();
                }
            }
            OpType::Equal | OpType::NEqual | OpType::Greater | OpType::GreaterE | OpType::Less | OpType::LessE => {
                let a = stack.pop();
                let b = stack.pop();
                match a.clone() {
                    Some(dt1) => match b.clone() {
                        Some(dt2) => {
                            if dt1 as u8 == dt2 as u8 {
                                stack.push(DataType::Bool);
                            }else{
                                wrong_arg(op, "<T>, <T>", vec![a, b])
                            }
                        }
                        None => wrong_arg(op, "<T>, <T>", vec![a, b]),
                    },
                    _ => wrong_arg(op, "<T>, <T>", vec![a, b]),
                }
            }
        }
        ip += 1;
    }
}

pub fn simulate(ops: Vec<Op>, stack: &mut Vec<u64>) {
    let mut ip: usize = 0;
    while let Some(op) = ops.get(ip) {
        match op.op_type {
            OpType::Push(val) => stack.push(val),
            OpType::Dump => println!("{}", op.pop(stack)),
            OpType::Minus => {
                let b = op.pop(stack);
                let a = op.pop(stack);
                stack.push(a - b);
            }
            OpType::Plus => {
                let b = op.pop(stack);
                let a = op.pop(stack);
                stack.push(a + b);
            }
            OpType::Mul => {
                let b = op.pop(stack);
                let a = op.pop(stack);
                stack.push(a * b);
            }
            OpType::Div => {
                let b = op.pop(stack);
                let a = op.pop(stack);
                stack.push(a / b);
                stack.push(a % b);
            }
            OpType::Inc => {
                let a = op.pop(stack);
                stack.push(a + 1);
            }
            OpType::Dec => {
                let a = op.pop(stack);
                stack.push(a - 1);
            }
            OpType::Drop => {
                op.pop(stack);
            }
            OpType::Drop2 => {
                op.pop(stack);
                op.pop(stack);
            }
            OpType::Swap => {
                let a = op.pop(stack);
                let b = op.pop(stack);
                stack.push(a);
                stack.push(b);
            }
            OpType::Over => {
                let a = op.pop(stack);
                let b = op.pop(stack);
                stack.push(b);
                stack.push(a);
                stack.push(b);
            }
            OpType::Over2 => {
                let a = op.pop(stack);
                let b = op.pop(stack);
                let c = op.pop(stack);
                stack.push(c);
                stack.push(b);
                stack.push(a);
                stack.push(c);
            },
            OpType::Dup => {
                let a = op.pop(stack);
                stack.push(a);
                stack.push(a);
            }
            OpType::If | OpType::While => (),
            OpType::Do(address) => {
                if op.pop(stack) == 0{
                    ip = address as usize- 1;
                }
            }
            OpType::Else(address) | OpType::End(address) => ip = address as usize- 1,
            OpType::Equal => {
                let b = op.pop(stack);
                let a = op.pop(stack);
                stack.push((a == b) as u64)
            }
            OpType::NEqual => {
                let b = op.pop(stack);
                let a = op.pop(stack);
                stack.push((a != b) as u64)
            }
            OpType::Greater => {
                let b = op.pop(stack);
                let a = op.pop(stack);
                stack.push((a > b) as u64)
            }
            OpType::GreaterE => {
                let b = op.pop(stack);
                let a = op.pop(stack);
                stack.push((a >= b) as u64)
            }
            OpType::Less => {
                let b = op.pop(stack);
                let a = op.pop(stack);
                stack.push((a < b) as u64)
            }
            OpType::LessE => {
                let b = op.pop(stack);
                let a = op.pop(stack);
                stack.push((a <= b) as u64)
            }
        }
        ip += 1;
    }
}

pub fn compile(ops: Vec<Op>, output_asm: &mut std::fs::File) -> Result<usize, Error> {
    let mut ip: usize = 0;
    while let Some(op) = ops.get(ip){
        match op.op_type {
            OpType::Push(val) => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write(format!("\t;; Pushing {val}\n").as_bytes())?;
                let _ = output_asm.write(format!("\tpush\t{val}\n").as_bytes())?;
            }
            OpType::Dump => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Calling Dump\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trdi\n".as_bytes())?;
                let _ = output_asm.write("\tcall\tdump\n".as_bytes())?;
            }
            OpType::Minus => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Minus\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tsub \trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::Plus => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Plus\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tadd \trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::Mul => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Mul\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\timul\trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::Div => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Div\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tcqo\n".as_bytes())?;
                let _ = output_asm.write("\tidiv\trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trdx\n".as_bytes())?;
            }
            OpType::Inc => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Inc\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tinc \trax\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::Dec => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Dec\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tdec \trax\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::Drop => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Drop\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
            }
            OpType::Drop2 => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; 2Drop\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
            }
            OpType::Swap => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Swap\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trbx\n".as_bytes())?;
            }
            OpType::Over => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Over\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trbx\n".as_bytes())?;
            }
            OpType::Over2 => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; 2Over\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trcx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trcx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trcx\n".as_bytes())?;
            }
            OpType::Dup => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Dup\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::If | OpType::While => (),
            OpType::Do(address) => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Do\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\ttest\trax, rax\n".as_bytes())?;
                let _ = output_asm.write(format!("\tje  \tIP_{address}\n").as_bytes())?;
            }
            OpType::Else(address) | OpType::End(address) => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Else/End\n".as_bytes())?;
                let _ = output_asm.write(format!("\tjmp \tIP_{address}\n").as_bytes())?;
            }
            OpType::Equal => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Equal\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tcmp \trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tmov \trbx, 1\n".as_bytes())?;
                let _ = output_asm.write("\tmov\trax, 0\n".as_bytes())?;
                let _ = output_asm.write("\tcmove\trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::NEqual => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; NEqual\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tcmp \trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tmov \trbx, 1\n".as_bytes())?;
                let _ = output_asm.write("\tmov\trax, 0\n".as_bytes())?;
                let _ = output_asm.write("\tcmovne\trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::Greater => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Greater\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tcmp \trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tmov \trbx, 1\n".as_bytes())?;
                let _ = output_asm.write("\tmov\trax, 0\n".as_bytes())?;
                let _ = output_asm.write("\tcmovg\trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::GreaterE => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; GreaterE\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tcmp \trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tmov \trbx, 1\n".as_bytes())?;
                let _ = output_asm.write("\tmov\trax, 0\n".as_bytes())?;
                let _ = output_asm.write("\tcmovge\trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::Less => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; Less\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tcmp \trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tmov \trbx, 1\n".as_bytes())?;
                let _ = output_asm.write("\tmov\trax, 0\n".as_bytes())?;
                let _ = output_asm.write("\tcmovl\trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
            OpType::LessE => {
                let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
                let _ = output_asm.write("\t;; LessE\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trbx\n".as_bytes())?;
                let _ = output_asm.write("\tpop \trax\n".as_bytes())?;
                let _ = output_asm.write("\tcmp \trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tmov \trbx, 1\n".as_bytes())?;
                let _ = output_asm.write("\tmov\trax, 0\n".as_bytes())?;
                let _ = output_asm.write("\tcmovle\trax, rbx\n".as_bytes())?;
                let _ = output_asm.write("\tpush\trax\n".as_bytes())?;
            }
        }
        ip += 1;
    }
    let _ = output_asm.write(format!("IP_{ip}:\n").as_bytes())?;
    output_asm.write("\t;; Exit".to_string().as_bytes())
}

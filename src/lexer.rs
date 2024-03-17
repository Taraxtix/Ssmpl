use std::{fmt::Display, process::exit};

use self::op::{Loc, Op, OpType};

pub mod op;

pub struct Token {
    pub content: String,
    loc: Loc,
}

fn end_of_token(c: char) -> bool{
    c.is_ascii_whitespace()
        || c == '('
        || c == ')'
        || c == '#'
}


pub fn to_op(tokens: Vec<Token>) -> Vec<Op> {
    let mut ip: usize = 0;
    let mut ops: Vec<Op> = vec![];
    let mut cf: Vec<(OpType, usize)> = vec![];
    while let Some(tok) = tokens.get(ip){
        let op_type: OpType;
        if let Ok(val) = tok.content.parse::<u64>(){
            op_type = OpType::Push(val);
        } else {
            match tok.content.as_str() {
                "dump" => op_type = OpType::Dump,
                "-" => op_type = OpType::Minus,
                "+" => op_type = OpType::Plus,
                "*" => op_type = OpType::Mul,
                "/" => op_type = OpType::Div,
                "++" => op_type = OpType::Inc,
                "--" => op_type = OpType::Dec,
                "drop" => op_type = OpType::Drop,
                "2drop" => op_type = OpType::Drop2,
                "swap" => op_type = OpType::Swap,
                "over" => op_type = OpType::Over,
                "2over" => op_type = OpType::Over2,
                "dup" => op_type = OpType::Dup,
                "if" => {
                    op_type = OpType::If;
                    cf.push((op_type.clone(), ip));
                }
                "while" => {
                    op_type = OpType::While;
                    cf.push((op_type.clone(), ip));
                }
                "do" => {
                    op_type = OpType::Do(0);
                    cf.push((op_type.clone(), ip));
                }
                "else" => {
                    op_type = OpType::Else(0);
                    let op_do = cf.pop().unwrap();
                    ops.get_mut(op_do.1).unwrap().op_type = OpType::Do(ip as u64+ 1);
                    cf.push((op_type.clone(), ip));
                }
                "end" => {
                    let op_do_or_else = cf.pop().unwrap();
                    ops.get_mut(op_do_or_else.1).unwrap().op_type = OpType::Do(ip as u64+ 1);
                    let op_if_or_while = cf.pop().unwrap();
                    match op_if_or_while.0 {
                        OpType::If => op_type = OpType::End(ip as u64 + 1),
                        OpType::While => op_type = OpType::End(op_if_or_while.1 as u64 + 1),
                        _ => unreachable!(),
                    }
                }
                "==" => op_type = OpType::Equal,
                "!=" => op_type = OpType::NEqual,
                ">" => op_type = OpType::Greater,
                ">=" => op_type = OpType::GreaterE,
                "<" => op_type = OpType::Less,
                "<=" => op_type = OpType::LessE,
                _ => {
                    eprintln!("ERROR: {}: Unknown word: `{}`", tok.loc, tok.content);
                    exit(1);
                }
            }
        }
        let op = Op{
            op_type, loc: tok.loc.clone()
        };
        ops.push(op);
        ip += 1;
    }
    ops
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("{}: {}", self.loc, self.content).fmt(f)
    }
}

pub struct Lexer<'a>{
    file_path: String,
    line: usize,
    line_start: usize,
    cursor: usize,
    content: &'a [char]
}

impl<'a> Lexer<'a> {
    pub fn new(file_path: String, content: &'a [char]) -> Self {
        Self { file_path, line: 0, line_start: 0, cursor: 0, content }
    }

    fn end(&self) -> bool {
        self.cursor >= self.content.len()
    }

    fn trim_left(&mut self) {
        while !self.end() && end_of_token(self.content[self.cursor]) && self.content[self.cursor] != '#' {
            if self.content[self.cursor] == '\n' {
                self.line += 1;
                self.line_start = self.cursor;
            }
            self.cursor += 1;
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.trim_left();

        if !self.end() && self.content[self.cursor] == '#' {
            self.cursor += 1;
            while !self.end() && self.content[self.cursor] != '#' {
                if self.content[self.cursor] == '\n' {
                    self.line += 1;
                    self.line_start = self.cursor;
                }
                self.cursor += 1;
            }
            self.cursor += 1;
            self.trim_left();   
        }
        
        if self.end() {
            return None;
        }
        let start = self.cursor;
        
        while !self.end() && !end_of_token(self.content[self.cursor]){
            self.cursor += 1;
        }

        let content: String = self.content[start..self.cursor].iter().collect::<String>();
        if content == "(" || content == ")" || content.is_empty(){
            return self.next();
        }

        Some(Token {
            loc: Loc{
                file_path: self.file_path.clone(),
                line: self.line,
                col: self.cursor - self.line_start,
            },
            content,
        })
    }
}

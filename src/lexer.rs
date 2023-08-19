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
}

impl Token {
    pub fn to_op(self) -> Op {
        let op_type: OpType;
        if let Ok(val) = self.content.parse::<u64>(){
            op_type = OpType::Push(val);
        } else {
            match self.content.as_str() {
                "dump" => op_type = OpType::Dump,
                "-" => op_type = OpType::Minus,
                "+" => op_type = OpType::Plus,
                "*" => op_type = OpType::Mult,
                "/" => op_type = OpType::Div,
                "++" => op_type = OpType::Inc,
                "--" => op_type = OpType::Dec,
                "drop" => op_type = OpType::Drop,
                "2drop" => op_type = OpType::Drop2,
                "swap" => op_type = OpType::Swap,
                "over" => op_type = OpType::Over,
                "2over" => op_type = OpType::Over2,
                "dup" => op_type = OpType::Dup,
                _ => {
                    eprintln!("ERROR: {}: Unknow word: `{}`", self.loc, self.content);
                    exit(1);
                }
            }
        }
        Op {
            op_type,
            loc: self.loc
        }
    }
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
        while !self.end() && end_of_token(self.content[self.cursor]){
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
            while !self.end() && self.content[self.cursor] != '#' && self.content[self.cursor] != '\n' {
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

        Some(Token {
            loc: Loc{
                file_path: self.file_path.clone(),
                line: self.line,
                col: self.cursor - self.line_start,
            },
            content: self.content[start..self.cursor].iter().collect::<String>(),
        })
    }
}

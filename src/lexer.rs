use std::{fmt::Display, process::exit};

use self::op::{Loc, Op, OpType};

pub mod op;

pub struct Token {
    content: String,
    loc: Loc,
}

impl Token {
    pub fn to_op(self) -> Op {
        let op_type: OpType;
        if let Ok(val) = self.content.parse::<u64>(){
            op_type = OpType::Push(val);
        } else {
            match self.content.as_str() {
                "dump" => op_type = OpType::Dump,
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
    file_path: &'static str,
    line: usize,
    line_start: usize,
    cursor: usize,
    content: &'a [char]
}

impl<'a> Lexer<'a> {
    pub fn new(file_path: &'static str, content: &'a [char]) -> Self {
        Self { file_path, line: 0, line_start: 0, cursor: 0, content }
    }

    fn end(&self) -> bool {
        self.cursor >= self.content.len()
    }

    fn trim_left(&mut self) {
        while !self.end() && self.content[self.cursor].is_ascii_whitespace(){
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
        if self.end() {
            return None;
        }
        let start = self.cursor;

        if self.content[self.cursor].is_alphabetic() {
            while !self.end() && self.content[self.cursor].is_alphanumeric(){
                self.cursor += 1;
            }
        } else if self.content[self.cursor].is_ascii_digit() {
            while !self.end() && self.content[self.cursor].is_ascii_digit() {
                self.cursor += 1;
            }
        } else {
            self.cursor += 1;
        }

        Some(Token {
            loc: Loc{
                file_path: self.file_path,
                line: self.line,
                col: self.cursor - self.line_start,
            },
            content: self.content[start..self.cursor].iter().collect::<String>(),
        })
    }
}

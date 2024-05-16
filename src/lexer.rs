//#region Imports
use std::fmt::Display;

use crate::{
	annotation::{Annotation, Position},
	parser::{Parser, Program},
	report::Reporter,
};
//#endregion

#[derive(Clone)]
pub struct Lexer {
	pub reporter: Reporter,
	input_path:   String,
	input:        Vec<char>,
	pos:          usize,
	line:         usize,
	line_start:   usize,
	pub strings:  Vec<String>,
}

#[derive(Clone)]
pub struct Token {
	pub typ:   TokenType,
	pub annot: Annotation,
}

impl Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: {}", self.annot, self.typ)
	}
}

#[derive(PartialEq, Clone)]
pub enum TokenType {
	IntLit(i64),
	FloatLit(f64),
	BoolLit(bool),
	Dump,
	Plus,
	Minus,
	Star,
	Slash,
	Modulo,
	DoubleMinus,
	DoublePlus,
	Drop,
	Swap,
	Over,
	Dup,
	If,
	Then,
	Else,
	End,
	While,
	Do,
	Eq,
	Neq,
	Lt,
	Gt,
	Lte,
	Gte,
	Syscall,
	StringLit(String),
	Argc,
	Argv,
	Load8,
	Load16,
	Load32,
	Load64,
	Store8,
	Store16,
	Store32,
	Store64,
	Macro,
	Id(String),
	OCurly,
	CCurly,
	OParen,
	CParen,
	Include,
	Cast,
	TypeI64,
	TypeF64,
	TypeBool,
	TypePtr,
	ShiftR,
	ShiftL,
	Or,
	BitOr,
	And,
	BitAnd,
	Not,
	Mem,
	Decla,
}

impl Display for TokenType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use TokenType::*;
		match self {
			| IntLit(v) => write!(f, "I64({})", v),
			| FloatLit(v) => write!(f, "F64({})", v),
			| Dump => write!(f, "dump"),
			| Plus => write!(f, "+"),
			| Minus => write!(f, "-"),
			| Star => write!(f, "*"),
			| Slash => write!(f, "/"),
			| DoubleMinus => write!(f, "--"),
			| DoublePlus => write!(f, "++"),
			| Modulo => write!(f, "%"),
			| Swap => write!(f, "swap"),
			| Drop => write!(f, "drop"),
			| Over => write!(f, "over"),
			| Dup => write!(f, "dup"),
			| If => write!(f, "if"),
			| Then => write!(f, "then"),
			| Else => write!(f, "else"),
			| End => write!(f, "end"),
			| While => write!(f, "while"),
			| Do => write!(f, "do"),
			| BoolLit(b) => write!(f, "{b}"),
			| Eq => write!(f, "eq"),
			| Neq => write!(f, "neq"),
			| Lt => write!(f, "lt"),
			| Gt => write!(f, "gt"),
			| Lte => write!(f, "lte"),
			| Gte => write!(f, "gte"),
			| Syscall => write!(f, "syscall"),
			| StringLit(str_lit) => write!(f, "\"{}\"", str_lit),
			| Argc => write!(f, "argc"),
			| Argv => write!(f, "argv"),
			| Load8 => write!(f, "load8"),
			| Load16 => write!(f, "load16"),
			| Load32 => write!(f, "load32"),
			| Load64 => write!(f, "load64"),
			| Store8 => write!(f, "store8"),
			| Store16 => write!(f, "store16"),
			| Store32 => write!(f, "store32"),
			| Store64 => write!(f, "store64"),
			| Macro => write!(f, "macro"),
			| Id(lit) => write!(f, "id({lit})"),
			| OCurly => write!(f, "{{"),
			| CCurly => write!(f, "}}"),
			| OParen => write!(f, "("),
			| CParen => write!(f, ")"),
			| Include => write!(f, "include"),
			| Cast => write!(f, "cast"),
			| TypeI64 => write!(f, "I64"),
			| TypeF64 => write!(f, "F64"),
			| TypeBool => write!(f, "Bool"),
			| TypePtr => write!(f, "Ptr"),
			| ShiftR => write!(f, ">>"),
			| ShiftL => write!(f, "<<"),
			| BitOr => write!(f, "||"),
			| Or => write!(f, "|"),
			| BitAnd => write!(f, "&&"),
			| And => write!(f, "&"),
			| Not => write!(f, "!"),
			| Mem => write!(f, "mem"),
			| Decla => write!(f, "decla"),
		}
	}
}

impl Lexer {
	const WORD_STOP: [char; 5] = ['/', '(', ')', '{', '}'];

	pub fn new(input: Vec<char>, input_path: String, reporter: Reporter) -> Self {
		Lexer {
			input_path,
			input,
			reporter,
			pos: 0,
			line: 1,
			line_start: 0,
			strings: vec![],
		}
	}

	fn trim(&mut self) -> &mut Self { self.skip_while(|c| c.is_whitespace()) }

	fn is_end(&self) -> bool { self.pos >= self.input.len() }

	fn take_while<F>(&mut self, pred: F) -> String
	where F: Fn(&char) -> bool {
		let start = self.pos;
		self.skip_while(|c| pred(c));
		self.input[start..self.pos].iter().collect()
	}

	fn at(&self) -> char {
		if self.is_end() {
			return '\0';
		}
		self.input[self.pos]
	}

	fn start_with<T>(&self, s: T) -> bool
	where T: Into<String> + Copy {
		if self.pos + s.into().len() > self.input.len() {
			return false;
		}
		self.input[self.pos..]
			.starts_with(s.into().chars().collect::<Vec<_>>().as_slice())
	}

	pub fn get_annot(&self) -> Annotation { Annotation::new(self.get_pos()) }

	pub fn get_pos(&self) -> Position {
		Position::new(self.input_path.clone(), self.line, self.pos - self.line_start)
	}

	pub fn add_error(&mut self, msg: String) -> &mut Self {
		self.reporter.add_error(format!("{}: {}", self.get_pos(), msg));
		self
	}

	pub fn exit(&mut self, code: i32) -> ! { self.reporter.exit(code) }

	fn skip_n(&mut self, n: usize) -> &mut Self {
		for _ in 0..n {
			if self.is_end() {
				return self;
			}
			if self.at() == '\n' {
				self.line += 1;
				self.line_start = self.pos;
			}
			self.pos += 1;
		}
		self
	}

	fn skip_while<F>(&mut self, pred: F) -> &mut Self
	where F: Fn(&char) -> bool {
		while (!self.is_end()) && pred(&self.at()) {
			self.skip_n(1);
		}
		self
	}

	fn skip_until_str(&mut self, s: &str) -> &mut Self {
		while (!self.is_end()) && !self.start_with(s) {
			self.skip_n(1);
		}
		self.skip_n(s.len())
	}

	fn take_word(&mut self) -> String {
		let mut lit =
			self.take_while(|c| !c.is_whitespace() && !Self::WORD_STOP.contains(c));
		if lit.is_empty() {
			lit = self.at().to_string();
			self.skip_n(1);
		}
		lit
	}

	pub fn parse(self) -> Program { Program::new(Parser::new(self)) }

	fn lex_number(&mut self, lit: &str) -> Result<Option<Token>, String> {
		let (lit, neg) = if lit.starts_with('-') {
			(lit.strip_prefix('-').unwrap(), true)
		} else {
			(lit, false)
		};

		Ok(if lit.contains('.') {
			let f_lit = lit.parse::<f64>().map_err(|e| e.to_string())?;
			Some(Token {
				typ:   TokenType::FloatLit(if neg { -f_lit } else { f_lit }),
				annot: self.get_annot(),
			})
		} else {
			let int_lit = if lit.starts_with("0b") {
				i64::from_str_radix(lit.strip_prefix("0b").unwrap(), 2)
					.map_err(|e| e.to_string())?
			} else if lit.starts_with("0o") {
				i64::from_str_radix(lit.strip_prefix("0b").unwrap(), 2)
					.map_err(|e| e.to_string())?
			} else if lit.starts_with("0x") {
				i64::from_str_radix(lit.strip_prefix("0b").unwrap(), 2)
					.map_err(|e| e.to_string())?
			} else if let Ok(int_lit) = lit.parse::<i64>() {
				int_lit
			} else {
				return if neg {
					Err("Invalid negative number".into())
				} else {
					Ok(None)
				};
			};
			Some(Token {
				typ:   TokenType::IntLit(if neg { -int_lit } else { int_lit }),
				annot: self.get_annot(),
			})
		})
	}
}

impl Iterator for Lexer {
	type Item = Token;

	fn next(&mut self) -> Option<Self::Item> {
		use TokenType::*;
		if self.trim().is_end() {
			return None;
		}

		if self.start_with("//") {
			return self.skip_until_str("\n").next();
		}
		if self.start_with("/*") {
			return self.skip_until_str("*/").next();
		}
		if self.at() == '"' {
			self.skip_n(1);
			let start_pos = self.get_pos();
			let start = self.pos;
			while self.at() != '"' {
				if self.is_end() || self.at() == '\n' {
					self.add_error(format!("{start_pos}: Unterminated string literal"))
						.exit(1);
				}
				if self.at() == '\\' {
					self.skip_n(1);
				}
				self.skip_n(1);
			}
			let lit = unescape_string(self.input[start..self.pos].iter().collect());
			if !self.strings.contains(&lit) {
				self.strings.push(lit.clone());
			}
			self.skip_n(1);
			return Some(Token { typ: StringLit(lit), annot: self.get_annot() });
		}
		if self.at() == '\'' {
			self.skip_n(1);
			let escaped = if self.at() == '\\' {
				self.skip_n(1);
				true
			} else {
				false
			};
			if ['\n', '\r', '\0'].contains(&self.at()) {
				self.add_error(format!(
					"{}: Unterminated character literal",
					self.get_pos()
				))
				.exit(1)
			}
			let mut lit = self.at();
			self.skip_n(1);
			if self.at() != '\'' {
				self.add_error(format!(
					"{}: Unterminated or too long character literal",
					self.get_pos()
				))
				.exit(1)
			}
			self.skip_n(1);
			if escaped {
				lit = match lit {
					| 'n' => '\n',
					| 'r' => '\r',
					| 't' => '\t',
					| '0' => '\0',
					| c => {
						self.add_error(format!(
							"{}: Invalid escape sequence in character literal: \\{c}",
							self.get_pos()
						))
						.exit(1)
					}
				};
			}
			return Some(Token { typ: IntLit(lit as i64), annot: self.get_annot() });
		}

		let lit = self.take_word();
		Some(Token {
			typ:   match lit.as_str() {
				| "dump" => Dump,
				| "+" => Plus,
				| "-" => Minus,
				| "*" => Star,
				| "/" => Slash,
				| "++" => DoublePlus,
				| "--" => DoubleMinus,
				| "%" => Modulo,
				| "swap" => Swap,
				| "if" => If,
				| "then" => Then,
				| "else" => Else,
				| "end" => End,
				| "while" => While,
				| "do" => Do,
				| "true" => BoolLit(true),
				| "false" => BoolLit(false),
				| "==" => Eq,
				| "!=" => Neq,
				| "<" => Lt,
				| ">" => Gt,
				| "<=" => Lte,
				| ">=" => Gte,
				| "argc" => Argc,
				| "argv" => Argv,
				| "<|8" => Load8,
				| "<|16" => Load16,
				| "<|32" => Load32,
				| "<|64" => Load64,
				| "|>8" => Store8,
				| "|>16" => Store16,
				| "|>32" => Store32,
				| "|>64" => Store64,
				| "macro" => Macro,
				| "drop" => Drop,
				| "dup" => Dup,
				| "over" => Over,
				| "syscall" => Syscall,
				| "{" => OCurly,
				| "}" => CCurly,
				| "(" => OParen,
				| ")" => CParen,
				| "include" => Include,
				| "cast" => Cast,
				| "I64" => TypeI64,
				| "F64" => TypeF64,
				| "Bool" => TypeBool,
				| "Ptr" => TypePtr,
				| ">>" => ShiftR,
				| "<<" => ShiftL,
				| "||" => Or,
				| "|" => BitOr,
				| "&&" => And,
				| "&" => BitAnd,
				| "!" => Not,
				| "mem" => Mem,
				| "decla" => Decla,
				| lit => {
					match self.lex_number(lit) {
						| Ok(Some(typ)) => return Some(typ.clone()),
						| Ok(None) => Id(lit.into()),
						| Err(e) => {
							let pos = self.get_pos();
							let msg = if e == "Invalid negative number" {
								format!("{pos}: Identifier cannot start with `-`")
							} else {
								format!("{pos}: Unable to parse number {lit}")
							};
							self.add_error(msg).exit(1)
						}
					}
				}
			},
			annot: self.get_annot(),
		})
	}
}

//#region Utils Functions
fn unescape_string(s: String) -> String {
	s.replace("\\n", "\n")
		.replace("\\t", "\t")
		.replace("\\r", "\r")
		.replace("\\0", "\0")
		.replace("\\'", "'")
		.replace("\\\"", "\"")
		.replace("\\\\", "\\")
}
//#endregion

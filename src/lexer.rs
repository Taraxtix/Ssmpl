//#region Imports
use std::fmt::Display;

use crate::{
	annotation::{Annotation, Position},
	parser::Program,
	report::Reporter,
};
//#endregion

pub struct Lexer<'a> {
	pub reporter: Reporter,
	input_path:   &'a String,
	input:        &'a [char],
	pos:          usize,
	line:         usize,
	line_start:   usize,
	pub strings:  Vec<String>,
}
pub enum Token<'a> {
	IntLit(i64, Annotation<'a>),
	FloatLit(f64, Annotation<'a>),
	BoolLit(bool, Annotation<'a>),
	Dump(Annotation<'a>),
	Plus(Annotation<'a>),
	Minus(Annotation<'a>),
	Star(Annotation<'a>),
	Slash(Annotation<'a>),
	Modulo(Annotation<'a>),
	DoubleMinus(Annotation<'a>),
	DoublePlus(Annotation<'a>),
	Drop(i64, Annotation<'a>),
	Swap(Annotation<'a>),
	Over(i64, Annotation<'a>),
	Dup(i64, Annotation<'a>),
	If(Annotation<'a>),
	Then(Annotation<'a>),
	Else(Annotation<'a>),
	End(Annotation<'a>),
	While(Annotation<'a>),
	Do(Annotation<'a>),
	Eq(Annotation<'a>),
	Neq(Annotation<'a>),
	Lt(Annotation<'a>),
	Gt(Annotation<'a>),
	Lte(Annotation<'a>),
	Gte(Annotation<'a>),
	Syscall(usize, usize, Annotation<'a>),
	StringLit(String, Annotation<'a>),
	Argc(Annotation<'a>),
	Argv(Annotation<'a>),
	Deref(Annotation<'a>),
}

impl Display for Token<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			| Token::IntLit(v, _) => write!(f, "I64({})", v),
			| Token::FloatLit(v, _) => write!(f, "F64({})", v),
			| Token::Dump(_) => write!(f, "dump"),
			| Token::Plus(_) => write!(f, "+"),
			| Token::Minus(_) => write!(f, "-"),
			| Token::Star(_) => write!(f, "*"),
			| Token::Slash(_) => write!(f, "/"),
			| Token::DoubleMinus(_) => write!(f, "--"),
			| Token::DoublePlus(_) => write!(f, "++"),
			| Token::Modulo(_) => write!(f, "%"),
			| Token::Swap(_) => write!(f, "swap"),
			| Token::Drop(n, _) => write!(f, "drop{}", n),
			| Token::Over(n, _) => write!(f, "over{}", n),
			| Token::Dup(n, _) => write!(f, "dup{}", n),
			| Token::If(_) => write!(f, "if"),
			| Token::Then(_) => write!(f, "then"),
			| Token::Else(_) => write!(f, "else"),
			| Token::End(_) => write!(f, "end"),
			| Token::While(_) => write!(f, "while"),
			| Token::Do(_) => write!(f, "do"),
			| Token::BoolLit(b, _) => {
				if *b {
					write!(f, "true")
				} else {
					write!(f, "false")
				}
			},
			| Token::Eq(_) => write!(f, "eq"),
			| Token::Neq(_) => write!(f, "neq"),
			| Token::Lt(_) => write!(f, "lt"),
			| Token::Gt(_) => write!(f, "gt"),
			| Token::Lte(_) => write!(f, "lte"),
			| Token::Gte(_) => write!(f, "gte"),
			| Token::Syscall(_, syscode, _) => write!(f, "syscall({syscode})"),
			| Token::StringLit(str_lit, _) => write!(f, "\"{}\"", str_lit),
			| Token::Argc(_) => write!(f, "argc"),
			| Token::Argv(_) => write!(f, "argv"),
			| Token::Deref(_) => write!(f, "deref"),
		}
	}
}

impl<'a> Lexer<'a> {
	const WORD_STOP: [char; 4] = ['/', '(', ')', '.'];

	pub fn new<T>(input: T, input_path: &'a String, reporter: Reporter) -> Self
	where T: Into<&'a [char]> {
		Lexer {
			input_path,
			input: input.into(),
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

	fn get_annot(&self) -> Annotation<'a> { Annotation::new(self.get_pos()) }

	fn get_pos(&self) -> Position<'a> {
		Position::new(self.input_path, self.line, self.pos - self.line_start)
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
		self.take_while(|c| !c.is_whitespace() && !Self::WORD_STOP.contains(c))
	}

	fn parse_number(&mut self, neg: bool) -> Option<Token<'a>> {
		let (pred, radix, skip): (fn(&char) -> bool, u32, usize) =
			if self.start_with("0b") {
				(|c: &char| c == &'0' || c == &'1', 2, 2)
			} else if self.start_with("0o") {
				(|c: &char| ('0'..='7').contains(&c.to_ascii_lowercase()), 8, 2)
			} else if self.start_with("0x") {
				(
					|c: &char| {
						c.is_numeric() || ('a'..='f').contains(&c.to_ascii_lowercase())
					},
					16,
					2,
				)
			} else {
				(|c: &char| c.is_numeric(), 10, 0)
			};
		let lit = self.skip_n(skip).take_while(pred);
		if self.at() == '.' {
			let decimal_part = self.skip_n(1).take_while(|c| c.is_numeric());
			let value: f64 =
				format!("{lit}.{decimal_part}").parse().unwrap_or_else(|e| {
					self.add_error(format!("Cannot parse int_literal ({lit}) : {e}"))
						.exit(1);
				});
			Some(Token::FloatLit(if neg { -value } else { value }, self.get_annot()))
		} else if lit == *"" {
			None
		} else {
			let value = i64::from_str_radix(lit.as_str(), radix).unwrap_or_else(|e| {
				self.add_error(format!("Cannot parse number ({lit}) : {e}")).exit(1);
			});
			Some(Token::IntLit(if neg { -value } else { value }, self.get_annot()))
		}
	}

	fn get_opt_size_arg(&mut self) -> i64 {
		match self.parse_number(false) {
			| Some(Token::IntLit(value, _)) => value,
			| None => 1,
			| _ => self.add_error("Invalid size argument".to_string()).exit(1),
		}
	}

	fn expect_size_arg(&mut self) -> i64 {
		match self.parse_number(false) {
			| Some(Token::IntLit(value, _)) => value,
			| _ => self
				.add_error(format!("{} expected size argument", self.get_annot()))
				.exit(1),
		}
	}

	fn expect(&mut self, lit: &str) {
		if !self.start_with(lit) {
			self.add_error(format!("{} expected {}", self.get_annot(), lit)).exit(1);
		}
		self.skip_n(lit.len());
	}

	pub fn parse(self) -> Program<'a> { Program::new(self) }
}

impl<'a> Iterator for Lexer<'a> {
	type Item = Token<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.trim().is_end() {
			return None;
		}

		if self.start_with("//") {
			return self.skip_until_str("\n").next();
		}
		if self.start_with("/*") {
			return self.skip_until_str("*/").next();
		}
		if self.at() == '/' {
			self.skip_n(1);
			return Some(Token::Slash(self.get_annot()));
		}

		if self.at() == '-' {
			if let Some(lit) = self.skip_n(1).parse_number(true) {
				return Some(lit);
			} else if self.at() == '-' {
				self.skip_n(1);
				return Some(Token::DoubleMinus(self.get_annot()));
			} else {
				return Some(Token::Minus(self.get_annot()));
			}
		}
		if let Some(lit) = self.parse_number(false) {
			return Some(lit);
		}
		if self.at() == '"' {
			let start_pos = self.get_pos();
			self.skip_n(1);
			let start = self.pos;
			while self.at() != '"' {
				if self.at() == '\\' {
					self.skip_n(1);
				}
				if self.at() == '\n' {
					self.add_error(format!("{start_pos}: Unterminated string literal"))
						.exit(1);
				}
				self.skip_n(1);
				if self.is_end() {
					self.add_error(format!("{start_pos}: Unterminated string literal"))
						.exit(1);
				}
			}
			let lit = unescape_string(self.input[start..self.pos].iter().collect());
			if !self.strings.contains(&lit) {
				self.strings.push(lit.clone());
			}
			self.skip_n(1);
			return Some(Token::StringLit(lit, self.get_annot()));
		}

		if self.start_with("drop") {
			self.skip_n(4);
			return Some(Token::Drop(self.get_opt_size_arg(), self.get_annot()));
		}
		if self.start_with("over") {
			self.skip_n(4);
			return Some(Token::Over(self.get_opt_size_arg(), self.get_annot()));
		}
		if self.start_with("dup") {
			self.skip_n(3);
			return Some(Token::Dup(self.get_opt_size_arg(), self.get_annot()));
		}
		if self.start_with("syscall") {
			self.skip_n(7);
			self.expect("(");
			let syscode = self.expect_size_arg();
			self.expect(")");
			return Some(Token::Syscall(
				syscode as usize,
				get_arg_count_from_syscode(&(syscode as usize)),
				self.get_annot(),
			));
		}

		let lit = self.take_word();
		match lit.as_str() {
			| "dump" => Some(Token::Dump(self.get_annot())),
			| "+" => Some(Token::Plus(self.get_annot())),
			| "*" => Some(Token::Star(self.get_annot())),
			| "++" => Some(Token::DoublePlus(self.get_annot())),
			| "--" => Some(Token::DoubleMinus(self.get_annot())),
			| "%" => Some(Token::Modulo(self.get_annot())),
			| "swap" => Some(Token::Swap(self.get_annot())),
			| "if" => Some(Token::If(self.get_annot())),
			| "then" => Some(Token::Then(self.get_annot())),
			| "else" => Some(Token::Else(self.get_annot())),
			| "end" => Some(Token::End(self.get_annot())),
			| "while" => Some(Token::While(self.get_annot())),
			| "do" => Some(Token::Do(self.get_annot())),
			| "true" => Some(Token::BoolLit(true, self.get_annot())),
			| "false" => Some(Token::BoolLit(false, self.get_annot())),
			| "==" => Some(Token::Eq(self.get_annot())),
			| "!=" => Some(Token::Neq(self.get_annot())),
			| "<" => Some(Token::Lt(self.get_annot())),
			| ">" => Some(Token::Gt(self.get_annot())),
			| "<=" => Some(Token::Lte(self.get_annot())),
			| ">=" => Some(Token::Gte(self.get_annot())),
			| "argc" => Some(Token::Argc(self.get_annot())),
			| "argv" => Some(Token::Argv(self.get_annot())),
			| "deref" => Some(Token::Deref(self.get_annot())),
			| _ => self.add_error(format!("Unknown token: {}", lit)).exit(1),
		}
	}
}

//#region Utils Functions
fn get_arg_count_from_syscode(syscode: &usize) -> usize {
	match syscode {
		| 24
		| 34
		| 39
		| 57..=58
		| 102
		| 104
		| 107..=108
		| 110..=112
		| 152..=153
		| 162
		| 186
		| 219
		| 253 => 0,
		| 3
		| 12
		| 22
		| 32
		| 37
		| 60
		| 67
		| 74..=75
		| 80..=81
		| 84
		| 87
		| 95
		| 99..=100
		| 105..=106
		| 121..=124
		| 134..=135
		| 145..=147
		| 151
		| 159
		| 161
		| 163
		| 168
		| 201
		| 207
		| 213
		| 218
		| 225..=226
		| 231
		| 241
		| 272
		| 284
		| 291
		| 306
		| 323
		| 331 => 1,
		| 4..=6
		| 11
		| 21
		| 33
		| 35..=36
		| 48
		| 50
		| 62
		| 68
		| 73
		| 76..=77
		| 79
		| 82..=83
		| 85..=86
		| 88
		| 90..=91
		| 96..=98
		| 109
		| 113..=116
		| 125..=127
		| 130..=132
		| 136..=138
		| 140
		| 142..=143
		| 148..=150
		| 155
		| 160
		| 164
		| 167
		| 170..=171
		| 176
		| 197..=201
		| 206
		| 224
		| 227..=229
		| 235
		| 244
		| 252
		| 255
		| 273
		| 283
		| 287
		| 290
		| 293
		| 300
		| 305
		| 308
		| 319
		| 324
		| 330 => 2,
		| 0..=2
		| 7..=8
		| 10
		| 16
		| 19..=20
		| 26..=31
		| 38
		| 41..=43
		| 46..=47
		| 49
		| 51..=52
		| 59
		| 64..=65
		| 71..=72
		| 78
		| 89
		| 92..=94
		| 103
		| 117..=120
		| 129
		| 133
		| 139
		| 141
		| 144
		| 173
		| 174
		| 187
		| 194..=196
		| 203..=204
		| 209..=210
		| 212
		| 217
		| 222
		| 234
		| 238
		| 245
		| 251
		| 254
		| 258
		| 261
		| 263
		| 266
		| 268..=269
		| 274
		| 282
		| 292
		| 304
		| 309
		| 313..=314
		| 317..=318
		| 325 => 3,
		| 13..=14
		| 17..=18
		| 40
		| 53
		| 61
		| 66
		| 69
		| 101
		| 128
		| 169
		| 179
		| 191..=193
		| 220..=221
		| 223
		| 230
		| 232..=233
		| 240
		| 246
		| 249
		| 256..=257
		| 259
		| 262
		| 264
		| 267
		| 276..=278
		| 280
		| 285..=286
		| 288..=289
		| 297
		| 302
		| 307
		| 315
		| 329 => 4,
		| 23
		| 25
		| 54..=56
		| 70
		| 157
		| 165
		| 188..=190
		| 208
		| 216
		| 239
		| 242..=243
		| 247..=248
		| 250
		| 260
		| 265
		| 271
		| 295..=296
		| 298..=299
		| 301
		| 303
		| 312
		| 316
		| 320
		| 322
		| 332 => 5,
		| 44..=45 | 202 | 237 | 270 | 275 | 279 | 281 | 310..=311 | 326..=328 => 6,
		| 333.. => unreachable!("invalid syscall code: {syscode}"),
		| _ => panic!("Unknown number of arguments for syscode: {syscode}"),
	}
}

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

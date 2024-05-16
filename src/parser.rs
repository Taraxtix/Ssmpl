//#region Imports
use std::{collections::HashMap, fmt::Display};

use crate::{
	annotation::{Annotation, Type},
	lexer::{Lexer, Token, TokenType},
	report::Reporter,
};
//#endregion

#[derive(Clone)]
pub enum OpType {
	PushI(i64),
	PushF(f64),
	PushB(bool),
	Dump(Type),
	Add(Type, Type),
	Sub(Type, Type),
	Mul(Type, Type),
	Div(Type, Type),
	Mod(Type, Type),
	Increment(Type),
	Decrement(Type),
	Swap,
	Drop(i64),
	Over(i64),
	Dup(i64),
	If(i64),
	Then(i64, bool),
	Else(i64),
	End(i64, bool),
	While(i64),
	Do(i64),
	Eq(Type, Type),
	Neq(Type, Type),
	Lt(Type, Type),
	Gt(Type, Type),
	Lte(Type, Type),
	Gte(Type, Type),
	Syscall(usize, usize),
	PushStr(String),
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
	Cast(Type),
	Nop,
	ShiftR,
	ShiftL,
	BitAnd,
	And,
	BitOr,
	Or,
	Not,
	Mem,
}

#[derive(Clone)]
pub struct Op {
	pub typ:   OpType,
	pub annot: Annotation,
}

impl Op {
	pub fn expected_args(&self) -> &[&str] {
		use OpType::*;
		match self.typ {
			| Drop(..) | Over(..) | Dup(..) | Then(..) | Else(..) | End(..)
			| While(..) | Do(..) | PushI(..) | PushF(..) | PushB(..) | Nop | If(..)
			| Cast(_) | PushStr(..) | Syscall(..) | Argc | Argv | Mem => unreachable!(),
			| ShiftR | ShiftL => &["I64", "_"],
			| Not | Increment(_) | Decrement(_) | Dump(_) => &["_"],
			| Swap | Mod(..) | Add(..) | Sub(..) | Mul(..) | Div(..) | Eq(..) | And
			| Or | Neq(..) | Lt(..) | Gt(..) | Lte(..) | BitAnd | BitOr | Gte(..) => &["_", "_"],
			| Store8 | Store16 | Store32 | Store64 => &["Ptr", "_"],
			| Load8 | Load16 | Load32 | Load64 => &["Ptr"],
		}
	}
}

impl Display for OpType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use OpType::*;
		match self {
			| PushI(v) => write!(f, "PushI({})", v),
			| PushF(v) => write!(f, "PushF({})", v),
			| PushB(v) => write!(f, "PushB({})", v),
			| Dump(_) => write!(f, "Dump"),
			| Add(..) => write!(f, "Add"),
			| Sub(..) => write!(f, "Sub"),
			| Mul(..) => write!(f, "Mul"),
			| Div(..) => write!(f, "Div"),
			| Mod(..) => write!(f, "Mod"),
			| Increment(_) => write!(f, "Increment"),
			| Decrement(_) => write!(f, "Decrement"),
			| Drop(n) => write!(f, "Drop{}", n),
			| Swap => write!(f, "Swap"),
			| Over(n) => write!(f, "Over{}", n),
			| Dup(n) => write!(f, "Dup{}", n),
			| If(..) => write!(f, "If"),
			| Then(..) => write!(f, "Then"),
			| Else(..) => write!(f, "Else"),
			| End(..) => write!(f, "End"),
			| While(..) => write!(f, "While"),
			| Do(..) => write!(f, "Do"),
			| Nop => unreachable!(),
			| Eq(..) => write!(f, "Eq"),
			| Neq(..) => write!(f, "Neq"),
			| Lt(..) => write!(f, "Lt"),
			| Gt(..) => write!(f, "Gt"),
			| Lte(..) => write!(f, "Lte"),
			| Gte(..) => write!(f, "Gte"),
			| Syscall(syscode, ..) => write!(f, "Syscall({})", syscode),
			| PushStr(lit) => write!(f, "PushStr({})", lit),
			| Argc => write!(f, "Argc"),
			| Argv => write!(f, "Argv"),
			| Load8 => write!(f, "Load8"),
			| Load16 => write!(f, "Load8"),
			| Load32 => write!(f, "Load8"),
			| Load64 => write!(f, "Load8"),
			| Store8 => write!(f, "Store8"),
			| Store16 => write!(f, "Store8"),
			| Store32 => write!(f, "Store8"),
			| Store64 => write!(f, "Store8"),
			| Cast(typ) => write!(f, "Cast({typ})"),
			| ShiftR => write!(f, "ShiftR"),
			| ShiftL => write!(f, "ShiftL"),
			| And => write!(f, "And"),
			| Or => write!(f, "Or"),
			| Not => write!(f, "Not"),
			| BitAnd => write!(f, "BitAnd"),
			| BitOr => write!(f, "BitOr"),
			| Mem => write!(f, "Mem"),
		}
	}
}

impl Display for Op {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: {}", self.annot, self.typ)
	}
}

#[derive(Clone)]
pub struct Program {
	pub ops:      Vec<Op>,
	pub reporter: Reporter,
	pub strings:  Vec<String>,
}

impl Program {
	pub fn new(parser: Parser) -> Self {
		Self { ops: parser.ops, reporter: parser.reporter, strings: parser.strings }
	}

	pub fn add_error(&mut self, error: String) -> &mut Self {
		self.reporter.add_error(error);
		self
	}

	pub fn add_info(&mut self, info: String) -> &mut Self {
		self.reporter.add_info(info);
		self
	}

	pub fn exit(&mut self, code: i32) -> ! { self.reporter.exit(code) }
}

#[derive(Clone)]
pub struct Parser {
	pub reporter: Reporter,
	pub ops:      Vec<Op>,
	pub macros:   HashMap<String, Vec<Op>>,
	pub strings:  Vec<String>,
	included:     Vec<String>,
}

impl Parser {
	pub fn new(lexer: Lexer) -> Self {
		let mut ops = lexer.clone().collect::<Vec<_>>();
		let mut itself = Self {
			reporter: lexer.reporter,
			strings:  lexer.strings,
			ops:      vec![],
			macros:   HashMap::new(),
			included: vec![],
		};
		while !ops.is_empty() {
			for op in itself.ops_from_first_token(&mut ops) {
				itself.ops.push(op);
			}
		}
		itself
	}

	fn ops_from_first_token(&mut self, ops: &mut Vec<Token>) -> Vec<Op> {
		use OpType as O;
		use TokenType as T;
		if ops.is_empty() {
			return vec![];
		}
		let Token { typ, annot } = ops.remove(0);
		match typ {
			| T::IntLit(v) => vec![Op { typ: O::PushI(v), annot }],
			| T::FloatLit(v) => vec![Op { typ: O::PushF(v), annot }],
			| T::BoolLit(v) => vec![Op { typ: O::PushB(v), annot }],
			| T::StringLit(lit) => {
				if !self.strings.contains(&lit) {
					self.strings.push(lit.clone());
				}
				vec![Op { typ: O::PushStr(lit), annot }]
			}
			| T::Dump => vec![Op { typ: O::Dump(Type::I64), annot }],
			| T::Plus => vec![Op { typ: O::Add(Type::I64, Type::I64), annot }],
			| T::Minus => vec![Op { typ: O::Sub(Type::I64, Type::I64), annot }],
			| T::Star => vec![Op { typ: O::Mul(Type::I64, Type::I64), annot }],
			| T::Slash => vec![Op { typ: O::Div(Type::I64, Type::I64), annot }],
			| T::Modulo => vec![Op { typ: O::Mod(Type::I64, Type::I64), annot }],
			| T::DoubleMinus => vec![Op { typ: O::Decrement(Type::I64), annot }],
			| T::DoublePlus => vec![Op { typ: O::Increment(Type::I64), annot }],
			| T::If => vec![Op { typ: O::If(0), annot }],
			| T::Then => vec![Op { typ: O::Then(0, false), annot }],
			| T::Else => vec![Op { typ: O::Else(0), annot }],
			| T::End => vec![Op { typ: O::End(0, false), annot }],
			| T::While => vec![Op { typ: O::While(0), annot }],
			| T::Do => vec![Op { typ: O::Do(0), annot }],
			| T::Eq => vec![Op { typ: O::Eq(Type::I64, Type::I64), annot }],
			| T::Neq => vec![Op { typ: O::Neq(Type::I64, Type::I64), annot }],
			| T::Lt => vec![Op { typ: O::Lt(Type::I64, Type::I64), annot }],
			| T::Gt => vec![Op { typ: O::Gt(Type::I64, Type::I64), annot }],
			| T::Lte => vec![Op { typ: O::Lte(Type::I64, Type::I64), annot }],
			| T::Gte => vec![Op { typ: O::Gte(Type::I64, Type::I64), annot }],
			| T::Argc => vec![Op { typ: O::Argc, annot }],
			| T::Argv => vec![Op { typ: O::Argv, annot }],
			| T::Load8 => vec![Op { typ: O::Load8, annot }],
			| T::Load16 => vec![Op { typ: O::Load16, annot }],
			| T::Load32 => vec![Op { typ: O::Load32, annot }],
			| T::Load64 => vec![Op { typ: O::Load64, annot }],
			| T::Store8 => vec![Op { typ: O::Store8, annot }],
			| T::Store16 => vec![Op { typ: O::Store16, annot }],
			| T::Store32 => vec![Op { typ: O::Store32, annot }],
			| T::Store64 => vec![Op { typ: O::Store64, annot }],
			| T::Swap => vec![Op { typ: O::Swap, annot }],
			| T::Drop => vec![Op { typ: O::Drop(self.expect_optional_arg(ops)), annot }],
			| T::Over => vec![Op { typ: O::Over(self.expect_optional_arg(ops)), annot }],
			| T::Dup => vec![Op { typ: O::Dup(self.expect_optional_arg(ops)), annot }],
			| T::Syscall => {
				let arg = self.expect_arg(ops) as usize;
				vec![Op { typ: O::Syscall(arg, get_arg_count_from_syscode(&arg)), annot }]
			}
			| T::Macro => {
				let name = self.expect_id(ops);
				self.expect(ops, T::OCurly);
				let macro_ops = self.collect_until(ops, T::CCurly);
				self.macros.insert(name, macro_ops);
				self.expect(ops, T::CCurly);
				self.ops_from_first_token(ops)
			}
			| T::Id(name) => {
				if let Some(macro_ops) = self.macros.get(&name) {
					macro_ops.clone()
				} else {
					self.add_error(format!(
						"{}: Undefined macro: {name}",
						annot.get_pos()
					))
					.exit(1);
				}
			}
			| T::Include => {
				let path = self.expect_string_lit(ops);
				if self.included.contains(&path) {
					return vec![];
				}
				let included_program_content = std::fs::read_to_string(path.clone())
					.unwrap_or_else(|e| {
						self.add_error(format!(
							"{}: Unable to read file {path} for include: {e}",
							annot.get_pos()
						))
						.exit(1)
					});
				let parsed_include = Parser::new(Lexer::new(
					included_program_content.chars().collect(),
					path,
					self.reporter.clone(),
				));
				self.macros.extend(parsed_include.macros);
				self.strings.extend(parsed_include.strings);
				self.included.extend(parsed_include.included);
				parsed_include.ops
			}
			| T::Cast => vec![Op { typ: O::Cast(self.expect_type_arg(ops)), annot }],
			| T::ShiftR => vec![Op { typ: O::ShiftR, annot }],
			| T::ShiftL => vec![Op { typ: O::ShiftL, annot }],
			| T::Or => vec![Op { typ: O::Or, annot }],
			| T::BitOr => vec![Op { typ: O::BitOr, annot }],
			| T::And => vec![Op { typ: O::And, annot }],
			| T::BitAnd => vec![Op { typ: O::BitAnd, annot }],
			| T::Not => vec![Op { typ: O::Not, annot }],
			| T::TypeI64
			| T::TypeF64
			| T::TypeBool
			| T::TypePtr
			| T::OCurly
			| T::CCurly
			| T::OParen
			| T::CParen => {
				self.add_error(format!("{}: Unexpected token: {typ}", annot.get_pos()))
					.exit(1)
			}
			| T::Mem => vec![Op { typ: O::Mem, annot }],
		}
	}

	pub fn add_error(&mut self, msg: String) -> &mut Self {
		self.reporter.add_error(msg);
		self
	}

	pub fn exit(&mut self, code: i32) -> ! { self.reporter.exit(code) }

	pub fn expect(&mut self, ops: &mut Vec<Token>, expected: TokenType) {
		if ops.is_empty() {
			self.add_error(format!("Expected {expected} but got nothing")).exit(1)
		}
		let Token { typ, annot } = ops.remove(0);
		if typ != expected {
			self.add_error(format!(
				"{}: Expected {expected} but got {typ}",
				annot.get_pos()
			))
			.exit(1)
		}
	}

	pub fn expect_arg(&mut self, ops: &mut Vec<Token>) -> i64 {
		self.expect(ops, TokenType::OParen);
		if ops.is_empty() {
			self.add_error("Expected size argument but got nothing".into()).exit(1)
		}
		let Token { typ, annot } = ops.remove(0);
		let arg = match typ {
			| TokenType::IntLit(arg) => arg,
			| _ => {
				self.add_error(format!(
					"{}: Expected size argument but got: {typ}",
					annot.get_pos()
				))
				.exit(1)
			}
		};
		self.expect(ops, TokenType::CParen);
		arg
	}

	pub fn expect_optional_arg(&mut self, ops: &mut Vec<Token>) -> i64 {
		match ops.first() {
			| Some(Token { typ, .. }) if *typ == TokenType::OParen => {
				self.expect_arg(ops)
			}
			| _ => 1,
		}
	}

	pub fn expect_id(&mut self, ops: &mut Vec<Token>) -> String {
		if ops.is_empty() {
			self.add_error("Expected identifier but got nothing".into()).exit(1)
		}
		let Token { typ, annot } = ops.remove(0);
		match typ {
			| TokenType::Id(id) => id,
			| _ => {
				self.add_error(format!(
					"{}: Expected identifier but got: {typ}",
					annot.get_pos()
				))
				.exit(1)
			}
		}
	}

	pub fn expect_string_lit(&mut self, ops: &mut Vec<Token>) -> String {
		if ops.is_empty() {
			self.add_error("Expected string literal but got nothing".into()).exit(1)
		}
		let Token { typ, annot } = ops.remove(0);
		match typ {
			| TokenType::StringLit(id) => id,
			| _ => {
				self.add_error(format!(
					"{}: Expected string literal but got: {typ}",
					annot.get_pos()
				))
				.exit(1)
			}
		}
	}

	pub fn expect_type_arg(&mut self, ops: &mut Vec<Token>) -> Type {
		self.expect(ops, TokenType::OParen);
		if ops.is_empty() {
			self.add_error("Expected type but got nothing".into()).exit(1)
		}
		let Token { typ, annot } = ops.remove(0);
		let typ = match typ {
			| TokenType::TypeI64 => Type::I64,
			| TokenType::TypeF64 => Type::F64,
			| TokenType::TypeBool => Type::Bool,
			| TokenType::TypePtr => Type::Ptr,
			| _ => {
				self.add_error(format!(
					"{}: Expected type but got: {typ}",
					annot.get_pos()
				))
				.exit(1)
			}
		};
		self.expect(ops, TokenType::CParen);
		typ
	}

	pub fn collect_until(&mut self, ops: &mut Vec<Token>, until: TokenType) -> Vec<Op> {
		let mut collected_ops = Vec::new();
		loop {
			let Some(Token { typ, .. }) = ops.first() else {
				self.add_error(format!("Expected {until} but got nothing",)).exit(1)
			};
			if *typ == until {
				break;
			} else {
				for op in self.ops_from_first_token(ops) {
					collected_ops.push(op);
				}
			}
		}
		collected_ops
	}
}

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

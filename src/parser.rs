//#region Imports
use std::fmt::Display;

use crate::{
	annotation::Annotation,
	lexer::{Lexer, Token},
	report::Reporter,
};
//#endregion

#[derive(Clone)]
pub enum Op<'a> {
	PushI(i64, Annotation<'a>),
	PushF(f64, Annotation<'a>),
	PushB(bool, Annotation<'a>),
	Dump(Annotation<'a>),
	Add(Annotation<'a>, Annotation<'a>),
	Sub(Annotation<'a>, Annotation<'a>),
	Mul(Annotation<'a>, Annotation<'a>),
	Div(Annotation<'a>, Annotation<'a>),
	Mod(Annotation<'a>, Annotation<'a>),
	Increment(Annotation<'a>),
	Decrement(Annotation<'a>),
	Swap(Annotation<'a>),
	Drop(i64, Annotation<'a>),
	Over(i64, Annotation<'a>),
	Dup(i64, Annotation<'a>),
	If(i64, Annotation<'a>),
	Then(i64, bool, Annotation<'a>),
	Else(i64, Annotation<'a>),
	End(i64, bool, Annotation<'a>),
	While(i64, Annotation<'a>),
	Do(i64, Annotation<'a>),
	Eq(Annotation<'a>, Annotation<'a>),
	Neq(Annotation<'a>, Annotation<'a>),
	Lt(Annotation<'a>, Annotation<'a>),
	Gt(Annotation<'a>, Annotation<'a>),
	Lte(Annotation<'a>, Annotation<'a>),
	Gte(Annotation<'a>, Annotation<'a>),
	Syscall(usize, usize, Annotation<'a>),
	PushStr(String, Annotation<'a>),
	Argc(Annotation<'a>),
	Argv(Annotation<'a>),
	Deref(Annotation<'a>),
	Nop,
}

impl<'a> Op<'a> {
	fn from_token(token: Token<'a>) -> Self {
		match token {
			| Token::IntLit(v, a) => Op::PushI(v, a),
			| Token::FloatLit(v, a) => Op::PushF(v, a),
			| Token::BoolLit(v, a) => Op::PushB(v, a),
			| Token::Dump(a) => Op::Dump(a),
			| Token::Plus(a) => Op::Add(a, a),
			| Token::Minus(a) => Op::Sub(a, a),
			| Token::Star(a) => Op::Mul(a, a),
			| Token::Slash(a) => Op::Div(a, a),
			| Token::Modulo(a) => Op::Mod(a, a),
			| Token::DoublePlus(a) => Op::Increment(a),
			| Token::DoubleMinus(a) => Op::Decrement(a),
			| Token::Swap(a) => Op::Swap(a),
			| Token::Drop(n, a) => Op::Drop(n, a),
			| Token::Over(n, a) => Op::Over(n, a),
			| Token::Dup(n, a) => Op::Dup(n, a),
			| Token::If(a) => Op::If(0, a),
			| Token::Then(a) => Op::Then(0, false, a),
			| Token::Else(a) => Op::Else(0, a),
			| Token::End(a) => Op::End(0, false, a),
			| Token::While(a) => Op::While(0, a),
			| Token::Do(a) => Op::Do(0, a),
			| Token::Eq(a) => Op::Eq(a, a),
			| Token::Neq(a) => Op::Neq(a, a),
			| Token::Lt(a) => Op::Lt(a, a),
			| Token::Gt(a) => Op::Gt(a, a),
			| Token::Lte(a) => Op::Lte(a, a),
			| Token::Gte(a) => Op::Gte(a, a),
			| Token::Syscall(syscode, argc, a) => Op::Syscall(syscode, argc, a),
			| Token::StringLit(lit, a) => Op::PushStr(lit, a),
			| Token::Argc(a) => Op::Argc(a),
			| Token::Argv(a) => Op::Argv(a),
			| Token::Deref(a) => Op::Deref(a),
		}
	}

	pub fn get_annot(&self) -> &Annotation<'a> {
		match self {
			| Op::PushI(_, annot)
			| Op::PushF(_, annot)
			| Op::PushB(_, annot)
			| Op::PushStr(_, annot)
			| Op::Dump(annot)
			| Op::Add(_, annot)
			| Op::Sub(_, annot)
			| Op::Mul(_, annot)
			| Op::Div(_, annot)
			| Op::Mod(_, annot)
			| Op::Increment(annot)
			| Op::Decrement(annot)
			| Op::Swap(annot)
			| Op::Drop(_, annot)
			| Op::Over(_, annot)
			| Op::Dup(_, annot)
			| Op::If(_, annot)
			| Op::Then(.., annot)
			| Op::Else(_, annot)
			| Op::End(.., annot)
			| Op::While(_, annot)
			| Op::Eq(_, annot)
			| Op::Do(_, annot)
			| Op::Neq(_, annot)
			| Op::Lt(_, annot)
			| Op::Gt(_, annot)
			| Op::Lte(_, annot)
			| Op::Gte(_, annot)
			| Op::Syscall(.., annot)
			| Op::Deref(annot)
			| Op::Argc(annot)
			| Op::Argv(annot) => annot,
			| Op::Nop => unreachable!(),
		}
	}

	pub fn expected_args(&self) -> &[&str] {
		match self {
			| Op::Drop(..)
			| Op::Over(..)
			| Op::Dup(..)
			| Op::Then(..)
			| Op::Else(..)
			| Op::End(..)
			| Op::While(..)
			| Op::Do(..)
			| Op::PushI(..)
			| Op::PushF(..)
			| Op::PushB(..)
			| Op::Nop
			| Op::If(..)
			| Op::PushStr(..)
			| Op::Syscall(..)
			| Op::Argc(..)
			| Op::Argv(..) => unreachable!(),
			| Op::Increment(_) | Op::Decrement(_) | Op::Dump(_) => &["_"],
			| Op::Swap(_)
			| Op::Mod(..)
			| Op::Add(..)
			| Op::Sub(..)
			| Op::Mul(..)
			| Op::Div(..)
			| Op::Eq(..)
			| Op::Neq(..)
			| Op::Lt(..)
			| Op::Gt(..)
			| Op::Lte(..)
			| Op::Gte(..) => &["_", "_"],
			| Op::Deref(..) => &["Ptr"],
		}
	}
}

impl Display for Op<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			| Op::PushI(v, _) => write!(f, "PushI({})", v),
			| Op::PushF(v, _) => write!(f, "PushF({})", v),
			| Op::PushB(v, _) => write!(f, "PushB({})", v),
			| Op::Dump(_) => write!(f, "Dump"),
			| Op::Add(..) => write!(f, "Add"),
			| Op::Sub(..) => write!(f, "Sub"),
			| Op::Mul(..) => write!(f, "Mul"),
			| Op::Div(..) => write!(f, "Div"),
			| Op::Mod(..) => write!(f, "Mod"),
			| Op::Increment(_) => write!(f, "Increment"),
			| Op::Decrement(_) => write!(f, "Decrement"),
			| Op::Drop(n, _) => write!(f, "Drop{}", n),
			| Op::Swap(_) => write!(f, "Swap"),
			| Op::Over(n, _) => write!(f, "Over{}", n),
			| Op::Dup(n, _) => write!(f, "Dup{}", n),
			| Op::If(..) => write!(f, "If"),
			| Op::Then(..) => write!(f, "Then"),
			| Op::Else(..) => write!(f, "Else"),
			| Op::End(..) => write!(f, "End"),
			| Op::While(..) => write!(f, "While"),
			| Op::Do(..) => write!(f, "Do"),
			| Op::Nop => unreachable!(),
			| Op::Eq(..) => write!(f, "Eq"),
			| Op::Neq(..) => write!(f, "Neq"),
			| Op::Lt(..) => write!(f, "Lt"),
			| Op::Gt(..) => write!(f, "Gt"),
			| Op::Lte(..) => write!(f, "Lte"),
			| Op::Gte(..) => write!(f, "Gte"),
			| Op::Syscall(syscode, ..) => write!(f, "Syscall({})", syscode),
			| Op::PushStr(lit, _) => write!(f, "PushStr({})", lit),
			| Op::Argc(_) => write!(f, "Argc"),
			| Op::Argv(_) => write!(f, "Argv"),
			| Op::Deref(..) => write!(f, "Deref"),
		}
	}
}

#[derive(Clone)]
pub struct Program<'a> {
	pub reporter: Reporter,
	pub ops:      Vec<Op<'a>>,
	pub strings:  Vec<String>,
}

impl<'a> Program<'a> {
	pub fn new(mut lexer: Lexer<'a>) -> Self {
		Self {
			ops:      (&mut lexer).map(Op::from_token).collect(),
			reporter: lexer.reporter,
			strings:  lexer.strings,
		}
	}

	pub fn add_error(&mut self, msg: String) -> &mut Self {
		self.reporter.add_error(msg);
		self
	}

	pub fn add_info(&mut self, msg: String) -> &mut Self {
		self.reporter.add_info(msg);
		self
	}

	pub fn exit(&mut self, code: i32) -> ! { self.reporter.exit(code) }
}

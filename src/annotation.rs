use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Position {
	file_path: String,
	line:      usize,
	col:       usize,
}

impl Position {
	pub fn new(file_path: String, line: usize, col: usize) -> Self {
		Position { file_path, line, col }
	}
}

impl Display for Position {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[{}:{}:{}]", self.file_path, self.line, self.col)
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Type {
	I64,
	F64,
	Bool,
	Ptr,
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			| Type::I64 => write!(f, "i64"),
			| Type::F64 => write!(f, "f64"),
			| Type::Bool => write!(f, "bool"),
			| Type::Ptr => write!(f, "ptr"),
		}
	}
}

#[derive(Clone)]
pub struct Annotation {
	pos:     Position,
	pub typ: Type,
}

impl Annotation {
	pub fn new(pos: Position) -> Self { Annotation { pos, typ: Type::I64 } }

	pub fn get_pos(&self) -> &Position { &self.pos }

	pub fn with_type(mut self, typ: Type) -> Self {
		self.typ = typ;
		self
	}

	pub fn get_type(&self) -> &Type { &self.typ }

	pub fn set_type(&mut self, typ: Type) { self.typ = typ }
}

impl Display for Annotation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: {}", self.pos, self.typ)
	}
}

impl Eq for Annotation {}
impl PartialEq for Annotation {
	fn eq(&self, other: &Self) -> bool { self.typ == other.typ }
}

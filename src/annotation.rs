use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub struct Position<'a> {
	file_path: &'a String,
	line:      usize,
	col:       usize,
}

impl<'a> Position<'a> {
	pub fn new(file_path: &'a String, line: usize, col: usize) -> Self {
		Position { file_path, line, col }
	}
}

impl Display for Position<'_> {
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

#[derive(Clone, Copy)]
pub struct Annotation<'a> {
	pos:     Position<'a>,
	pub typ: Option<Type>,
}

impl<'a> Annotation<'a> {
	pub fn new(pos: Position<'a>) -> Self { Annotation { pos, typ: None } }

	pub fn get_pos(&self) -> &Position { &self.pos }

	pub fn with_type(mut self, typ: Type) -> Self {
		self.typ = Some(typ);
		self
	}

	pub fn get_type(&self) -> Option<&Type> { self.typ.as_ref() }

	pub fn set_type(&mut self, typ: Type) { self.typ = Some(typ) }

	pub fn no_annot(&self) -> Self { Self { pos: self.pos, typ: None } }
}

impl Display for Annotation<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let typ = if let Some(typ) = self.typ {
			format!("{}", typ)
		} else {
			"None".to_string()
		};
		write!(f, "{}: {}", self.pos, typ)
	}
}

impl Eq for Annotation<'_> {}
impl PartialEq for Annotation<'_> {
	fn eq(&self, other: &Self) -> bool { self.typ == other.typ }
}

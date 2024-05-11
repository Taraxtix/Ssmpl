//#region Imports
use std::fmt::Display;

use crate::{
	annotation::{Annotation, Type},
	parser::{Op, OpType, Program},
};
//#endregion

pub struct Stack {
	stack: Vec<Annotation>,
}

impl Display for Stack {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut stack = self.stack.clone();

		writeln!(f, "[")?;
		while !stack.is_empty() {
			writeln!(f, "\t{}", stack.pop().unwrap())?;
		}
		write!(f, "]")
	}
}

impl Stack {
	pub fn from_vec(stack: Vec<Annotation>) -> Self { Stack { stack } }
}

impl Eq for Stack {}
impl PartialEq for Stack {
	fn eq(&self, other: &Self) -> bool { self.stack == other.stack }
}

impl Program {
	const ALLOWED_IMPLICIT_CAST: [(Type, Type); 6] = [
		(Type::I64, Type::F64),
		(Type::I64, Type::Bool),
		(Type::I64, Type::Ptr),
		(Type::Bool, Type::I64),
		(Type::Bool, Type::F64),
		(Type::Ptr, Type::I64),
	];

	pub fn type_check(mut self) -> Self {
		let mut stack: Vec<Annotation> = vec![];
		let mut cf: Vec<&mut OpType> = vec![];
		let mut stack_snapshots: Vec<Vec<Annotation>> = vec![];
		let mut if_else_count = 0;
		let mut while_do_count = 0;
		let mut ops = self.ops.clone();

		use OpType::*;

		ops.iter_mut().for_each(|op| {
			let debug_op = op.clone();
			let Op { typ, annot } = op;
			match typ {
				| PushI(_) => {
					annot.set_type(Type::I64);
					stack.push(annot.clone());
				}
				| PushF(_) => {
					annot.set_type(Type::F64);
					stack.push(annot.clone());
				}
				| PushB(_) => {
					annot.set_type(Type::Bool);
					stack.push(annot.clone());
				}
				| PushStr(_) => stack.push(annot.clone().with_type(Type::Ptr)),
				| Dump(typ) => {
					match stack.pop() {
						| Some(a) => *typ = *a.get_type().unwrap(),
						| None => {
							self.add_error(format!(
								"{}: `Dump` cannot be called on an empty stack",
								annot.get_pos()
							))
							.exit(1)
						}
					}
				}
				| Add(type1, type2) | Sub(type1, type2) => {
					match (stack.pop(), stack.pop()) {
						| (Some(a), Some(b)) => {
							let a_typ = a.get_type().unwrap();
							let b_typ = b.get_type().unwrap();
							if a_typ == &Type::F64 || b_typ == &Type::F64 {
								self.check_implicit_conversion(&a, &Type::F64);
								self.check_implicit_conversion(&b, &Type::F64);
								stack.push(annot.clone().with_type(Type::F64));
							} else if a_typ == &Type::Ptr {
								self.check_implicit_conversion(&b, &Type::I64);
								stack.push(annot.clone().with_type(Type::Ptr));
							} else if b_typ == &Type::Ptr {
								self.check_implicit_conversion(&a, &Type::I64);
								stack.push(annot.clone().with_type(Type::Ptr));
							} else {
								self.check_implicit_conversion(&a, &Type::I64);
								self.check_implicit_conversion(&b, &Type::I64);
								stack.push(annot.clone().with_type(Type::I64));
							}
							*type1 = *a_typ;
							*type2 = *b_typ;
						}
						| (a, b) => self.wrong_arg(&[a, b], debug_op, stack.clone()),
					}
				}
				| Mul(type1, type2) | Div(type1, type2) => {
					match (stack.pop(), stack.pop()) {
						| (Some(a), Some(b)) => {
							let a_typ = a.get_type().unwrap();
							let b_typ = b.get_type().unwrap();
							if a_typ == &Type::F64 || b_typ == &Type::F64 {
								self.check_implicit_conversion(&a, &Type::F64);
								self.check_implicit_conversion(&b, &Type::F64);
								stack.push(annot.clone().with_type(Type::F64));
							} else {
								self.check_implicit_conversion(&a, &Type::I64);
								self.check_implicit_conversion(&b, &Type::I64);
								stack.push(annot.clone().with_type(Type::I64));
							}
							*type1 = *a_typ;
							*type2 = *b_typ;
						}
						| (a, b) => self.wrong_arg(&[a, b], debug_op, stack.clone()),
					}
				}
				| Mod(type1, type2) => {
					match (stack.pop(), stack.pop()) {
						| (Some(a), Some(b)) => {
							let a_typ = a.get_type().unwrap();
							let b_typ = b.get_type().unwrap();
							self.check_implicit_conversion(&a, &Type::I64);
							self.check_implicit_conversion(&b, &Type::I64);
							*type1 = *a_typ;
							*type2 = *b_typ;
							stack.push(annot.clone().with_type(Type::I64));
						}
						| (a, b) => self.wrong_arg(&[a, b], debug_op, stack.clone()),
					}
				}
				| Increment(typ) | Decrement(typ) => {
					match stack.pop() {
						| Some(a) => {
							match a.get_type().unwrap() {
								| Type::F64 => {
									stack.push(annot.clone().with_type(Type::F64));
									*typ = Type::F64
								}
								| Type::Ptr => {
									stack.push(annot.clone().with_type(Type::Ptr));
									*typ = Type::Ptr
								}
								| a_typ => {
									self.check_implicit_conversion(&a, &Type::I64);
									stack.push(annot.clone().with_type(Type::I64));
									*typ = *a_typ
								}
							}
						}
						| None => self.wrong_arg(&[None], debug_op, stack.clone()),
					}
				}
				| Drop(n) => {
					if stack.len() < *n as usize {
						self.add_error(format!(
							"Cannot drop more elements than there are in the stack.\n \
							 Tried to drop {} elements, but there's only {} available \
							 on the stack",
							n,
							stack.len()
						))
						.exit(1);
					}
					for _ in 0..*n {
						let _ = stack.pop();
					}
				}
				| Swap => {
					match (stack.pop(), stack.pop()) {
						| (Some(a), Some(b)) => {
							stack.push(a);
							stack.push(b);
						}
						| (a, b) => self.wrong_arg(&[a, b], debug_op, stack.clone()),
					}
				}
				| Over(n) => {
					if stack.len() < *n as usize + 1 {
						self.add_error(format!(
							"Cannot get over {} elements because there's only {} \
							 available on the stack",
							n,
							stack.len()
						))
						.exit(1);
					}
					let typ = *stack[stack.len() - *n as usize - 1].get_type().unwrap();
					stack.push(annot.clone().with_type(typ));
				}
				| Dup(n) => {
					if stack.len() < *n as usize {
						self.add_error(format!(
							"Cannot copy {} element because there's only {} available \
							 on the stack",
							n,
							stack.len()
						))
						.exit(1);
					}
					for _ in 0..*n {
						stack.push(annot.clone().with_type(
							*stack[stack.len() - *n as usize].get_type().unwrap(),
						));
					}
				}
				| If(label_count) => {
					*label_count = if_else_count;
					if_else_count += 1;
					stack_snapshots.push(stack.clone());
					cf.push(typ);
				}
				| Then(label_count, _) => {
					if let Some(If(if_label_count)) = cf.pop() {
						*label_count = *if_label_count;
						let stack_snapshot = stack_snapshots.last_mut().unwrap();
						match stack.pop() {
							| Some(a) => {
								self.check_implicit_conversion(&a, &Type::Bool);
								if stack_snapshot.clone() != stack {
									self.add_error(format!(
										"{}: Condition between If and Then must only \
										 add one value to the stack",
										a.get_pos()
									))
									.exit(1);
								}
							}
							| None => {
								self.wrong_arg(&[None], debug_op, stack_snapshot.clone())
							}
						}
						cf.push(typ)
					} else {
						self.add_error(format!(
							"{}: Expected If before Then",
							annot.get_pos()
						))
						.exit(1);
					}
				}
				| Else(label_count) => {
					if let Some(Then(then_label_count, else_)) = cf.pop() {
						*else_ = true;
						*label_count = *then_label_count;
						let stack_snapshot = stack_snapshots.pop().unwrap();
						stack_snapshots.push(stack.clone());
						stack = stack_snapshot;
					}
					cf.push(typ)
				}
				| End(label_count, while_) => {
					match cf.pop() {
						| Some(Then(then_label_count, ..)) => {
							*label_count = *then_label_count;
							let stack_snapshot = stack_snapshots.pop().unwrap();
							if stack_snapshot != stack {
								self.add_error(format!(
									"{}: The code inside a IF ... THEN ... END block \
									 should not alter the stack\nBefore: {}\nAfter: {}",
									annot.get_pos(),
									Stack::from_vec(stack_snapshot),
									Stack::from_vec(stack.clone())
								))
								.exit(1);
							}
						}
						| Some(Else(else_label_count, ..)) => {
							*label_count = *else_label_count;
							let stack_snapshot = stack_snapshots.pop().unwrap();
							if stack_snapshot != stack {
								self.add_error(format!(
									"{}: code inside both of IF ... THEN ... ELSE ... \
									 END blocks should alter the stack in the same \
									 way\nThen: {}\nElse: {}",
									annot.get_pos(),
									Stack::from_vec(stack_snapshot),
									Stack::from_vec(stack.clone())
								))
								.exit(1);
							}
						}
						| Some(Do(do_label_count, ..)) => {
							*label_count = *do_label_count;
							*while_ = true;
							let stack_snapshot = stack_snapshots.pop().unwrap();
							if stack_snapshot != stack {
								self.add_error(format!(
									"{}: code inside of WHILE ... DO ... END block \
									 should not alter the stack\nBefore: {}\nAfter: {}",
									annot.get_pos(),
									Stack::from_vec(stack_snapshot),
									Stack::from_vec(stack.clone())
								))
								.exit(1);
							}
						}
						| _ => {
							self.add_error(format!(
								"{}: Expected Then or Else before End",
								annot.get_pos()
							))
							.exit(1)
						}
					}
				}
				| While(label_count) => {
					*label_count = while_do_count;
					while_do_count += 1;
					stack_snapshots.push(stack.clone());
					cf.push(typ);
				}
				| Do(label_count) => {
					if let Some(While(while_label_count)) = cf.pop() {
						*label_count = *while_label_count;
						let stack_snapshot = stack_snapshots.last_mut().unwrap();
						match stack.pop() {
							| Some(a) => {
								self.check_implicit_conversion(&a, &Type::Bool);
								if stack_snapshot.clone() != stack {
									self.add_error(format!(
										"{}: Condition between While and Do must only \
										 add one value to the stack",
										a.get_pos()
									))
									.exit(1);
								}
							}
							| None => {
								self.wrong_arg(&[None], debug_op, stack_snapshot.clone())
							}
						}
						cf.push(typ)
					} else {
						self.add_error(format!(
							"{}: Expected While before Do",
							annot.get_pos()
						))
						.exit(1);
					}
				}
				| Eq(type_l, type_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], debug_op, stack.clone());
					}
					let stack1 = stack.pop().unwrap();
					let stack2 = stack.pop().unwrap();
					let a_typ = stack1.get_type().unwrap();
					let b_typ = stack2.get_type().unwrap();
					if a_typ == b_typ {
					} else if a_typ == &Type::F64 || b_typ == &Type::F64 {
						self.check_implicit_conversion(&stack1, &Type::F64);
						self.check_implicit_conversion(&stack2, &Type::F64);
					} else {
						self.check_implicit_conversion(&stack1, &Type::I64);
						self.check_implicit_conversion(&stack2, &Type::I64);
					}
					*type_l = *b_typ;
					*type_r = *a_typ;
					stack.push(annot.clone().with_type(Type::Bool))
				}
				| Neq(type_l, type_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], debug_op, stack.clone());
					}
					let stack1 = stack.pop().unwrap();
					let stack2 = stack.pop().unwrap();
					let a_typ = stack1.get_type().unwrap();
					let b_typ = stack2.get_type().unwrap();
					if a_typ == b_typ {
					} else if a_typ == &Type::F64 || b_typ == &Type::F64 {
						self.check_implicit_conversion(&stack1, &Type::F64);
						self.check_implicit_conversion(&stack2, &Type::F64);
					} else {
						self.check_implicit_conversion(&stack1, &Type::I64);
						self.check_implicit_conversion(&stack2, &Type::I64);
					}
					*type_l = *b_typ;
					*type_r = *a_typ;
					stack.push(annot.clone().with_type(Type::Bool))
				}
				| Lt(type_l, type_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], debug_op, stack.clone());
					}
					let stack1 = stack.pop().unwrap();
					let stack2 = stack.pop().unwrap();
					let a_typ = stack1.get_type().unwrap();
					let b_typ = stack2.get_type().unwrap();
					if a_typ == b_typ {
					} else if a_typ == &Type::F64 || b_typ == &Type::F64 {
						self.check_implicit_conversion(&stack1, &Type::F64);
						self.check_implicit_conversion(&stack2, &Type::F64);
					} else {
						self.check_implicit_conversion(&stack1, &Type::I64);
						self.check_implicit_conversion(&stack2, &Type::I64);
					}
					*type_l = *b_typ;
					*type_r = *a_typ;
					stack.push(annot.clone().with_type(Type::Bool))
				}
				| Gt(type_l, type_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], debug_op, stack.clone());
					}
					let stack1 = stack.pop().unwrap();
					let stack2 = stack.pop().unwrap();
					let a_typ = stack1.get_type().unwrap();
					let b_typ = stack2.get_type().unwrap();
					if a_typ == b_typ {
					} else if a_typ == &Type::F64 || b_typ == &Type::F64 {
						self.check_implicit_conversion(&stack1, &Type::F64);
						self.check_implicit_conversion(&stack2, &Type::F64);
					} else {
						self.check_implicit_conversion(&stack1, &Type::I64);
						self.check_implicit_conversion(&stack2, &Type::I64);
					}
					*type_l = *b_typ;
					*type_r = *a_typ;
					stack.push(annot.clone().with_type(Type::Bool))
				}
				| Lte(type_l, type_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], debug_op, stack.clone());
					}
					let stack1 = stack.pop().unwrap();
					let stack2 = stack.pop().unwrap();
					let a_typ = stack1.get_type().unwrap();
					let b_typ = stack2.get_type().unwrap();
					if a_typ == b_typ {
					} else if a_typ == &Type::F64 || b_typ == &Type::F64 {
						self.check_implicit_conversion(&stack1, &Type::F64);
						self.check_implicit_conversion(&stack2, &Type::F64);
					} else {
						self.check_implicit_conversion(&stack1, &Type::I64);
						self.check_implicit_conversion(&stack2, &Type::I64);
					}
					*type_l = *b_typ;
					*type_r = *a_typ;
					stack.push(annot.clone().with_type(Type::Bool))
				}
				| Gte(type_l, type_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], debug_op, stack.clone());
					}
					let stack1 = stack.pop().unwrap();
					let stack2 = stack.pop().unwrap();
					let a_typ = stack1.get_type().unwrap();
					let b_typ = stack2.get_type().unwrap();
					if a_typ == b_typ {
					} else if a_typ == &Type::F64 || b_typ == &Type::F64 {
						self.check_implicit_conversion(&stack1, &Type::F64);
						self.check_implicit_conversion(&stack2, &Type::F64);
					} else {
						self.check_implicit_conversion(&stack1, &Type::I64);
						self.check_implicit_conversion(&stack2, &Type::I64);
					}
					*type_l = *b_typ;
					*type_r = *a_typ;
					stack.push(annot.clone().with_type(Type::Bool))
				}
				| Syscall(_, argc) => {
					if stack.len() < *argc {
						self.reporter
							.add_error(format!(
								"{}: Wrong number of arguments for syscall: expected \
								 {}, got {}\n",
								annot.get_pos(),
								argc,
								stack.len(),
							))
							.exit(1)
					} else {
						for _ in 0..*argc {
							stack.pop();
						}
						stack.push(annot.clone().with_type(Type::I64));
					}
				}
				| Argc => stack.push(annot.clone().with_type(Type::I64)),
				| Argv => stack.push(annot.clone().with_type(Type::Ptr)),
				| Load8 | Load16 | Load32 | Load64 => {
					let stack_val = stack.pop();
					match stack_val.clone() {
						| Some(a) => {
							if let Some(Type::Ptr) = a.get_type() {
								stack.push(annot.clone().with_type(Type::I64));
							} else {
								self.wrong_arg(&[stack_val], debug_op, stack.clone())
							}
						}
						| None => self.wrong_arg(&[stack_val], debug_op, stack.clone()),
					}
				}
				| Store8 | Store16 | Store32 | Store64 => {
					let val = stack.pop();
					let ptr = stack.pop();
					match (ptr.clone(), val.clone()) {
						| (Some(a), Some(_)) => {
							if !matches!(a.get_type(), Some(Type::Ptr)) {
								self.wrong_arg(&[ptr, val], debug_op, stack.clone())
							}
						}
						| _ => self.wrong_arg(&[ptr, val], debug_op, stack.clone()),
					}
				}
				| Nop => unreachable!(),
				| Cast(typ) => {
					let stack_val = stack.last_mut().unwrap_or_else(|| {
						self.add_error(format!(
							"{}: Expected a value on the stack to cast\n",
							annot.get_pos()
						))
						.exit(1)
					});
					stack_val.set_type(*typ);
				}
			}
		});

		if !cf.is_empty() {
			self.reporter.add_error(format!(
				"Some control flow is left open at the end of the program\n{}",
				Stack::from_vec(stack.clone())
			));
		}
		if !stack.is_empty() {
			self.reporter.add_warning(format!(
				"The stack is not empty at the end of the program\n{}",
				Stack::from_vec(stack)
			));
		}
		Program { ops, reporter: self.reporter, strings: self.strings }
	}

	fn wrong_arg(
		&mut self,
		got: &[Option<Annotation>],
		op: Op,
		mut stack: Vec<Annotation>,
	) -> ! {
		let annot = op.annot.clone();
		got.iter().for_each(|ann| stack.push(ann.clone().unwrap_or(annot.no_annot())));
		let expected = op
			.expected_args()
			.iter()
			.fold(String::new(), |output, str| output + "\t" + str + "\n");
		self.reporter
			.add_error(format!(
				"{}: Not enough arguments for `{}`\nExpected: [\n{}]\nGot: {}",
				annot.get_pos(),
				op.typ,
				expected,
				Stack::from_vec(stack.clone())
			))
			.exit(1)
	}

	fn check_implicit_conversion(&mut self, from: &Annotation, to: &Type) {
		if from.get_type().unwrap() == to {
			return;
		}
		if !Self::ALLOWED_IMPLICIT_CAST.contains(&(*from.get_type().unwrap(), *to)) {
			self.add_error(format!(
				"{}: Attempting to implicitly convert from {from} to {to}",
				from.get_pos()
			));
			return;
		}
		self.reporter.add_warning(format!(
			"{}: Implicit conversion from {} to {}",
			from.get_pos(),
			from.get_type().unwrap(),
			to
		));
	}
}

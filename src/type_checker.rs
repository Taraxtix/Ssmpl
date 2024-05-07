//#region Imports
use std::fmt::Display;

use crate::{
	annotation::{Annotation, Type},
	parser::{Op, Program},
};
//#endregion

pub struct Stack<'a> {
	stack: Vec<Annotation<'a>>,
}

impl Display for Stack<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut stack = self.stack.clone();

		writeln!(f, "[")?;
		while !stack.is_empty() {
			writeln!(f, "\t{}", stack.pop().unwrap())?;
		}
		write!(f, "]")
	}
}

impl<'a> Stack<'a> {
	pub fn from_vec(stack: Vec<Annotation<'a>>) -> Self { Stack { stack } }
}

impl Eq for Stack<'_> {}
impl PartialEq for Stack<'_> {
	fn eq(&self, other: &Self) -> bool { self.stack == other.stack }
}

impl<'a> Program<'a> {
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
		let mut cf: Vec<&mut Op<'a>> = vec![];
		let mut stack_snapshots: Vec<Vec<Annotation>> = vec![];
		let mut if_else_count = 0;
		let mut while_do_count = 0;
		let mut ops = self.ops.clone();

		ops.iter_mut().for_each(|op| {
			match op {
				| Op::PushI(_, annot) => {
					annot.set_type(Type::I64);
					stack.push(*annot);
				}
				| Op::PushF(_, annot) => {
					annot.set_type(Type::F64);
					stack.push(*annot);
				}
				| Op::PushB(_, annot) => {
					annot.set_type(Type::Bool);
					stack.push(*annot);
				}
				| Op::PushStr(_, annot) => stack.push(annot.with_type(Type::Ptr)),
				| Op::Dump(annot) => {
					match stack.pop() {
						| Some(a) => annot.set_type(*a.get_type().unwrap()),
						| None => {
							self.add_error(format!(
								"{}: `Dump` cannot be called on an empty stack",
								annot.get_pos()
							))
							.exit(1)
						}
					}
				}
				| Op::Add(annot1, annot2) | Op::Sub(annot1, annot2) => {
					match (stack.pop(), stack.pop()) {
						| (Some(a), Some(b)) => {
							let a_typ = a.get_type().unwrap();
							let b_typ = b.get_type().unwrap();
							if a_typ == &Type::F64 || b_typ == &Type::F64 {
								self.check_implicit_conversion(&a, &Type::F64);
								self.check_implicit_conversion(&b, &Type::F64);
								stack.push(a.with_type(Type::F64));
							} else if a_typ == &Type::Ptr {
								self.check_implicit_conversion(&b, &Type::I64);
								stack.push(a.with_type(Type::Ptr));
							} else if b_typ == &Type::Ptr {
								self.check_implicit_conversion(&a, &Type::I64);
								stack.push(a.with_type(Type::Ptr));
							} else {
								self.check_implicit_conversion(&a, &Type::I64);
								self.check_implicit_conversion(&b, &Type::I64);
								stack.push(a.with_type(Type::I64));
							}
							annot1.set_type(*a_typ);
							annot2.set_type(*b_typ);
						}
						| (a, b) => self.wrong_arg(&[a, b], op, stack.clone()),
					}
				}
				| Op::Mul(annot1, annot2) | Op::Div(annot1, annot2) => {
					match (stack.pop(), stack.pop()) {
						| (Some(a), Some(b)) => {
							let a_typ = a.get_type().unwrap();
							let b_typ = b.get_type().unwrap();
							if a_typ == &Type::F64 || b_typ == &Type::F64 {
								self.check_implicit_conversion(&a, &Type::F64);
								self.check_implicit_conversion(&b, &Type::F64);
								stack.push(a.with_type(Type::F64));
							} else {
								self.check_implicit_conversion(&a, &Type::I64);
								self.check_implicit_conversion(&b, &Type::I64);
								stack.push(a.with_type(Type::I64));
							}
							annot1.set_type(*a_typ);
							annot2.set_type(*b_typ);
						}
						| (a, b) => self.wrong_arg(&[a, b], op, stack.clone()),
					}
				}
				| Op::Mod(annot1, annot2) => {
					match (stack.pop(), stack.pop()) {
						| (Some(a), Some(b)) => {
							let a_typ = a.get_type().unwrap();
							let b_typ = b.get_type().unwrap();
							self.check_implicit_conversion(&a, &Type::I64);
							self.check_implicit_conversion(&b, &Type::I64);
							annot1.set_type(*a_typ);
							annot2.set_type(*b_typ);
							stack.push(a.with_type(Type::I64));
						}
						| (a, b) => self.wrong_arg(&[a, b], op, stack.clone()),
					}
				}
				| Op::Increment(annot) | Op::Decrement(annot) => {
					match stack.pop() {
						| Some(a) => {
							match a.get_type().unwrap() {
								| Type::F64 => {
									stack.push(annot.with_type(Type::F64));
									annot.set_type(Type::F64)
								}
								| Type::Ptr => {
									stack.push(annot.with_type(Type::Ptr));
									annot.set_type(Type::Ptr)
								}
								| a_typ => {
									self.check_implicit_conversion(&a, &Type::I64);
									stack.push(annot.with_type(Type::I64));
									annot.set_type(*a_typ)
								}
							}
						}
						| None => self.wrong_arg(&[None], op, stack.clone()),
					}
				}
				| Op::Drop(n, _) => {
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
				| Op::Swap(_) => {
					match (stack.pop(), stack.pop()) {
						| (Some(a), Some(b)) => {
							stack.push(a);
							stack.push(b);
						}
						| (a, b) => self.wrong_arg(&[a, b], op, stack.clone()),
					}
				}
				| Op::Over(n, annot) => {
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
					stack.push(annot.with_type(typ));
				}
				| Op::Dup(n, annot) => {
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
						stack.push(annot.with_type(
							*stack[stack.len() - *n as usize].get_type().unwrap(),
						));
					}
				}
				| Op::If(label_count, _) => {
					*label_count = if_else_count;
					if_else_count += 1;
					stack_snapshots.push(stack.clone());
					cf.push(op);
				}
				| Op::Then(label_count, _, a) => {
					if let Some(Op::If(if_label_count, _)) = cf.pop() {
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
							| None => self.wrong_arg(&[None], op, stack_snapshot.clone()),
						}
						cf.push(op)
					} else {
						self.add_error(format!(
							"{}: Expected If before Then",
							a.get_pos()
						))
						.exit(1);
					}
				}
				| Op::Else(label_count, _) => {
					if let Some(Op::Then(then_label_count, else_, _)) = cf.pop() {
						*else_ = true;
						*label_count = *then_label_count;
						let stack_snapshot = stack_snapshots.pop().unwrap();
						stack_snapshots.push(stack.clone());
						stack = stack_snapshot;
					}
					cf.push(op)
				}
				| Op::End(label_count, while_, a) => {
					match cf.pop() {
						| Some(Op::Then(then_label_count, ..)) => {
							*label_count = *then_label_count;
							let stack_snapshot = stack_snapshots.pop().unwrap();
							if stack_snapshot != stack {
								self.add_error(format!(
									"{}: The code inside a IF ... THEN ... END block \
									 should not alter the stack\nBefore: {}\nAfter: {}",
									a.get_pos(),
									Stack::from_vec(stack_snapshot),
									Stack::from_vec(stack.clone())
								))
								.exit(1);
							}
						}
						| Some(Op::Else(else_label_count, ..)) => {
							*label_count = *else_label_count;
							let stack_snapshot = stack_snapshots.pop().unwrap();
							if stack_snapshot != stack {
								self.add_error(format!(
									"{}: code inside both of IF ... THEN ... ELSE ... \
									 END blocks should alter the stack in the same \
									 way\nThen: {}\nElse: {}",
									a.get_pos(),
									Stack::from_vec(stack_snapshot),
									Stack::from_vec(stack.clone())
								))
								.exit(1);
							}
						}
						| Some(Op::Do(do_label_count, ..)) => {
							*label_count = *do_label_count;
							*while_ = true;
							let stack_snapshot = stack_snapshots.pop().unwrap();
							if stack_snapshot != stack {
								self.add_error(format!(
									"{}: code inside of WHILE ... DO ... END block \
									 should not alter the stack\nBefore: {}\nAfter: {}",
									a.get_pos(),
									Stack::from_vec(stack_snapshot),
									Stack::from_vec(stack.clone())
								))
								.exit(1);
							}
						}
						| _ => {
							self.add_error(format!(
								"{}: Expected Then or Else before End",
								a.get_pos()
							))
							.exit(1)
						}
					}
				}
				| Op::While(label_count, _) => {
					*label_count = while_do_count;
					while_do_count += 1;
					stack_snapshots.push(stack.clone());
					cf.push(op);
				}
				| Op::Do(label_count, a) => {
					if let Some(Op::While(while_label_count, _)) = cf.pop() {
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
							| None => self.wrong_arg(&[None], op, stack_snapshot.clone()),
						}
						cf.push(op)
					} else {
						self.add_error(format!(
							"{}: Expected While before Do",
							a.get_pos()
						))
						.exit(1);
					}
				}
				| Op::Eq(annot_l, annot_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], op, stack.clone());
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
					annot_l.set_type(*b_typ);
					annot_r.set_type(*a_typ);
					stack.push(annot_l.with_type(Type::Bool))
				}
				| Op::Neq(annot_l, annot_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], op, stack.clone());
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
					annot_l.set_type(*b_typ);
					annot_r.set_type(*a_typ);
					stack.push(annot_l.with_type(Type::Bool))
				}
				| Op::Lt(annot_l, annot_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], op, stack.clone());
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
					annot_l.set_type(*b_typ);
					annot_r.set_type(*a_typ);
					stack.push(annot_l.with_type(Type::Bool))
				}
				| Op::Gt(annot_l, annot_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], op, stack.clone());
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
					annot_l.set_type(*b_typ);
					annot_r.set_type(*a_typ);
					stack.push(annot_l.with_type(Type::Bool))
				}
				| Op::Lte(annot_l, annot_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], op, stack.clone());
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
					annot_l.set_type(*b_typ);
					annot_r.set_type(*a_typ);
					stack.push(annot_l.with_type(Type::Bool))
				}
				| Op::Gte(annot_l, annot_r) => {
					if stack.len() < 2 {
						self.wrong_arg(&[None, None], op, stack.clone());
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
					annot_l.set_type(*b_typ);
					annot_r.set_type(*a_typ);
					stack.push(annot_l.with_type(Type::Bool))
				}
				| Op::Syscall(_, argc, annot) => {
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
						stack.push(annot.with_type(Type::I64));
					}
				}
				| Op::Argc(annot) => stack.push(annot.with_type(Type::I64)),
				| Op::Argv(annot) => stack.push(annot.with_type(Type::Ptr)),
				| Op::Deref(_) => {
					let stack_val = stack.pop();
					match stack_val {
						| Some(a) => {
							if let Some(Type::Ptr) = a.get_type() {
								stack.push(a.with_type(Type::I64));
							} else {
								self.wrong_arg(&[stack_val], op, stack.clone())
							}
						}
						| None => self.wrong_arg(&[stack_val], op, stack.clone()),
					}
				}
				| Op::Nop => unreachable!(),
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
		got: &[Option<Annotation<'a>>],
		op: &Op<'a>,
		mut stack: Vec<Annotation<'a>>,
	) -> ! {
		let annot = op.get_annot();
		got.iter().for_each(|ann| stack.push(ann.unwrap_or(annot.no_annot())));
		let expected = op
			.expected_args()
			.iter()
			.fold(String::new(), |output, str| output + "\t" + str + "\n");
		self.reporter
			.add_error(format!(
				"{}: Not enough arguments for `{}`\nExpected: [\n{}]\nGot: {}",
				annot.get_pos(),
				op,
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

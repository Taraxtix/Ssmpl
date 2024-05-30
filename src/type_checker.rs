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

impl Op {
	pub fn required_stack_len(&self) -> usize {
		match self.typ {
			| OpType::Nop
			| OpType::Argc
			| OpType::Argv
			| OpType::PushI(_)
			| OpType::PushF(_)
			| OpType::PushStr(_)
			| OpType::PushB(_)
			| OpType::If(_)
			| OpType::Else(_)
			| OpType::End(..)
			| OpType::While(_)
			| OpType::Mem(_) => 0,
			| OpType::Load8
			| OpType::Load16
			| OpType::Load32
			| OpType::Load64
			| OpType::Increment(_)
			| OpType::Decrement(_)
			| OpType::Cast(_)
			| OpType::Not
			| OpType::Then(..)
			| OpType::Do(_)
			| OpType::Dump(_) => 1,
			| OpType::Store8
			| OpType::Store16
			| OpType::Store32
			| OpType::Store64
			| OpType::Eq(..)
			| OpType::Neq(..)
			| OpType::Lt(..)
			| OpType::Gt(..)
			| OpType::Lte(..)
			| OpType::Gte(..)
			| OpType::Add(..)
			| OpType::Sub(..)
			| OpType::Mul(..)
			| OpType::Div(..)
			| OpType::Mod(..)
			| OpType::ShiftR
			| OpType::ShiftL
			| OpType::BitAnd
			| OpType::And
			| OpType::BitOr
			| OpType::Or
			| OpType::Swap => 2,
			| OpType::SetOver(size) | OpType::Over(size) => size as usize + 1,
			| OpType::Syscall(_, size) => size,
			| OpType::Drop(size) | OpType::Dup(size) => size as usize,
		}
	}
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
			self.check_args(op, &stack);
			let Op { typ, annot } = op;
			match typ {
				| Nop => unreachable!(),
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
				| Dump(typ) => *typ = *stack.pop().unwrap().get_type(),
				| Add(type1, type2) | Sub(type1, type2) => {
					let a_typ = *stack.pop().unwrap().get_type();
					let b_typ = *stack.pop().unwrap().get_type();
					if a_typ == Type::F64 || b_typ == Type::F64 {
						stack.push(annot.clone().with_type(Type::F64));
					} else if a_typ == Type::Ptr || b_typ == Type::Ptr {
						stack.push(annot.clone().with_type(Type::Ptr));
					} else {
						stack.push(annot.clone().with_type(Type::I64));
					}
					*type1 = a_typ;
					*type2 = b_typ;
				}
				| Mul(type1, type2) | Div(type1, type2) => {
					let a_typ = *stack.pop().unwrap().get_type();
					let b_typ = *stack.pop().unwrap().get_type();
					if a_typ == Type::F64 || b_typ == Type::F64 {
						stack.push(annot.clone().with_type(Type::F64));
					} else {
						stack.push(annot.clone().with_type(Type::I64));
					}
					*type1 = a_typ;
					*type2 = b_typ;
				}
				| Mod(type1, type2) => {
					*type1 = *stack.pop().unwrap().get_type();
					*type2 = *stack.pop().unwrap().get_type();
					stack.push(annot.clone().with_type(Type::I64));
				}
				| Increment(typ) | Decrement(typ) => {
					let a_typ = *stack.pop().unwrap().get_type();
					if a_typ != Type::F64 && a_typ != Type::Ptr {
						stack.push(annot.clone().with_type(Type::I64));
					} else {
						stack.push(annot.clone().with_type(a_typ));
					}
					*typ = a_typ;
				}
				| Drop(n) => {
					for _ in 0..*n {
						let _ = stack.pop();
					}
				}
				| Swap => {
					let a = stack.pop().unwrap();
					let b = stack.pop().unwrap();
					stack.push(a);
					stack.push(b);
				}
				| Over(n) => {
					let typ = *stack[stack.len() - *n as usize - 1].get_type();
					stack.push(annot.clone().with_type(typ));
				}
				| Dup(n) => {
					for _ in 0..*n {
						stack.push(
							annot
								.clone()
								.with_type(*stack[stack.len() - *n as usize].get_type()),
						);
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
						let a = stack.pop().unwrap();
						if stack_snapshot.clone() != stack {
							self.add_error(format!(
								"{}: Condition between If and Then must only add one \
								 value to the stack",
								a.get_pos()
							))
							.exit(1);
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
						let a = stack.pop().unwrap();
						if stack_snapshot.clone() != stack {
							self.add_error(format!(
								"{}: Condition between While and Do must only add one \
								 value to the stack",
								a.get_pos()
							))
							.exit(1);
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
				| Eq(type_l, type_r)
				| Neq(type_l, type_r)
				| Lt(type_l, type_r)
				| Gt(type_l, type_r)
				| Lte(type_l, type_r)
				| Gte(type_l, type_r) => {
					let a_typ = *stack.pop().unwrap().get_type();
					let b_typ = *stack.pop().unwrap().get_type();
					*type_l = b_typ;
					*type_r = a_typ;
					stack.push(annot.clone().with_type(Type::Bool))
				}
				| Syscall(_, argc) => {
					for _ in 0..*argc {
						stack.pop();
					}
					stack.push(annot.clone().with_type(Type::I64));
				}
				| Argc => stack.push(annot.clone().with_type(Type::I64)),
				| Argv => stack.push(annot.clone().with_type(Type::Ptr)),
				| Load8 | Load16 | Load32 | Load64 => {
					stack.pop();
					stack.push(annot.clone().with_type(Type::I64));
				}
				| Store8 | Store16 | Store32 | Store64 => {
					stack.pop().unwrap();
					stack.pop().unwrap();
				}
				| Cast(typ) => stack.last_mut().unwrap().set_type(*typ),
				| ShiftR | ShiftL => {
					stack.pop();
					let b_typ = *stack.pop().unwrap().get_type();
					stack.push(annot.clone().with_type(b_typ));
				}
				| BitAnd | BitOr => {
					let a_typ = *stack.pop().unwrap().get_type();
					stack.pop();
					stack.push(annot.clone().with_type(a_typ));
				}
				| And | Or => stack.push(annot.clone().with_type(Type::Bool)),
				| Not => {
					let arg_typ = *stack.pop().unwrap().get_type();
					stack.push(annot.clone().with_type(arg_typ));
				}
				| Mem(_) => stack.push(annot.clone().with_type(Type::Ptr)),
				| SetOver(size) => {
					let set_type = *stack.pop().unwrap().get_type();
					let index = stack.len() - *size as usize;
					let val = stack.get_mut(index).unwrap();
					*val = annot.clone().with_type(set_type);
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
		Program {
			ops,
			reporter: self.reporter,
			strings: self.strings,
			memory_regions: self.memory_regions,
			memory_regions_order: self.memory_regions_order,
		}
	}

	#[allow(clippy::ptr_arg)]
	pub fn check_args(&mut self, op: &Op, stack: &Vec<Annotation>) {
		if stack.len() < op.required_stack_len() {
			self.add_error(format!(
				"{} requires at least{} values on the stack but got {}",
				op,
				op.required_stack_len(),
				stack.len()
			))
			.exit(1);
		}
		let Op { typ, .. } = op;
		let mut cloned_stack = stack.clone();
		cloned_stack.reverse();
		let arg = cloned_stack.as_slice();
		match typ {
			| _ if op.required_stack_len() == 0 => (),
			| OpType::Cast(_)
			| OpType::Syscall(..)
			| OpType::Drop(_)
			| OpType::Over(_)
			| OpType::SetOver(_)
			| OpType::Dup(_)
			| OpType::Swap
			| OpType::Not
			| OpType::Dump(_) => (),
			| OpType::Add(..) | OpType::Sub(..) => {
				let a_typ = arg[0].get_type();
				let b_typ = arg[1].get_type();
				if a_typ == &Type::F64 || b_typ == &Type::F64 {
					self.check_implicit_conversion(&arg[0], &Type::F64);
					self.check_implicit_conversion(&arg[1], &Type::F64);
				} else if a_typ == &Type::Ptr {
					self.check_implicit_conversion(&arg[1], &Type::I64);
				} else if b_typ == &Type::Ptr {
					self.check_implicit_conversion(&arg[0], &Type::I64);
				} else {
					self.check_implicit_conversion(&arg[0], &Type::I64);
					self.check_implicit_conversion(&arg[1], &Type::I64);
				}
			}
			| OpType::Mul(..) | OpType::Div(..) => {
				let typ = match (arg[0].get_type(), arg[1].get_type()) {
					| (Type::F64, _) | (_, Type::F64) => Type::F64,
					| _ => Type::I64,
				};
				self.check_implicit_conversion(&arg[0], &typ);
				self.check_implicit_conversion(&arg[1], &typ);
			}
			| OpType::Mod(..) => {
				self.check_implicit_conversion(&arg[0], &Type::I64);
				self.check_implicit_conversion(&arg[1], &Type::I64);
			}
			| OpType::Increment(_) | OpType::Decrement(_) => {
				if ![Type::F64, Type::Ptr].contains(arg[0].get_type()) {
					self.check_implicit_conversion(&arg[0], &Type::I64);
				}
			}
			| OpType::Then(..) | OpType::Do(_) => {
				self.check_implicit_conversion(&arg[0], &Type::Bool)
			}
			| OpType::Eq(..)
			| OpType::Neq(..)
			| OpType::Lt(..)
			| OpType::Gt(..)
			| OpType::Lte(..)
			| OpType::Gte(..) => {
				match (arg[0].get_type(), arg[1].get_type()) {
					| (a, b) if a == b => (),
					| (Type::F64, _) | (_, Type::F64) => {
						self.check_implicit_conversion(&arg[0], &Type::F64);
						self.check_implicit_conversion(&arg[1], &Type::F64);
					}
					| _ => {
						self.check_implicit_conversion(&arg[0], &Type::I64);
						self.check_implicit_conversion(&arg[1], &Type::I64);
					}
				}
			}
			| OpType::Load8 | OpType::Load16 | OpType::Load32 | OpType::Load64 => {
				if arg[0].get_type() != &Type::Ptr {
					self.add_error(format!(
						"{op} Expected a PTR on top of the stack but got {}\n",
						arg[0]
					))
					.exit(1)
				}
			}
			| OpType::Store8 | OpType::Store16 | OpType::Store32 | OpType::Store64 => {
				if arg[1].get_type() != &Type::Ptr {
					self.add_error(format!(
						"{op} Expected a PTR on second position of the stack but got \
						 {}\n",
						arg[1]
					))
					.exit(1)
				}
			}
			| OpType::ShiftR | OpType::ShiftL => {
				if arg[0].get_type() != &Type::I64 {
					self.add_error(format!(
						"{op} Expected an I64 on top of the stack but got {}\n",
						arg[0]
					))
					.exit(1)
				}
			}
			| OpType::BitAnd | OpType::BitOr => {
				self.check_implicit_conversion(&arg[1], arg[0].get_type())
			}
			| OpType::And | OpType::Or => {
				self.check_implicit_conversion(&arg[0], &Type::Bool);
				self.check_implicit_conversion(&arg[1], &Type::Bool);
			}
			| _ => unreachable!(),
		}
	}

	fn check_implicit_conversion(&mut self, from: &Annotation, to: &Type) {
		if from.get_type() == to {
			return;
		}
		if !Self::ALLOWED_IMPLICIT_CAST.contains(&(*from.get_type(), *to)) {
			self.add_error(format!(
				"{}: Attempting to implicitly convert from {from} to {to}",
				from.get_pos()
			));
			return;
		}
		self.reporter.add_warning(format!(
			"{}: Implicit conversion from {} to {}",
			from.get_pos(),
			from.get_type(),
			to
		));
	}
}

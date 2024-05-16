//#region Imports
use std::collections::HashMap;

use crate::{
	annotation::Type,
	parser::{Op, OpType, Program},
};
//#endregion

#[derive(Clone, PartialEq)]
enum Data {
	I64(i64),
	F64(f64),
	Bool(bool),
	Ptr(i64),
}

impl Data {
	fn to_i64(&self) -> i64 {
		match self {
			| Data::Ptr(v) | Data::I64(v) => *v,
			| Data::Bool(v) => *v as i64,
			| Data::F64(v) => i64::from_ne_bytes(v.to_ne_bytes()),
		}
	}

	fn to_bool(&self) -> bool {
		match self {
			| Data::Ptr(v) | Data::I64(v) => *v != 0,
			| Data::F64(v) => *v != 0.,
			| Data::Bool(v) => *v,
		}
	}

	fn bytes_to_data_f64(&self) -> Data {
		match self {
			| Data::F64(v) => Data::F64(*v),
			| a => Data::F64(f64::from_ne_bytes(a.to_i64().to_ne_bytes())),
		}
	}
}

#[derive(Default)]
struct CFLabels {
	else_label:  i64,
	while_label: i64,
	end_label:   i64,
}

fn simulate_syscall(syscode: &usize) -> Option<i64> {
	match syscode {
		| 1 => Some(42),
		| _ => None,
	}
}

const MEM_LENGTH: usize = 1024;

impl Program {
	fn find_op_by_label(&self, label: &i64, op: &str) -> Option<(usize, &Op)> {
		self.ops.iter().enumerate().find(|(_, f_op)| {
			let Op { typ, .. } = f_op;
			match op {
				| "Then" => {
					if let OpType::Then(f_label, ..) = typ {
						if f_label == label {
							return true;
						}
					}
				}
				| "Else" => {
					if let OpType::Else(f_label, ..) = typ {
						if f_label == label {
							return true;
						}
					}
				}
				| "While" => {
					if let OpType::While(f_label, ..) = typ {
						if f_label == label {
							return true;
						}
					}
				}
				| "Do" => {
					if let OpType::Do(f_label, ..) = typ {
						if f_label == label {
							return true;
						}
					}
				}
				| "EndIf" => {
					if let OpType::End(f_label, false, ..) = typ {
						if f_label == label {
							return true;
						}
					}
				}
				| "EndWhile" => {
					if let OpType::End(f_label, true, ..) = typ {
						if f_label == label {
							return true;
						}
					}
				}
				| _ => unreachable!(),
			}
			false
		})
	}

	#[allow(clippy::identity_op)]
	pub fn simulate(&mut self) {
		use OpType::*;

		let mut stack: Vec<Data> = vec![];
		let mut cf_labels: Vec<CFLabels> = vec![];
		let mut ip = 0;
		let mut labels_map: HashMap<i64, (usize, usize)> = HashMap::new(); // label -> (if_idx, while_idx)
		let mut memory: [u8; MEM_LENGTH] = [0; MEM_LENGTH];
		let mut strings_ptr: HashMap<String, usize> = HashMap::new();
		let mut string_end = 0;

		for lit in self.strings.iter() {
			strings_ptr.insert(lit.clone(), string_end);
			for byte in lit.as_bytes() {
				if string_end >= MEM_LENGTH {
					self.add_error("Not enough memory for strings allocation".into())
						.exit(1);
				}
				memory[string_end] = *byte;
				string_end += 1;
			}
		}

		while ip < self.ops.len() {
			let Op { typ, annot } = &self.ops[ip];
			match typ {
				| PushI(i) => stack.push(Data::I64(*i)),
				| PushB(b) => stack.push(Data::Bool(*b)),
				| PushF(f) => stack.push(Data::F64(*f)),
				| PushStr(s) => {
					stack.push(Data::Ptr(*strings_ptr.get(s).unwrap() as i64))
				}
				| Dump(_) => {
					match stack.pop().unwrap() {
						| Data::I64(i) | Data::Ptr(i) => println!("{}", i),
						| Data::F64(f) => println!("{:?}", f),
						| Data::Bool(b) => println!("{}", b),
					}
				}
				| Add(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::F64(v1), Data::I64(v2)) => {
							stack.push(Data::F64(v2 as f64 + v1))
						}
						| (Data::I64(v1), Data::F64(v2)) => {
							stack.push(Data::F64(v2 + v1 as f64))
						}
						| (Data::F64(v1), Data::F64(v2)) => {
							stack.push(Data::F64(v2 + v1))
						}
						| (a, b) => stack.push(Data::I64(b.to_i64() + a.to_i64())),
					}
				}
				| Sub(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::F64(v1), Data::I64(v2)) => {
							stack.push(Data::F64(v2 as f64 - v1))
						}
						| (Data::I64(v1), Data::F64(v2)) => {
							stack.push(Data::F64(v2 - v1 as f64))
						}
						| (Data::F64(v1), Data::F64(v2)) => {
							stack.push(Data::F64(v2 - v1))
						}
						| (a, b) => stack.push(Data::I64(b.to_i64() - a.to_i64())),
					}
				}
				| Mul(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::F64(v1), Data::I64(v2)) => {
							stack.push(Data::F64(v2 as f64 * v1))
						}
						| (Data::I64(v1), Data::F64(v2)) => {
							stack.push(Data::F64(v2 * v1 as f64))
						}
						| (Data::F64(v1), Data::F64(v2)) => {
							stack.push(Data::F64(v2 * v1))
						}
						| (a, b) => stack.push(Data::I64(b.to_i64() * a.to_i64())),
					}
				}
				| Div(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::F64(v1), Data::I64(v2)) => {
							stack.push(Data::F64(v2 as f64 / v1))
						}
						| (Data::I64(v1), Data::F64(v2)) => {
							stack.push(Data::F64(v2 / v1 as f64))
						}
						| (Data::F64(v1), Data::F64(v2)) => {
							stack.push(Data::F64(v2 / v1))
						}
						| (a, b) => stack.push(Data::I64(b.to_i64() / a.to_i64())),
					}
				}
				| Increment(_) => {
					match stack.pop().unwrap() {
						| Data::F64(f) => stack.push(Data::F64(f + 1.)),
						| a => stack.push(Data::I64(a.to_i64() + 1)),
					}
				}
				| Decrement(_) => {
					match stack.pop().unwrap() {
						| Data::F64(f) => stack.push(Data::F64(f - 1.)),
						| a => stack.push(Data::I64(a.to_i64() - 1)),
					}
				}
				| Mod(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (a, b) => {
							stack.push(Data::I64(b.to_i64().rem_euclid(a.to_i64())))
						}
					}
				}
				| Drop(n) => {
					for _ in 0..*n {
						stack.pop();
					}
				}
				| Swap => {
					let a = stack.pop().unwrap();
					let b = stack.pop().unwrap();
					stack.push(a);
					stack.push(b);
				}
				| Over(n) => stack.push(stack[stack.len() - *n as usize - 1].clone()),
				| Dup(n) => {
					for _ in 0..*n {
						stack.push(stack[stack.len() - *n as usize].clone());
					}
				}
				| If(label_count) => {
					let if_label = cf_labels.len();
					match labels_map.get_mut(label_count) {
						| Some((curr, _)) => {
							*curr = if_label;
						}
						| None => {
							labels_map.insert(*label_count, (if_label, 0));
						}
					}
					cf_labels.push(CFLabels::default());
					let (else_label, _) = self
						.find_op_by_label(label_count, "Else")
						.unwrap_or((0, &Op { typ: Nop, annot: annot.clone() }));
					let (end_label, _) =
						self.find_op_by_label(label_count, "EndIf").unwrap();
					cf_labels.as_mut_slice()[if_label].else_label = else_label as i64;
					cf_labels.as_mut_slice()[if_label].end_label = end_label as i64;
				}
				| While(label_count) => {
					let while_label = cf_labels.len();
					match labels_map.get_mut(label_count) {
						| Some((curr, _)) => {
							*curr = while_label;
						}
						| None => {
							labels_map.insert(*label_count, (while_label, 0));
						}
					}
					cf_labels.push(CFLabels::default());
					let (end_label, _) =
						self.find_op_by_label(label_count, "EndWhile").unwrap();
					cf_labels.as_mut_slice()[*label_count as usize].while_label =
						ip as i64;
					cf_labels.as_mut_slice()[*label_count as usize].end_label =
						end_label as i64;
				}
				| Then(label_count, else_) => {
					let (idx, _) = labels_map.get(label_count).unwrap();
					match stack.pop().unwrap() {
						| Data::Bool(false)
						| Data::I64(0)
						| Data::F64(0.0)
						| Data::Ptr(0) => {
							if *else_ {
								ip = cf_labels[*idx].else_label as usize;
							} else {
								ip = cf_labels[*idx].end_label as usize;
							}
						}
						| _ => (),
					}
				}
				| Else(label_count, ..) => {
					let (idx, _) = labels_map.get(label_count).unwrap();
					ip = cf_labels[*idx].end_label as usize
				}
				| End(label_count, while_, ..) => {
					let (_, idx) = labels_map.get(label_count).unwrap();
					if *while_ {
						ip = cf_labels[*idx].while_label as usize - 1;
					}
				}
				| Do(label_count, ..) => {
					let (_, idx) = labels_map.get(label_count).unwrap();
					match stack.pop().unwrap() {
						| Data::Bool(false)
						| Data::I64(0)
						| Data::F64(0.0)
						| Data::Ptr(0) => ip = cf_labels[*idx].end_label as usize,
						| _ => (),
					}
				}
				| Eq(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::F64(val_r), Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l == val_r))
						}
						| (a, Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l == a.to_i64() as f64))
						}
						| (Data::F64(val_r), b) => {
							stack.push(Data::Bool(b.to_i64() as f64 == val_r))
						}
						| (a, b) => stack.push(Data::Bool(b.to_i64() == a.to_i64())),
					}
				}
				| Neq(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::F64(val_r), Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l != val_r))
						}
						| (a, Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l != a.to_i64() as f64))
						}
						| (Data::F64(val_r), b) => {
							stack.push(Data::Bool(b.to_i64() as f64 != val_r))
						}
						| (a, b) => stack.push(Data::Bool(b.to_i64() != a.to_i64())),
					}
				}
				| Lt(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::F64(val_r), Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l < val_r))
						}
						| (a, Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l < a.to_i64() as f64))
						}
						| (Data::F64(val_r), b) => {
							stack.push(Data::Bool((b.to_i64() as f64) < val_r))
						}
						| (a, b) => stack.push(Data::Bool(b.to_i64() < a.to_i64())),
					}
				}
				| Gt(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::F64(val_r), Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l > val_r))
						}
						| (a, Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l > a.to_i64() as f64))
						}
						| (Data::F64(val_r), b) => {
							stack.push(Data::Bool(b.to_i64() as f64 > val_r))
						}
						| (a, b) => stack.push(Data::Bool(b.to_i64() > a.to_i64())),
					}
				}
				| Lte(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::F64(val_r), Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l <= val_r))
						}
						| (a, Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l <= a.to_i64() as f64))
						}
						| (Data::F64(val_r), b) => {
							stack.push(Data::Bool(b.to_i64() as f64 <= val_r))
						}
						| (a, b) => stack.push(Data::Bool(b.to_i64() <= a.to_i64())),
					}
				}
				| Gte(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::F64(val_r), Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l >= val_r))
						}
						| (a, Data::F64(val_l)) => {
							stack.push(Data::Bool(val_l >= a.to_i64() as f64))
						}
						| (Data::F64(val_r), b) => {
							stack.push(Data::Bool(b.to_i64() as f64 >= val_r))
						}
						| (a, b) => stack.push(Data::Bool(b.to_i64() >= a.to_i64())),
					}
				}
				| Syscall(syscode, _argc) => {
					match simulate_syscall(syscode) {
						| Some(val) => stack.push(Data::I64(val)),
						| None => {
							self.add_error(format!(
								"{}: Syscall {} not implemented",
								annot.get_pos(),
								syscode
							))
							.exit(1)
						}
					}
				}
				| Argc | Argv => {
					self.add_error(
						"Program Argument is not supported in simulation mode".into(),
					)
					.exit(1)
				}
				| Load8 => {
					let ptr = stack.pop().unwrap().to_i64() as usize;
					stack.push(Data::I64(memory[ptr] as i64))
				}
				| Load16 => {
					let ptr = stack.pop().unwrap().to_i64() as usize;
					let bytes: [u8; 2] = memory[ptr..ptr + 2].try_into().unwrap();
					stack.push(Data::I64(u16::from_ne_bytes(bytes) as i64))
				}
				| Load32 => {
					let ptr = stack.pop().unwrap().to_i64() as usize;
					let bytes: [u8; 4] = memory[ptr..ptr + 4].try_into().unwrap();
					stack.push(Data::I64(u32::from_ne_bytes(bytes) as i64))
				}
				| Load64 => {
					let ptr = stack.pop().unwrap().to_i64() as usize;
					let bytes: [u8; 8] = memory[ptr..ptr + 8].try_into().unwrap();
					stack.push(Data::I64(u64::from_ne_bytes(bytes) as i64))
				}
				| Store8 => {
					let val = stack.pop().unwrap().to_i64();
					let ptr = stack.pop().unwrap().to_i64() as usize;
					memory[ptr] = (val & 0xFF) as u8
				}
				| Store16 => {
					let val = stack.pop().unwrap().to_i64();
					let ptr = stack.pop().unwrap().to_i64() as usize;
					memory[ptr + 0] = (val & 0xFF00) as u8;
					memory[ptr + 1] = (val & 0x00FF) as u8;
				}
				| Store32 => {
					let val = stack.pop().unwrap().to_i64();
					let ptr = stack.pop().unwrap().to_i64() as usize;
					for i in 0..4 {
						memory[ptr + i] = (val & (0xFF << (3 - i))) as u8
					}
				}
				| Store64 => {
					let val = stack.pop().unwrap().to_i64();
					let ptr = stack.pop().unwrap().to_i64() as usize;
					for i in 0..8 {
						memory[ptr + i] = (val & (0xFF << (7 - i))) as u8
					}
				}
				| Cast(typ) => {
					let stack_val = stack.pop().unwrap();
					stack.push(match typ {
						| Type::I64 => Data::I64(stack_val.to_i64()),
						| Type::F64 => {
							Data::F64(f64::from_ne_bytes(
								stack_val.to_i64().to_ne_bytes(),
							))
						}
						| Type::Bool => Data::Bool(stack_val.to_i64() != 0),
						| Type::Ptr => Data::Ptr(stack_val.to_i64()),
					})
				}
				| ShiftR => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::I64(v1), Data::I64(v2)) => {
							stack.push(Data::I64(v2 >> v1))
						}
						| (Data::I64(v1), Data::F64(v2)) => {
							stack.push(
								Data::I64(Data::F64(v2).to_i64() >> v1)
									.bytes_to_data_f64(),
							)
						}
						| (Data::I64(v1), Data::Bool(v2)) => {
							stack.push(Data::Bool((v2 as i64 >> v1) != 0))
						}
						| (Data::I64(v1), Data::Ptr(v2)) => {
							stack.push(Data::Ptr(v2 >> v1))
						}
						| _ => unreachable!("Prevented by type check"),
					}
				}
				| ShiftL => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (Data::I64(v1), Data::I64(v2)) => {
							stack.push(Data::I64(v2 << v1))
						}
						| (Data::I64(v1), Data::F64(v2)) => {
							stack.push(
								Data::I64(Data::F64(v2).to_i64() << v1)
									.bytes_to_data_f64(),
							)
						}
						| (Data::I64(v1), Data::Bool(v2)) => {
							stack.push(Data::Bool(((v2 as i64) << v1) != 0))
						}
						| (Data::I64(v1), Data::Ptr(v2)) => {
							stack.push(Data::Ptr(v2 << v1))
						}
						| _ => unreachable!("Prevented by type check"),
					}
				}
				| BitAnd => {
					let v2 = stack.pop().unwrap().to_i64();
					let v1 = stack.pop().unwrap().to_i64();
					stack.push(Data::I64(v1 & v2))
				}
				| BitOr => {
					let v2 = stack.pop().unwrap().to_i64();
					let v1 = stack.pop().unwrap().to_i64();
					stack.push(Data::I64(v1 | v2))
				}
				| And => {
					let v2 = stack.pop().unwrap().to_bool();
					let v1 = stack.pop().unwrap().to_bool();
					stack.push(Data::Bool(v1 && v2))
				}
				| Or => {
					let v2 = stack.pop().unwrap().to_bool();
					let v1 = stack.pop().unwrap().to_bool();
					stack.push(Data::Bool(v1 || v2))
				}
				| Not => {
					match stack.pop().unwrap() {
						| Data::I64(v) => stack.push(Data::I64(!v)),
						| Data::F64(v) => {
							stack.push(
								Data::I64(Data::F64(v).to_i64()).bytes_to_data_f64(),
							)
						}
						| Data::Bool(v) => stack.push(Data::Bool(!v)),
						| Data::Ptr(v) => stack.push(Data::Ptr(!v)),
					}
				}
				| Nop => unreachable!(),
			}
			ip += 1;
		}
	}
}

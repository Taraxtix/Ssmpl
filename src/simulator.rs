//#region Imports
use std::collections::HashMap;

use crate::parser::{Op, Program};
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
			| Data::F64(_) => unreachable!(),
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

impl Program<'_> {
	fn find_op_by_label(&self, label: &i64, op: &str) -> Option<(usize, &Op)> {
		self.ops.iter().enumerate().find(|(_, f_op)| {
			match op {
				| "Then" => {
					if let Op::Then(f_label, ..) = f_op {
						if f_label == label {
							return true;
						}
					}
				}
				| "Else" => {
					if let Op::Else(f_label, ..) = f_op {
						if f_label == label {
							return true;
						}
					}
				}
				| "While" => {
					if let Op::While(f_label, ..) = f_op {
						if f_label == label {
							return true;
						}
					}
				}
				| "Do" => {
					if let Op::Do(f_label, ..) = f_op {
						if f_label == label {
							return true;
						}
					}
				}
				| "EndIf" => {
					if let Op::End(f_label, false, ..) = f_op {
						if f_label == label {
							return true;
						}
					}
				}
				| "EndWhile" => {
					if let Op::End(f_label, true, ..) = f_op {
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

	pub fn simulate(&mut self) {
		let mut stack: Vec<Data> = vec![];
		let mut cf_labels: Vec<CFLabels> = vec![];
		let mut ip = 0;
		let mut labels_map: HashMap<i64, (usize, usize)> = HashMap::new(); // label -> (if_idx, while_idx)
		let mut memory: [u64; MEM_LENGTH] = [0; MEM_LENGTH];
		let mut strings_ptr: HashMap<String, usize> = HashMap::new();
		let mut string_end = 0;

		for lit in self.strings.iter() {
			strings_ptr.insert(lit.clone(), string_end);
			let mut value: u64 = 0;
			for byte_idx in 0..lit.as_bytes().len() {
				value = value << 8 | lit.as_bytes()[byte_idx] as u64;
				if byte_idx % 8 == 7 || byte_idx == lit.as_bytes().len() - 1 {
					if string_end == MEM_LENGTH {
						self.add_error("Not enough memory to initialize strings".into())
							.exit(1);
					}
					memory[string_end] = value;
					string_end += 1;
				}
			}
		}

		while ip < self.ops.len() {
			match &self.ops[ip] {
				| Op::PushI(i, _) => stack.push(Data::I64(*i)),
				| Op::PushB(b, _) => stack.push(Data::Bool(*b)),
				| Op::PushF(f, _) => stack.push(Data::F64(*f)),
				| Op::PushStr(s, _) => {
					stack.push(Data::Ptr(*strings_ptr.get(s).unwrap() as i64))
				}
				| Op::Dump(_) => {
					match stack.pop().unwrap() {
						| Data::I64(i) | Data::Ptr(i) => println!("{}", i),
						| Data::F64(f) => println!("{:?}", f),
						| Data::Bool(b) => println!("{}", b),
					}
				}
				| Op::Add(..) => {
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
				| Op::Sub(..) => {
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
				| Op::Mul(..) => {
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
				| Op::Div(..) => {
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
				| Op::Increment(_) => {
					match stack.pop().unwrap() {
						| Data::F64(f) => stack.push(Data::F64(f + 1.)),
						| a => stack.push(Data::I64(a.to_i64() + 1)),
					}
				}
				| Op::Decrement(_) => {
					match stack.pop().unwrap() {
						| Data::F64(f) => stack.push(Data::F64(f - 1.)),
						| a => stack.push(Data::I64(a.to_i64() - 1)),
					}
				}
				| Op::Mod(..) => {
					match (stack.pop().unwrap(), stack.pop().unwrap()) {
						| (a, b) => {
							stack.push(Data::I64(b.to_i64().rem_euclid(a.to_i64())))
						}
					}
				}
				| Op::Drop(n, _) => {
					for _ in 0..*n {
						stack.pop();
					}
				}
				| Op::Swap(_) => {
					let a = stack.pop().unwrap();
					let b = stack.pop().unwrap();
					stack.push(a);
					stack.push(b);
				}
				| Op::Over(n, _) => {
					stack.push(stack[stack.len() - *n as usize - 1].clone())
				}
				| Op::Dup(n, _) => {
					for _ in 0..*n {
						stack.push(stack[stack.len() - *n as usize].clone());
					}
				}
				| Op::If(label_count, _) => {
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
						.unwrap_or((0, &Op::Nop));
					let (end_label, _) =
						self.find_op_by_label(label_count, "EndIf").unwrap();
					cf_labels.as_mut_slice()[if_label].else_label = else_label as i64;
					cf_labels.as_mut_slice()[if_label].end_label = end_label as i64;
				}
				| Op::While(label_count, _) => {
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
				| Op::Then(label_count, else_, _) => {
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
				| Op::Else(label_count, ..) => {
					let (idx, _) = labels_map.get(label_count).unwrap();
					ip = cf_labels[*idx].end_label as usize
				}
				| Op::End(label_count, while_, ..) => {
					let (_, idx) = labels_map.get(label_count).unwrap();
					if *while_ {
						ip = cf_labels[*idx].while_label as usize - 1;
					}
				}
				| Op::Do(label_count, ..) => {
					let (_, idx) = labels_map.get(label_count).unwrap();
					match stack.pop().unwrap() {
						| Data::Bool(false)
						| Data::I64(0)
						| Data::F64(0.0)
						| Data::Ptr(0) => ip = cf_labels[*idx].end_label as usize,
						| _ => (),
					}
				}
				| Op::Eq(..) => {
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
				| Op::Neq(..) => {
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
				| Op::Lt(..) => {
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
				| Op::Gt(..) => {
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
				| Op::Lte(..) => {
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
				| Op::Gte(..) => {
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
				| Op::Syscall(syscode, _argc, annot) => {
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
				| Op::Argc(_) | Op::Argv(_) => {
					self.add_error(
						"Program Argument is not supported in simulation mode".into(),
					)
					.exit(1)
				}
				| Op::Deref(_) => {
					match stack.pop().unwrap() {
						| Data::Ptr(val) => {
							if val as usize >= MEM_LENGTH {
								self.add_error(
									"Memory access out of bounds inside of simulation"
										.to_string(),
								)
								.exit(1)
							} else {
								stack.push(Data::I64(memory[val as usize] as i64))
							}
						}
						| _ => unreachable!(),
					}
				}
				| Op::Nop => unreachable!(),
			}
			ip += 1;
		}
	}
}

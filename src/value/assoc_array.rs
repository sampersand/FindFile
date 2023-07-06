use crate::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Debug, Clone)]
pub struct AssocArray(Rc<RefCell<HashMap<Value, Value>>>);

impl PartialEq for AssocArray {
	fn eq(&self, rhs: &Self) -> bool {
		if Rc::ptr_eq(&self.0, &rhs.0) {
			return true;
		}

		if self.len() != rhs.len() {
			return false;
		}

		// let mut v = self.keys().collect::<Vec<_>>();

		todo!()
	}
}

impl AssocArray {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.0.borrow().len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.borrow().is_empty()
	}

	// pub fn insert(&self, )
}

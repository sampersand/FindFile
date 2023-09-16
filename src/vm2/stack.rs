use crate::Value;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct Stack {
	items: MaybeUninit<Box<[MaybeUninit<Value>]>>,
	len: usize,
}

impl Stack {
	pub fn new() -> Self {
		Self::with_capacity(16)
	}

	pub fn with_capacity(cap: usize) -> Self {
		assert_ne!(cap, 0, "must have a starting capacity");

		Self { items: MaybeUninit::new(Vec::with_capacity(cap).into_boxed_slice()), len: 0 }
	}

	pub fn capacity(&self) -> usize {
		unsafe { self.items.assume_init_ref() }.len()
	}

	unsafe fn offset_from_len_mut(&mut self, index: usize) -> &mut MaybeUninit<Value> {
		let slice = self.items.assume_init_mut();
		debug_assert!(self.len + index < slice.len());
		slice.get_unchecked_mut(self.len)
	}

	pub fn push(&mut self, item: Value) {
		if unlikely!(self.capacity() == self.len) {
			// extend the capacity manually
			let mut v = Vec::from(unsafe { self.items.assume_init_read() });
			v.reserve(v.capacity() * 2);
			self.items.write(v.into_boxed_slice());
		}

		unsafe {
			self.offset_from_len_mut(0).write(item);
			self.len += 1;
		}
	}

	pub fn pop(&mut self) -> Option<Value> {
		if unlikely!(self.len == 0) {
			return None;
		}

		unsafe {
			let item = self.offset_from_len_mut(0).assume_init_read();

			if cfg!(debug_assertions) {
				*self.offset_from_len_mut(0) = MaybeUninit::uninit();
			}

			self.len -= 1;

			Some(item)
		}
	}

	// pub unsafe fn past_end(&self, amount: usize) -> Value {
	// 	self.offset_from_len_mut(amount).assume_init_read()
	// }

	// pub unsafe fn past_end(&self, amount: usize) -> Value {
	// 	self.offset_from_len_mut(amount).assume_init_read()
	// }
}

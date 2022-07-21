use std::marker::PhantomData;

use capability::*;
use kind::*;
use scope::*;

mod scope {
	pub trait Scope {}

	pub struct Chunks;
	impl Scope for Chunks {}

	pub struct Index;
	impl Scope for Index {}
}

mod kind {
	pub trait Kind {}

	pub struct None;
	impl Kind for None {}

	pub trait AnyKind {}

	pub struct Shared;
	impl Kind for Shared {}
	impl AnyKind for Shared {}

	pub struct Exclusive;
	impl Kind for Exclusive {}
	impl AnyKind for Exclusive {}
}

mod capability {
	pub trait ReadChunk {
		fn read(&self);
	}

	pub trait WriteChunk {
		fn write(&self);
	}

	pub trait DeleteChunk {
		fn delete(&self);
	}
}

#[derive(Default)]
pub struct LockState<C, I> {
	chunks: PhantomData<C>,
	index: PhantomData<I>,
}

#[derive(Default)]
pub struct Transaction<C, I> {
	locks: LockState<C, I>,
}

pub trait Lock<S: Scope, K: Kind> {
	type Output;

	fn locka(self) -> Self::Output;
}

impl<K: Kind, C, I> Lock<Chunks, K> for Transaction<C, I> {
	type Output = Transaction<K, I>;

	fn locka(self) -> Self::Output {
		Transaction {
			locks: LockState {
				chunks: Default::default(),
				index: self.locks.index,
			},
		}
	}
}

impl<K: Kind, C, I> Lock<Index, K> for Transaction<C, I> {
	type Output = Transaction<C, K>;

	fn locka(self) -> Self::Output {
		Transaction {
			locks: LockState {
				index: Default::default(),
				chunks: self.locks.chunks,
			},
		}
	}
}

impl<C, I> Transaction<C, I> {
	pub fn lock<S: Scope, K: Kind>(self) -> <Self as Lock<S, K>>::Output
	where
		Self: Lock<S, K>,
	{
		Lock::<S, K>::locka(self)
	}
}

impl<C: AnyKind, I> ReadChunk for Transaction<C, I> {
	fn read(&self) {}
}

impl<C: AnyKind, I> WriteChunk for Transaction<C, I> {
	fn write(&self) {}
}

impl<I> DeleteChunk for Transaction<Exclusive, I> {
	fn delete(&self) {}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn full_example() {
		fn delete_chunk<T: DeleteChunk>(txn: T) {
			txn.delete();
		}

		let txn = Transaction {
			locks: LockState {
				chunks: PhantomData::<Shared>::default(),
				index: PhantomData::<Shared>::default(),
			},
		};

		let txn = txn.lock::<Chunks, Exclusive>();

		delete_chunk(txn);
	}
}
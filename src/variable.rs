use std::hash::{Hash, Hasher};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::fmt;
use std::cell::Cell;
use std::rc::Rc;

#[cfg(not(debug_assertions))]
pub trait Spawnable: Hash + Eq + Clone {
	type Namespace;

	fn namespace(&self) -> &Self::Namespace;

	fn spawn(ns: &Self::Namespace) -> Self;
}

#[cfg(debug_assertions)]
pub trait Spawnable: Hash + Eq + Clone + fmt::Display + fmt::Debug {
	type Namespace;

	fn namespace(&self) -> &Self::Namespace;

	fn spawn(ns: &Self::Namespace) -> Self;
}

/// Parented variables.
pub trait Family: Spawnable {
	fn generate(&self) -> Self;

	/// Check if descendent of (or equal to).
	fn is_descendent_of(&self, other: &Self) -> bool;

	/// Check if parent of (or equal to).
	fn is_parent_of(&self, other: &Self) -> bool {
		other.is_descendent_of(self)
	}
}

#[cfg(not(debug_assertions))]
pub trait Incr: Copy {
	fn incr(self) -> Self;
}

#[cfg(debug_assertions)]
pub trait Incr: Copy + fmt::Display + fmt::Debug {
	fn incr(self) -> Self;
}

impl Incr for u32 {
	fn incr(self) -> u32 {
		self + 1
	}
}

impl Incr for i32 {
	fn incr(self) -> i32 {
		self + 1
	}
}

impl Incr for u64 {
	fn incr(self) -> u64 {
		self + 1
	}
}

impl Incr for usize {
	fn incr(self) -> usize {
		self + 1
	}
}

#[derive(Clone, Debug)]
pub struct Var<'a, T: Incr> {
	count: &'a Cell<T>,
	id: T
}

impl<'a, T: Incr> Var<'a, T> {
	pub fn from(id: T, namespace: &&'a Cell<T>) -> Var<'a, T> where T: Ord {
		if id >= namespace.get() {
			namespace.set(id.incr())
		}
		Var {
			count: namespace,
			id: id
		}
	}
}

impl<'a, T: Incr + Hash + Eq + Clone> Spawnable for Var<'a, T> {
	type Namespace = &'a Cell<T>;

	fn namespace(&self) -> &&'a Cell<T> {
		&self.count
	}

	fn spawn(namespace: &&'a Cell<T>) -> Var<'a, T> {
		let id = namespace.get();
		namespace.set(id.incr());
		Var {
			count: namespace,
			id: id
		}
	}
}

impl<'a, T: Incr + PartialOrd + Eq> PartialOrd for Var<'a, T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.id.partial_cmp(&other.id)
	}
}

impl<'a, T: Incr + Ord + Eq> Ord for Var<'a, T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.id.cmp(&other.id)
	}
}

impl<'a, T: Incr + Eq> PartialEq for Var<'a, T> {
	fn eq(&self, other: &Var<'a, T>) -> bool {
		self.count as *const Cell<T> == other.count as *const Cell<T> && self.id == other.id
	}
}

impl<'a, T: Incr + Eq> Eq for Var<'a, T> {}

impl<'a, T: Incr + Hash> Hash for Var<'a, T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.id.hash(state)
	}
}

impl<'a, T: Incr + fmt::Display> fmt::Display for Var<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "x{}", self.id)
	}
}

pub struct ParentedInner<T> {
	parent: Option<Rc<ParentedInner<T>>>,
	value: T
}

impl<T: PartialEq> ParentedInner<T> {
	fn is_descendent_of(this: &Rc<Self>, other: &Parented<T>) -> bool {
		if this.value == other.0.value {
			true
		} else {
			match this.parent.as_ref() {
				Some(parent) => Self::is_descendent_of(parent, other),
				None => false
			}
		}
	}
}

pub struct Parented<T>(Rc<ParentedInner<T>>);

impl<T: Hash> Hash for Parented<T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.0.value.hash(h)
	}
}

impl<T: PartialEq> PartialEq for Parented<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0.value.eq(&other.0.value)
	}
}
impl<T: Eq> Eq for Parented<T> { }

impl<T: PartialOrd> PartialOrd for Parented<T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.0.value.partial_cmp(&other.0.value)
	}
}

impl<T: Ord> Ord for Parented<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.value.cmp(&other.0.value)
	}
}

impl<T: fmt::Display> fmt::Display for Parented<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.value.fmt(f)
	}
}

impl<T: fmt::Debug> fmt::Debug for Parented<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.value.fmt(f)
	}
}

impl<T> Clone for Parented<T> {
	fn clone(&self) -> Self {
		Parented(self.0.clone())
	}
}

impl<T> From<T> for Parented<T> {
	fn from(value: T) -> Self {
		Parented(Rc::new(ParentedInner {
			parent: None,
			value
		}))
	}
}

impl<T> Spawnable for Parented<T> where T: Spawnable {
	type Namespace = T::Namespace;

	fn namespace(&self) -> &T::Namespace {
		self.0.value.namespace()
	}

	fn spawn(namespace: &T::Namespace) -> Self {
		Parented(Rc::new(ParentedInner {
			parent: None,
			value: T::spawn(namespace)
		}))
	}
}

impl<T: PartialEq> Family for Parented<T> where T: Spawnable {
	fn generate(&self) -> Self {
		Parented(Rc::new(ParentedInner {
			parent: Some(self.0.clone()),
			value: T::spawn(self.namespace())
		}))
	}

	fn is_descendent_of(&self, other: &Self) -> bool {
		ParentedInner::is_descendent_of(&self.0, other)
	}
}

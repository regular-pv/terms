use std::hash::{Hash, Hasher};
use std::fmt;
use std::cell::Cell;

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

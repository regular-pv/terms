extern crate rand;

//use std::rc::Arc;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::collections::hash_map::DefaultHasher;
use std::sync::atomic::{self, AtomicU64};
use std::fmt;

pub mod pattern;
pub mod variable;
pub mod macros;
mod index;

pub use pattern::{Pattern, PatternKind, PatternLike, PatternLikeKind};
pub use variable::Var;
pub use index::*;

pub trait Ranked {
    fn arity(&self) -> usize;
}

/// A term.
///
/// Subterms are reference counter using [`std::rc::Arc`].
pub struct Term<F> {
    f: F,
    subs: Arc<Vec<Self>>,
    hash: AtomicU64
}

impl<F: fmt::Debug> fmt::Debug for Term<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}({:?})", self.f, self.subs)
    }
}

impl<F: fmt::Display> fmt::Display for Term<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.f.fmt(f)?;
        match self.subs.split_first() {
            Some((head, tail)) => {
                write!(f, "(")?;
                head.fmt(f)?;
                for e in tail.iter() {
                    write!(f, ", ")?;
                    e.fmt(f)?;
                }
                write!(f, ")")
            },
            None => Ok(())
        }
    }
}

impl<F> Term<F> {
    pub fn new(f: F, subs: Vec<Self>) -> Self where F: Clone {
        Term {
            f: f,
            subs: Arc::new(subs),
            hash: AtomicU64::new(0)
        }
    }

    pub fn from_slice(f: F, subs: &[Self]) -> Self where F: Clone {
        Term {
            f: f,
            subs: Arc::new(subs.iter().map(|p| p.clone()).collect()),
            hash: AtomicU64::new(0)
        }
    }

    pub fn symbol(&self) -> &F {
        &self.f
    }

    // /// Note that it is unwise to use the hash of the while it is still mutating.
    // pub fn with_capacity(f: F, capacity: usize) -> Self where F: Clone {
    //     Term {
    //         f: f,
    //         subs: Vec::with_capacity(capacity),
    //         hash: Cell::new(0)
    //     }
    // }

    pub fn sub_terms(&self) -> &Vec<Self> {
        &self.subs
    }

    // /// Beware that is may change to term's hash!
    // pub fn sub_terms_mut(&mut self) -> &mut Vec<Self> {
    //     self.hash.set(0);
    //     &mut self.subs
    // }

    pub fn depth(&self) -> u64 {
        let mut depth = 0;
        for sub in self.subs.iter() {
            let d = sub.depth() + 1;
            if d > depth {
                depth = d
            }
        }

        depth
    }

    fn random_with_zeros(zero_alphabet: &[F], alphabet: &[F], max_depth: u64) -> Term<F> where F: Clone + Ranked {
        let i: usize = rand::random();
        let (f, arity, next_depth) = if max_depth == 0 {
            let f: &F = &zero_alphabet[i%zero_alphabet.len()];
            (f.clone(), 0, 0)
        } else {
            let f: &F = &alphabet[i%alphabet.len()];
            (f.clone(), f.arity(), max_depth-1)
        };

        let mut subs = Vec::with_capacity(arity);
        for _ in 0..arity {
            subs.push(Self::random_with_zeros(zero_alphabet, alphabet, next_depth))
        }

        Term {
            f: f,
            subs: Arc::new(subs),
            hash: AtomicU64::new(0)
        }
    }

    /// Generate a random term with the given alphabet.
    /// The alphabet must contain at least one constant (of arity 0), otherwise it will panic.
    pub fn random(alphabet: &[F], max_depth: u64) -> Term<F> where F: Clone + Ranked {
        let mut zeros = Vec::with_capacity(alphabet.len());
        for f in alphabet.iter() {
            if f.arity() == 0 {
                zeros.push(f.clone())
            }
        }

        assert!(!zeros.is_empty());
        Self::random_with_zeros(&zeros, alphabet, max_depth)
    }
}

impl<F, X> PatternLike<F, X> for Term<F> {
    fn kind(&self) -> PatternLikeKind<F, X, Self> {
        PatternLikeKind::Cons(&self.f, &self.subs)
    }
}

impl<F: Clone> Clone for Term<F> {
    fn clone(&self) -> Term<F> {
        Term {
            f: self.f.clone(),
            subs: self.subs.clone(),
            hash: AtomicU64::new(self.hash.load(atomic::Ordering::Relaxed)),
        }
    }
}

impl<F: PartialEq> PartialEq for Term<F> {
    fn eq(&self, other: &Term<F>) -> bool {
        self.f == other.f && self.subs == other.subs
    }
}

impl<F: PartialEq + Eq> Eq for Term<F> {}

impl<F: Hash> Hash for Term<F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut h = self.hash.load(atomic::Ordering::Relaxed);
        if h == 1 { // hash is beeing computed by another thread.
            loop {
                h = self.hash.load(atomic::Ordering::Relaxed);
                if h != 1 {
                    break;
                }
            }
        }
        if h == 0 {
            //self.hash.set(1); // set to 1 to avoid loops.
            self.hash.store(1, atomic::Ordering::Relaxed);

            let mut hasher = DefaultHasher::new();
            self.f.hash(&mut hasher);
            for sub in self.subs.iter() {
                Term::<F>::hash(sub, &mut hasher)
            }
            h = hasher.finish();
            if h <= 1 { // just to be sure...
                h = 2;
            }
            self.hash.store(h, atomic::Ordering::Relaxed);
        }
        h.hash(state)
    }
}

impl<F: Ord> PartialOrd for Term<F> {
    fn partial_cmp(&self, other: &Term<F>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: Ord> Ord for Term<F> {
    fn cmp(&self, other: &Term<F>) -> Ordering {
        match self.depth().cmp(&other.depth()) {
            Ordering::Equal => {
                match self.f.cmp(&other.f) {
                    Ordering::Equal => {
                        match self.subs.len().cmp(&other.subs.len()) {
                            Ordering::Equal => {
                                for (i, a) in self.subs.iter().enumerate() {
                                    let b = &other.subs[i];
                                    match a.cmp(b) {
                                        Ordering::Equal => (),
                                        ord => return ord
                                    }
                                }

                                Ordering::Equal
                            },
                            ord => ord
                        }
                    },
                    ord =>  ord
                }
            },
            ord => ord
        }
    }
}

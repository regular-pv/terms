//use std::rc::Arc;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::sync::atomic::{self, AtomicU64};
use std::fmt;
use crate::Term;

// pub trait Meta<F>: Clone + Eq + Sized + fmt::Debug {
//     /// Gives the sub-patterns needed to recognize the given symbol.
//     fn matches(&self, index: usize, f: &F, arity: usize) -> Option<(Vec<Self>, Self)>;
// }
//
// impl<F> Meta<F> for () {
//     fn matches(&self, _index: usize, _f: &F, arity: usize) -> Option<(Vec<Self>, Self)> {
//         let mut sub_patterns = Vec::with_capacity(arity);
//         sub_patterns.resize(arity, ());
//         Some((sub_patterns, ()))
//     }
// }

/// Any object that can act like a pattern. Such as a term.
pub trait PatternLike<F, X>: Sized {
    fn kind(&self) -> PatternLikeKind<F, X, Self>;
}

pub enum PatternLikeKind<'a, F, X, T: PatternLike<F, X>> {
    Cons(&'a F, &'a [T]),
    Var(&'a X)
}

pub trait Spawnable: Sized {
    fn spawn() -> Self;
}

pub struct Pattern<F, X> {
    kind: PatternKind<F, X>,
    hash: AtomicU64
}

impl<F, X> PatternLike<F, X> for Pattern<F, X> {
    fn kind(&self) -> PatternLikeKind<F, X, Self> {
        match &self.kind {
            PatternKind::Cons(f, subs) => PatternLikeKind::Cons(&f, &subs),
            PatternKind::Var(x) => PatternLikeKind::Var(&x)
        }
    }
}

impl<F: fmt::Debug, X: fmt::Debug> fmt::Debug for Pattern<F, X> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            PatternKind::Cons(g, subs) => write!(f, "{:?}({:?})", g, subs),
            PatternKind::Var(x) => write!(f, "{:?}", x)
        }
    }
}

impl<F: fmt::Display, X: fmt::Display> fmt::Display for Pattern<F, X> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            PatternKind::Cons(g, subs) => {
                g.fmt(f)?;
                match subs.split_first() {
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
            },
            PatternKind::Var(x) => x.fmt(f)
        }
    }
}

/// A pattern.
pub enum PatternKind<F, X> {
    Cons(F, Arc<Vec<Pattern<F, X>>>),
    Var(X)
}

impl<F, X> Pattern<F, X> {
    pub fn kind(&self) -> &PatternKind<F, X> {
        &self.kind
    }

    pub fn into_kind(self) -> PatternKind<F, X> {
        self.kind
    }

    pub fn cons(f: F, subs: Vec<Self>) -> Self where F: Clone, X: Clone {
        Pattern {
            kind: PatternKind::Cons(f, Arc::new(subs)),
            hash: AtomicU64::new(0)
        }
    }

    pub fn from_slice(f: F, subs: &[Self]) -> Self where F: Clone, X: Clone {
        Pattern {
            kind: PatternKind::Cons(f, Arc::new(subs.iter().map(|p| p.clone()).collect())),
            hash: AtomicU64::new(0)
        }
    }

    pub fn var(x: X) -> Self {
        Pattern {
            kind: PatternKind::Var(x),
            hash: AtomicU64::new(0)
        }
    }

    pub fn symbol(&self) -> Option<&F> {
        match &self.kind {
            PatternKind::Cons(f, _) => Some(f),
            _ => None
        }
    }

    pub fn sub_patterns(&self) -> Option<&Vec<Self>> {
        match &self.kind {
            PatternKind::Cons(_, list) => Some(list),
            _ => None
        }
    }

    // /// Beware that this may change the pattern hash!
    // pub fn sub_patterns_mut(&mut self) -> Option<&mut Vec<Arc<Self>>> {
    //     self.hash.set(0);
    //     match &mut self.kind {
    //         PatternKind::Cons(_, ref mut list) => Some(list),
    //         _ => None
    //     }
    // }

    pub fn get(&self, i: usize) -> Option<&Self> {
        match &self.kind {
            PatternKind::Cons(_, list) => list.get(i),
            _ => None
        }
    }

    pub fn as_cons(&self) -> Option<(&F, &Vec<Self>)> {
        match &self.kind {
            PatternKind::Cons(f, list) => Some((f, list)),
            _ => None
        }
    }

    pub fn as_term(&self) -> Option<Term<F>> where F: Clone {
        match &self.kind {
            PatternKind::Var(_) => None,
            PatternKind::Cons(f, sub_patterns) => {
                let mut sub_terms = Vec::with_capacity(sub_patterns.len());
                for sub in sub_patterns.iter() {
                    match sub.as_term() {
                        Some(term) => sub_terms.push(term),
                        None => return None
                    }
                }

                Some(Term::new(f.clone(), sub_terms))
            }
        }
    }

    pub fn variables(&self) -> UniqueVariables<X> where X: PartialEq {
        UniqueVariables::new(self)
    }

    pub fn map_variables<Y, M>(&self, g: &M) -> Pattern<F, Y> where M: Fn(&X) -> Pattern<F, Y>, F: Clone {
        let kind = match &self.kind {
            PatternKind::Var(x) => {
                ((*g)(x)).kind
            },
            PatternKind::Cons(f, sub_patterns) => {
                let mapped_sub_patterns = sub_patterns.iter().map(|sub| sub.map_variables(g)).collect();
                PatternKind::Cons(f.clone(), Arc::new(mapped_sub_patterns))
            }
        };

        Pattern {
            kind: kind,
            hash: AtomicU64::new(0)
        }
    }

    pub fn try_map_variables<Y, M>(&self, g: &M) -> Option<Pattern<F, Y>> where M: Fn(&X) -> Option<Pattern<F, Y>>, F: Clone {
        let kind = match &self.kind {
            PatternKind::Var(x) => {
                match (*g)(x) {
                    Some(p) => p.kind,
                    None => return None
                }
            },
            PatternKind::Cons(f, sub_patterns) => {
                let mut mapped_sub_patterns = Vec::with_capacity(sub_patterns.len());
                for sub in sub_patterns.iter() {
                    match sub.try_map_variables(g) {
                        Some(mapped_sub) => mapped_sub_patterns.push(mapped_sub),
                        None => return None
                    }
                }
                PatternKind::Cons(f.clone(), Arc::new(mapped_sub_patterns))
            }
        };

        Some(Pattern {
            kind: kind,
            hash: AtomicU64::new(0)
        })
    }

    /// Find a renaming from X -> Y so that both patterns are equals.
    pub fn renaming<Z: Clone, W: Clone, Y: AsRef<W>>(&self, other: &Pattern<F, Y>, renaming: &mut HashMap<Z, W>) -> bool where X: AsRef<Z> + PartialOrd<Y>, Z: Hash + Eq, W: Eq, F: PartialEq {
        match (self.kind(), other.kind()) {
            (PatternKind::Cons(f1, subs1), PatternKind::Cons(f2, subs2)) if f1 == f2 && subs1.len() == subs2.len() => {
                for i in 0..subs1.len() {
                    let a = &subs1[i];
                    let b = &subs2[i];

                    if !a.renaming(b, renaming) {
                        return false
                    }
                }

                true
            },
            (PatternKind::Var(x), PatternKind::Var(y)) => {
                let z = x.as_ref();
                if let Some(w) = renaming.get(z) {
                    if w == y.as_ref() {
                        true
                    } else {
                        false
                    }
                } else {
                    if x <= y {
                        renaming.insert(z.clone(), y.as_ref().clone());
                        true
                    } else {
                        false
                    }
                }
            },
            _ => false
        }
    }
}

impl<F, X> From<PatternKind<F, X>> for Pattern<F, X> {
    fn from(kind: PatternKind<F, X>) -> Pattern<F, X> {
        Pattern {
            kind: kind,
            hash: AtomicU64::new(0)
        }
    }
}

pub enum UniqueVariables<'a, X: PartialEq> {
    Var(&'a X, bool),
    Cons(Box<Vec<UniqueVariables<'a, X>>>, Vec<&'a X>)
}

impl<'a, X: PartialEq> UniqueVariables<'a, X> {
    pub fn new<F>(pattern: &'a Pattern<F, X>) -> UniqueVariables<'a, X> {
        match &pattern.kind {
            PatternKind::Var(x) => UniqueVariables::Var(x, false),
            PatternKind::Cons(_, sub_patterns) => {
                let iterators = sub_patterns.iter().map(|sub| {
                    sub.variables()
                }).collect();

                UniqueVariables::Cons(Box::new(iterators), Vec::new())
            }
        }
    }
}

impl<'a, X: PartialEq> Iterator for UniqueVariables<'a, X> {
    type Item = &'a X;

    fn next(&mut self) -> Option<&'a X> {
        match self {
            UniqueVariables::Var(x, visited) => {
                if *visited {
                    None
                } else {
                    *visited = true;
                    Some(x)
                }
            },
            UniqueVariables::Cons(ref mut iterators, ref mut visited) => {
                for it in iterators.iter_mut() {
                    loop {
                        match it.next() {
                            Some(x) => {
                                if !visited.contains(&x) {
                                    visited.push(x);
                                    return Some(x)
                                }
                            },
                            None => break
                        }
                    }
                }
                None
            }
        }
    }
}

// impl<F: Clone + Eq + fmt::Debug, X: Eq + fmt::Debug + Clone + Spawnable> Meta<F> for Pattern<F, X> {
//     fn matches(&self, _index: usize, f: &F, arity: usize) -> Option<(Vec<Self>, Self)> {
//         match &self.kind {
//             PatternKind::Cons(g, list) if g == f && list.len() == arity => {
//                 let sub_patterns = list.iter().map(|p| (**p).clone()).collect();
//                 Some((sub_patterns, self.clone()))
//             },
//             PatternKind::Var(_) => {
//                 let mut sub_patterns = Vec::with_capacity(arity);
//                 for i in 0..arity {
//                     sub_patterns.push(X::spawn().into())
//                 }
//                 Some((sub_patterns, self.clone()))
//             },
//             _ => None
//         }
//     }
// }

impl<F: Clone, X: Clone> Clone for Pattern<F, X> {
    fn clone(&self) -> Pattern<F, X> {
        let kind = match &self.kind {
            PatternKind::Cons(f, l) => PatternKind::Cons(f.clone(), l.clone()),
            PatternKind::Var(x) => PatternKind::Var(x.clone())
        };
        Pattern {
            kind: kind,
            hash: AtomicU64::new(self.hash.load(atomic::Ordering::Relaxed))
        }
    }
}

impl<F: PartialEq, X: PartialEq> PartialEq for Pattern<F, X> {
    fn eq(&self, other: &Pattern<F, X>) -> bool {
        match (&self.kind, &other.kind) {
            (PatternKind::Cons(f1, subs1), PatternKind::Cons(f2, subs2)) => {
                f1 == f2 && subs1 == subs2
            },
            (PatternKind::Var(x1), PatternKind::Var(x2)) => x1 == x2,
            _ => false
        }
    }
}

impl<F: PartialEq + Eq, X: PartialEq + Eq> Eq for Pattern<F, X> {}

impl<F: Hash, X: Hash> Hash for Pattern<F, X> {
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
            match &self.kind {
                PatternKind::Cons(f, l) => {
                    f.hash(&mut hasher);
                    for sub in l.iter() {
                        Pattern::<F, X>::hash(sub, &mut hasher)
                    }
                },
                PatternKind::Var(x) => {
                    x.hash(&mut hasher)
                }
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

impl<F, X> From<X> for Pattern<F, X> {
    fn from(x: X) -> Self {
        Pattern {
            kind: PatternKind::Var(x),
            hash: AtomicU64::new(0)
        }
    }
}

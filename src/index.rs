use crate::Pattern;

pub trait Index: Ord + Copy {
    const ZERO: Self;

    fn next(self) -> Self;
}

impl Index for u8 {
    const ZERO: u8 = 0;

    fn next(self) -> u8 {
        self + 1
    }
}

impl Index for u16 {
    const ZERO: u16 = 0;

    fn next(self) -> u16 {
        self + 1
    }
}

impl Index for u32 {
    const ZERO: u32 = 0;

    fn next(self) -> u32 {
        self + 1
    }
}

impl Index for u64 {
    const ZERO: u64 = 0;

    fn next(self) -> u64 {
        self + 1
    }
}

impl Index for usize {
    const ZERO: usize = 0;

    fn next(self) -> usize {
        self + 1
    }
}

impl<F: Clone, X: Index> Pattern<F, X> {
    pub fn reindex(&self) -> Pattern<F, X> {
        // Step 1: retreive every index.
        let mut indexes = Vec::new();
        for i in self.variables() {
            indexes.push(i)
        }
        indexes.sort();

        // Step 2: create new indexes.
        let mut new_indexes = Vec::with_capacity(indexes.len());
        let mut next = X::ZERO;
        for _ in indexes.iter() {
            new_indexes.push(next);
            next = next.next();
        }

        // Step 3: replace the old indexes in the pattern.
        self.map_variables(&|x| {
            for i in 0..indexes.len() {
                if indexes[i] == x {
                    return Pattern::var(new_indexes[i]);
                }
            }
            unreachable!()
        })
    }
}

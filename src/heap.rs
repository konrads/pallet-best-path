// heap implementation with the use of sp collections

use sp_std::collections::btree_set::BTreeSet;
use sp_std::cmp::Ord;

pub (crate) trait Heap<E> {
    fn pop(&mut self) -> Option<E>;
    fn push(&mut self, e: E) -> bool;
}

pub (crate) struct BTreeSetHeap<E>(BTreeSet<E>);

impl<E> BTreeSetHeap<E>
where E: Ord, E: Clone {
    #[allow(dead_code)]
    pub (crate) fn new() -> Self {
        Self(BTreeSet::new())
    }
}

impl <E> Heap<E> for BTreeSetHeap<E>
where E: Ord, E: Clone {
    fn pop(&mut self) -> Option<E> {
        let last = self.0.iter().next_back()?;
        // FIXME: can avoid the clone???
        let last = last.clone();
        self.0.remove(&last);
        Some(last)
    }
    
    fn push(&mut self, e: E) -> bool {
        self.0.insert(e)
    }
}

use pipec_arena::{ASpan, Arena};

#[derive(Debug, Clone, Copy, Default)]
pub enum ListNode<T> {
    #[default]
    Empty,
    Node(T, ASpan<Self>),
}

#[derive(Clone, Copy, Debug)]
pub struct ADynList<T> {
    first: ASpan<ListNode<T>>,
    mutate: ASpan<ListNode<T>>,
}

pub struct ADynListIter<'a, T> {
    current: ASpan<ListNode<T>>,
    arena: &'a Arena,
}

impl<'a, T> Iterator for ADynListIter<'a, T>
where
    T: Clone,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.arena.take(self.current.clone()) {
            ListNode::Empty => None,
            ListNode::Node(current, next) => {
                self.current = next.clone();
                Some(current.clone())
            }
        }
    }
}

impl<T> ADynList<T> {
    pub fn push(&mut self, input: T, arena: &mut Arena) {
        let handle = arena.take(self.mutate.clone());
        let empty = arena.alloc(ListNode::Empty);
        *handle = ListNode::Node(input, empty.clone());
        self.mutate = empty
    }
    pub fn new(arena: &mut Arena) -> Self {
        let out = arena.alloc(ListNode::Empty);
        ADynList {
            first: out.clone(),
            mutate: out,
        }
    }
    pub fn first(&self, arena: &mut Arena) -> &mut ListNode<T> {
        arena.take(self.first.clone())
    }
    pub fn iter<'a>(&'a self, arena: &'a Arena) -> ADynListIter<'a, T> {
        ADynListIter {
            current: self.first.clone(),
            arena,
        }
    }
}

impl<T: Clone> ADynList<T> {
    pub fn len_eq(&self, input: usize, arena: &mut Arena) -> bool {
        let mut val: usize = 0;
        for _ in self.iter(arena) {
            val += 1;
            if val > input {
                return false;
            }
        }
        val == input
    }

    pub fn len_gt(&self, input: usize, arena: &mut Arena) -> bool {
        let mut val: usize = 0;
        for _ in self.iter(arena) {
            val += 1;
            if val > input {
                return true;
            }
        }
        false
    }

    pub fn len_lt(&self, input: usize, arena: &mut Arena) -> bool {
        !self.len_gt(input, arena)
    }
}

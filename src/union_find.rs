#[derive(Debug, Copy, Clone)]
pub struct Entry {
    parent: usize,
    size: usize,
}

impl Entry {
    fn new(parent: usize) -> Self {
        Self { parent, size: 1 }
    }

    pub fn id(&self) -> usize {
        self.parent
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct SimpleDisjointSet(Vec<Entry>, usize);

impl SimpleDisjointSet {
    pub fn new(n: usize) -> Self {
        SimpleDisjointSet((0..n).map(|i| Entry::new(i)).collect(), n)
    }

    pub fn node_count(&self) -> usize {
        self.0.len()
    }

    pub fn set_count(&self) -> usize {
        self.1
    }

    pub fn reset(&mut self) {
        self.0
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = Entry::new(i));
        self.1 = self.0.len();
    }

    fn find_id(&mut self, mut id: usize) -> usize {
        assert!(id < self.node_count());

        unsafe {
            macro_rules! p {
                ($x:expr) => {
                    self.0.get_unchecked_mut($x).parent
                };
            }

            while p!(id) != id {
                let next = p!(id);
                debug_assert!(next < self.node_count());
                p!(id) = p!(next);
                id = next;
            }
        }
        id
    }

    pub fn find_mut(&mut self, id: usize) -> &mut Entry {
        let id = self.find_id(id);
        &mut self.0[id]
    }

    pub fn union(&mut self, x: usize, y: usize) -> bool {
        let x = self.find_id(x);
        let y = self.find_id(y);

        if x == y {
            return false;
        }

        debug_assert!(self.1 > 1);
        self.1 -= 1;
        let [x, y] = unsafe {
            let x: *mut _ = &mut self.0.get_unchecked_mut(x);
            let y: *mut _ = &mut self.0.get_unchecked_mut(y);
            debug_assert_ne!(x, y);
            [&mut *x, &mut *y]
        };
        if x.size < y.size {
            x.parent = y.parent;
            y.size += x.size;
        } else {
            y.parent = x.parent;
            x.size += y.size;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmm() {
        let mut set = SimpleDisjointSet::new(5);
        set.union(1, 2);
        assert_eq!(set.find_mut(1).id(), set.find_mut(2).id());
        assert_ne!(set.find_mut(0).id(), set.find_mut(1).id());
        set.union(3, 4);
        assert_eq!(set.find_mut(3).id(), set.find_mut(4).id());
        assert_ne!(set.find_mut(2).id(), set.find_mut(3).id());
        set.union(2, 3);
        assert_eq!(set.find_mut(1).id(), set.find_mut(4).id());

        assert_eq!(set.find_mut(4).size(), 4);
    }
}

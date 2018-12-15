use std::collections::VecDeque;

pub trait Bag {
    type Item;
    fn clear(&mut self);
    fn push(&mut self, x: Self::Item);
    fn pop(&mut self) -> Option<Self::Item>;
}

impl<T> Bag for Vec<T> {
    type Item = T;
    fn clear(&mut self) {
        self.clear();
    }
    fn push(&mut self, x: T) {
        self.push(x);
    }
    fn pop(&mut self) -> Option<T> {
        self.pop()
    }
}

impl<T> Bag for VecDeque<T> {
    type Item = T;
    fn clear(&mut self) {
        self.clear();
    }
    fn push(&mut self, x: T) {
        self.push_back(x);
    }
    fn pop(&mut self) -> Option<T> {
        self.pop_front()
    }
}

impl<T: Ord> Bag for std::collections::BinaryHeap<T> {
    type Item = T;
    fn clear(&mut self) {
        self.clear();
    }
    fn push(&mut self, x: T) {
        self.push(x);
    }
    fn pop(&mut self) -> Option<T> {
        self.pop()
    }
}

#[derive(Debug, Clone)]
pub struct CustomBucketQueue<B> {
    data: VecDeque<B>,
    front: usize,
}

pub type BucketQueue<T> = CustomBucketQueue<Vec<T>>;

impl<B: Bag> CustomBucketQueue<B> {
    pub fn clear(&mut self) {
        self.data.iter_mut().for_each(|x| x.clear());
        self.front = 0;
    }

    pub fn push(&mut self, priority: usize, extra: B::Item) where B: Default {
        assert!(priority >= self.front);
        let delta = priority - self.front;
        while self.data.len() <= delta {
            self.data.push_back(Default::default());
        }
        self.data[delta].push(extra);
    }

    pub fn pop(&mut self) -> Option<(usize, B::Item)> {
        loop {
            let first = self.data.front_mut()?;
            if let Some(x) = first.pop() {
                return Some((self.front, x));
            }

            let first = self.data.pop_front().unwrap();
            self.front += 1;
            self.data.push_back(first);
        }
    }
}

impl<B> Default for CustomBucketQueue<B> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            front: 0,
        }
    }
}

impl<T, B: Bag<Item=T> + Default> Extend<(usize, T)> for CustomBucketQueue<B> {
    fn extend<I>(&mut self, it: I) where I: IntoIterator<Item=(usize, T)> {
        it.into_iter().for_each(|(p, x)| self.push(p, x));
    }
}

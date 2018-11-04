use std::collections::hash_map::{Entry, HashMap, RandomState};
use std::collections::HashSet;
use std::hash::{BuildHasher, Hash};

pub struct UnGraph<I, N, E, S = RandomState> {
    nodes: HashMap<I, N, S>,
    edges: HashMap<(I, I), E, S>,
    adjacency: HashMap<I, HashSet<I, S>, S>,
}

impl<I: Clone, N: Clone, E: Clone, S: Clone> Clone for UnGraph<I, N, E, S> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            adjacency: self.adjacency.clone(),
        }
    }
}

fn min_max<T: Ord>(a: T, b: T) -> (T, T) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

impl<I: Copy + Ord + Hash, N, E, S: BuildHasher + Default> UnGraph<I, N, E, S> {
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity_and_hasher(nodes, S::default()),
            edges: HashMap::with_capacity_and_hasher(edges, S::default()),
            adjacency: HashMap::with_capacity_and_hasher(edges, S::default()),
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = (I, &N)> + ExactSizeIterator {
        self.nodes.iter().map(|(k, v)| (*k, v))
    }

    pub fn nodes_mut(&mut self) -> impl Iterator<Item = (I, &mut N)> + ExactSizeIterator {
        self.nodes.iter_mut().map(|(k, v)| (*k, v))
    }

    fn require_node(&self, i: I) {
        assert!(self.nodes.contains_key(&i));
    }

    pub fn try_add_node(&mut self, i: I, n: N) -> Option<&mut N> {
        if let Entry::Vacant(v) = self.nodes.entry(i) {
            Some(v.insert(n))
        } else {
            None
        }
    }

    pub fn add_node(&mut self, i: I, n: N) {
        self.try_add_node(i, n).expect("Node already exists");
    }

    pub fn try_remove_node(&mut self, i: I) -> Option<N> {
        let n = self.nodes.remove(&i)?;
        for j in self
            .adjacency
            .remove(&i)
            .expect("somehow not in adjacency?")
        {
            self.edges
                .remove(&min_max(i, j))
                .expect("edge in adjacency not in edges?");
            self.adjacency.get_mut(&j).unwrap().remove(&i);
        }
        Some(n)
    }

    pub fn remove_node(&mut self, i: I) -> N {
        self.try_remove_node(i).expect("Node does not exist")
    }

    pub fn edges(&self) -> impl Iterator<Item = (I, I, &E)> + ExactSizeIterator {
        self.edges.iter().map(|(k, v)| (k.0, k.1, v))
    }

    pub fn edges_mut(&mut self) -> impl Iterator<Item = (I, I, &mut E)> + ExactSizeIterator {
        self.edges.iter_mut().map(|(k, v)| (k.0, k.1, v))
    }

    pub fn try_add_edge(&mut self, i: I, j: I, e: E) -> Option<&mut E> {
        self.require_node(i);
        self.require_node(j);
        if let Entry::Vacant(v) = self.edges.entry(min_max(i, j)) {
            let e = v.insert(e);
            let i1 = self.adjacency.entry(i).or_default().insert(j);
            let i2 = self.adjacency.entry(j).or_default().insert(i);
            debug_assert!(i1 && i2);
            Some(e)
        } else {
            None
        }
    }

    pub fn add_edge(&mut self, i: I, j: I, e: E) {
        self.try_add_edge(i, j, e).expect("Edge already exists");
    }

    pub fn neighbours(&self, i: I) -> impl Iterator<Item = (I, &E)> + ExactSizeIterator {
        let e = self.adjacency.get(&i).expect("Node does not exist");
        e.iter()
            .cloned()
            .map(move |j| (j, self.edges.get(&min_max(i, j)).unwrap()))
    }
}

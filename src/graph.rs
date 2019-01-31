use std::cmp::Reverse;
use std::collections::hash_map::{Entry, HashMap, RandomState};
use std::collections::{BinaryHeap, HashSet};
use std::hash::{BuildHasher, Hash};
use std::ops::{Add, AddAssign};

use crate::bucket_queue::BucketQueue;

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

    pub fn node_ids(&self) -> impl ExactSizeIterator<Item = I> + Clone + '_ {
        self.nodes.keys().cloned()
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = (I, &N)> + Clone {
        self.nodes.iter().map(|(k, v)| (*k, v))
    }

    pub fn nodes_mut(&mut self) -> impl ExactSizeIterator<Item = (I, &mut N)> {
        self.nodes.iter_mut().map(|(k, v)| (*k, v))
    }

    pub fn contains_node(&self, i: I) -> bool {
        self.nodes.contains_key(&i)
    }

    fn require_node(&self, i: I) {
        assert!(self.contains_node(i));
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

    pub fn edges(&self) -> impl ExactSizeIterator<Item = (I, I, &E)> + Clone {
        self.edges.iter().map(|(k, v)| (k.0, k.1, v))
    }

    pub fn edges_mut(&mut self) -> impl ExactSizeIterator<Item = (I, I, &mut E)> {
        self.edges.iter_mut().map(|(k, v)| (k.0, k.1, v))
    }

    pub fn contains_edge(&self, i: I, j: I) -> bool {
        self.edges.contains_key(&min_max(i, j))
    }

    pub fn try_add_edge(&mut self, i: I, j: I, e: E) -> Option<&mut E> {
        assert!(i != j);
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

    pub fn neighbours(&self, i: I) -> impl ExactSizeIterator<Item = (I, &E)> {
        let e = self.adjacency.get(&i).expect("Node does not exist");
        e.iter()
            .cloned()
            .map(move |j| (j, self.edges.get(&min_max(i, j)).unwrap()))
    }
}

pub fn metric_closure_usize<I, N, E, S>(
    g: &UnGraph<I, N, E, S>,
    cost: impl Fn(&E) -> usize,
) -> UnGraph<I, (), usize, S>
where
    I: Copy + Ord + Hash,
    S: Default + BuildHasher,
{
    let mut closure = UnGraph::with_capacity(g.nodes().len(), g.nodes().len());
    g.node_ids().for_each(|n| closure.add_node(n, ()));
    g.node_ids().for_each(|n| {
        let distances = dijkstra_usize(g, n, &cost);
        distances.nodes().for_each(|(m, d)| {
            if n != m {
                closure.try_add_edge(n, m, *d);
            }
        });
    });
    closure
}

pub fn dijkstra_usize<I, N, E, S>(
    g: &UnGraph<I, N, E, S>,
    seed: I,
    cost: impl Fn(&E) -> usize,
) -> UnGraph<I, usize, (), S>
where
    I: Copy + Ord + Hash,
    S: Default + BuildHasher,
{
    let mut distances = UnGraph::with_capacity(g.nodes().len(), 0);

    let mut queue = BucketQueue::default();
    queue.push(0, seed);
    while let Some((d, n)) = queue.pop() {
        if distances.try_add_node(n, d).is_some() {
            for (m, e) in g.neighbours(n) {
                if !distances.contains_node(m) {
                    queue.push(d + cost(e), m);
                }
            }
        }
    }

    distances
}

pub fn steiner_mst<I, N, E, S, C>(
    g: &UnGraph<I, N, E, S>,
    seed: I,
    terminals: impl Iterator<Item = I>,
    cost: impl Fn(&E) -> C,
) -> (C, UnGraph<I, (), (), S>)
where
    I: Copy + Ord + Hash,
    S: Default + BuildHasher,
    C: Copy + Ord + Default + Add<Output = C> + AddAssign,
{
    let mut tree_cost = C::default();
    let mut tree = UnGraph::with_capacity(1, 0);
    tree.add_node(seed, ());

    let mut terminals = terminals.zip(0..).collect::<HashMap<_, usize, S>>();
    let mut heap = BinaryHeap::new();
    let mut incoming = HashMap::with_hasher(S::default());
    while !terminals.is_empty() {
        incoming.clear();
        heap.clear();
        heap.extend(
            terminals
                .iter()
                .map(|t| Reverse((C::default(), *t.0, Err(*t.1)))),
        );

        let (mut prev, path_cost) = loop {
            let Reverse((c, n, p)) = heap.pop().expect("tree not reachable from terminals");
            if let Entry::Vacant(entry) = incoming.entry(n) {
                entry.insert(p);

                if tree.contains_node(n) {
                    break (n, c);
                }

                for (m, e) in g.neighbours(n) {
                    if !incoming.contains_key(&m) {
                        heap.push(Reverse((c + cost(e), m, Ok(n))));
                    }
                }
            }
        };
        tree_cost += path_cost;

        while let &Ok(next) = incoming.get(&prev).unwrap() {
            tree.add_node(next, ());
            tree.add_edge(next, prev, ());
            prev = next;
        }

        terminals.remove(&prev);
    }

    (tree_cost, tree)
}

pub fn steiner_mst_usize<I, N, E, S>(
    g: &UnGraph<I, N, E, S>,
    seed: I,
    terminals: impl Iterator<Item = I>,
    cost: impl Fn(&E) -> usize,
) -> (usize, UnGraph<I, (), (), S>)
where
    I: Copy + Ord + Hash,
    S: Default + BuildHasher,
{
    let mut tree_cost = 0;
    let mut tree = UnGraph::with_capacity(1, 0);
    tree.add_node(seed, ());

    let mut terminals = terminals.zip(0..).collect::<HashMap<_, usize, S>>();
    let mut heap = BucketQueue::default();
    let mut incoming = HashMap::with_hasher(S::default());
    while !terminals.is_empty() {
        incoming.clear();
        heap.clear();
        heap.extend(
            terminals
                .iter()
                .map(|t| (0, (*t.0, Err(*t.1)))),
        );

        let (mut prev, path_cost) = loop {
            let (c, (n, p)) = heap.pop().expect("tree not reachable from terminals");
            if let Entry::Vacant(entry) = incoming.entry(n) {
                entry.insert(p);

                if tree.contains_node(n) {
                    break (n, c);
                }

                for (m, e) in g.neighbours(n) {
                    if !incoming.contains_key(&m) {
                        heap.push(c + cost(e), (m, Ok(n)));
                    }
                }
            }
        };
        tree_cost += path_cost;

        while let &Ok(next) = incoming.get(&prev).unwrap() {
            tree.add_node(next, ());
            tree.add_edge(next, prev, ());
            prev = next;
        }

        terminals.remove(&prev);
    }

    (tree_cost, tree)
}

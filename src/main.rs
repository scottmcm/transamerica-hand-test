#![allow(dead_code)]

use fnv::{FnvHashSet, FnvHashMap};
use itertools::Itertools;
use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::{hash_map::Entry, BinaryHeap};
use std::iter::FromIterator;
use std::sync::Mutex;
use std::time::Instant;

mod data;
use self::data::{make_board, BoardGraph, Position};
mod graph;

fn shortest_path(g: &BoardGraph, a: Position, b: Position) -> (usize, Vec<Position>) {
    shortest_path_any(g, a, |c| c == b)
}

fn shortest_path_any(
    g: &BoardGraph,
    a: Position,
    f: impl Fn(Position) -> bool,
) -> (usize, Vec<Position>) {
    if f(a) {
        return (0, vec![a]);
    }

    let mut distances = FnvHashMap::with_capacity_and_hasher(g.nodes().len(), Default::default());
    let mut heap = BinaryHeap::with_capacity(g.edges().len());
    heap.push((Reverse((0, 0)), (a, a)));
    let (steps, total, last) = loop {
        let (Reverse((d, s)), (n, p)) = heap.pop().unwrap();
        if let Entry::Vacant(entry) = distances.entry(n) {
            entry.insert(p);
            if f(n) {
                //println!("distances from {:?}: {:?}", a, distances);
                //println!("{:?} to {:?} in {} ({} steps)", a, n, d, s);
                break (s, d, n);
            }

            for (m, e) in g.neighbours(n) {
                heap.push((Reverse((d + e.cost, s + 1)), (m, n)));
            }
        }
    };
    let mut path = Vec::with_capacity(steps + 1);
    path.push(last);
    loop {
        let n = path.last().unwrap();
        let p = distances.get(n).unwrap();
        if n == p {
            path.reverse();
            return (total, path);
        }
        path.push(p.to_owned());
    }
}

fn smallest_tree(g: &BoardGraph, ns: &[Position]) -> (usize, FnvHashSet<Position>) {
    let first = ns
        .iter()
        .cloned()
        .map(|p| shortest_path_any(&g, p, |x| x != p && ns.contains(&x)))
        .min()
        .unwrap();
    let mut cost = first.0;
    let mut reached = first.1.into_iter().collect::<FnvHashSet<_>>();
    let mut unreached = ns
        .iter()
        .cloned()
        .filter(|p| !reached.contains(p))
        .collect::<FnvHashSet<_>>();
    while let Some(best) = unreached
        .iter()
        .cloned()
        .map(|p| shortest_path_any(&g, p, |x| x != p && reached.contains(&x)))
        .min()
    {
        //println!("  {:?}", best);
        cost += best.0;
        for n in best.1 {
            unreached.remove(&n);
            reached.insert(n);
        }
    }
    //println!("  cost {:?}", cost);
    (cost, reached)
}

fn main() {
    let start_instant = Instant::now();

    let g = make_board();
    //println!("{} nodes; {} edges", g.node_count(), g.edge_count());
    //println!("{:?}", g);

    // let p = shortest_path(&g, Position(0, 4), Position(3, 4));
    // println!("{:?}", p);

    // let p = shortest_path(&g, Position(3, 4), Position(0, 4));
    // println!("{:?}", p);

    let cities_by_pos = data::CITIES
        .iter()
        .map(|x| (x.pos, *x))
        .collect::<FnvHashMap<_, _>>();

    let hands_by_city = Mutex::new(FnvHashMap::<_, Vec<_>>::with_hasher(Default::default()));
    let mut all_hands = data::hands(None)
        .par_bridge()
        .map(|a| {
            let mut g = g.clone();

            // Don't go through cities you don't own
            for c in data::CITIES {
                if !a.contains(&c.pos) {
                    g.remove_node(c.pos);
                }
            }

            let hand = (smallest_tree(&g, &a), a);
            for c in a.iter().cloned() {
                hands_by_city
                    .lock()
                    .unwrap()
                    .entry(c)
                    .or_default()
                    .push(hand.clone());
            }
            hand
        }).collect::<Vec<_>>();
    let mut hands_by_city = hands_by_city.into_inner().unwrap();

    println!("*** Everything ***");
    all_hands.sort_unstable_by_key(|x| ((x.0).0, x.1));
    let all_stats = all_hands
        .iter()
        .map(|x| (x.0).0 as f64)
        .collect::<ParallelVariance>();
    println!(
        "mean: {:.2} stdev: {:.2}",
        all_stats.mean(),
        all_stats.pop_stdev()
    );
    {
        let best = all_hands.first().unwrap();
        println!(
            "best: {:?} {:?}",
            (best.0).0,
            best.1
                .iter()
                .map(|x| cities_by_pos.get(x).unwrap().name)
                .collect::<Vec<_>>()
        );
        let median = &all_hands[all_hands.len() / 2];
        println!(
            "median: {:?} {:?}",
            (median.0).0,
            median
                .1
                .iter()
                .map(|x| cities_by_pos.get(x).unwrap().name)
                .collect::<Vec<_>>()
        );
        let worst = all_hands.last().unwrap();
        println!(
            "worst: {:?} {:?}",
            (worst.0).0,
            worst
                .1
                .iter()
                .map(|x| cities_by_pos.get(x).unwrap().name)
                .collect::<Vec<_>>()
        );
        println!();
    }

    for c in data::CITIES {
        println!("*** {:?}: {} ({:?}) ***", c.color, c.name, c.pos);
        let hands = hands_by_city.get_mut(&c.pos).unwrap();
        // let avg = hands.iter().map(|x| (x.0).0).sum::<usize>() as f32 / hands.len() as f32;
        // println!("average: {:?}", avg);
        hands.sort_unstable_by_key(|x| ((x.0).0, x.1));
        let stats = hands
            .iter()
            .map(|x| (x.0).0 as f64)
            .collect::<ParallelVariance>();
        println!(
            "mean: {:.2} ({:+.1}) stdev: {:.2} ({:+.1})",
            stats.mean(),
            stats.mean() - all_stats.mean(),
            stats.pop_stdev(),
            stats.pop_stdev() - all_stats.pop_stdev()
        );
        let best = hands.first().unwrap();
        println!(
            "best: {:?} {:?}",
            (best.0).0,
            best.1
                .iter()
                .map(|x| cities_by_pos.get(x).unwrap().name)
                .collect::<Vec<_>>()
        );
        let median = &hands[hands.len() / 2];
        println!(
            "median: {:?} {:?}",
            (median.0).0,
            median
                .1
                .iter()
                .map(|x| cities_by_pos.get(x).unwrap().name)
                .collect::<Vec<_>>()
        );
        let worst = hands.last().unwrap();
        println!(
            "worst: {:?} {:?}",
            (worst.0).0,
            worst
                .1
                .iter()
                .map(|x| cities_by_pos.get(x).unwrap().name)
                .collect::<Vec<_>>()
        );
    }

    let elapsed = start_instant.elapsed();
    eprintln!("Done in {:?}", elapsed);
}

#[derive(Debug, Clone, Copy, Default)]
struct ParallelVariance {
    n: f64,
    x: f64,
    m: f64,
}

impl ParallelVariance {
    fn mean(self) -> f64 {
        self.x
    }

    fn sum(self) -> f64 {
        self.n * self.x
    }

    fn corrected_var(self, c: f64) -> f64 {
        self.m / (self.n - c)
    }

    fn pop_var(self) -> f64 {
        self.corrected_var(0.0)
    }

    fn corrected_stdev(self, c: f64) -> f64 {
        self.corrected_var(c).sqrt()
    }

    fn pop_stdev(self) -> f64 {
        self.corrected_stdev(0.0)
    }

    fn merge(a: Self, b: Self) -> Self {
        let n = a.n + b.n;
        let x = (a.sum() + b.sum()) / n;
        let m = (a.m + b.m) + (a.x - b.x).powi(2) * (a.n * b.n) / n;
        Self { n, x, m }
    }
}

impl From<f64> for ParallelVariance {
    fn from(x: f64) -> Self {
        Self { n: 1.0, x, m: 0.0 }
    }
}

impl FromIterator<f64> for ParallelVariance {
    fn from_iter<I: IntoIterator<Item = f64>>(it: I) -> Self {
        it.into_iter()
            .map(|x| x.into())
            .tree_fold1(ParallelVariance::merge)
            .unwrap_or_default()
    }
}

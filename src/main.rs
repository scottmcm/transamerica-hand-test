#![allow(dead_code)]

use fnv::FnvHashMap;
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::sync::Mutex;
use std::time::Instant;

mod bucket_queue;
mod data;
mod graph;
mod union_find;

fn histogram<T: Ord>(it: impl Iterator<Item = T>) -> BTreeMap<T, usize> {
    let mut counts = BTreeMap::new();
    it.for_each(|x| *counts.entry(x).or_default() += 1);
    counts
}

fn main() {
    let start_instant = Instant::now();

    let g = data::make_board();
    println!(
        "Board has {} nodes & {} edges",
        g.nodes().count(),
        g.edges().count()
    );
    println!();

    let metric_closure = graph::metric_closure_usize(&g, |e| 0 + e.cost);

    let cities_by_pos = data::CITIES
        .iter()
        .map(|x| (x.pos, *x))
        .collect::<FnvHashMap<_, _>>();

    let hands_by_city = Mutex::new(FnvHashMap::<_, Vec<_>>::with_hasher(Default::default()));
    let mut all_hands = data::hands(None)
        .par_bridge()
        .map(|a| {
            let best = g.nodes()
                .map(|(n, _)| n)
                .filter(|n| !a.contains(n))
                .map(|n| {
                    let mut extended_hand = [n; 6];
                    extended_hand[..5].copy_from_slice(&a);
                    let c = graph::kruskal_mst_len_usize(&metric_closure, &extended_hand);
                    (c, n)
                }).min()
                .unwrap();

            //let kmst = graph::kruskal_mst_len_usize(&metric_closure, &a);
            //dbg!((kmst, a));
            //let hand = ((kmst, 0), a);
            let hand = (best, a);
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
        all_stats.stdev_pop()
    );
    {
        let hist = histogram(all_hands.iter().map(|x| (x.0).0));
        println!("histogram: {:?}", hist);
        let best = all_hands.first().unwrap();
        println!(
            "best: {:?} via {:?} {:?}",
            (best.0).0,
            (best.0).1,
            best.1
                .iter()
                .map(|x| cities_by_pos.get(x).unwrap().name)
                .collect::<Vec<_>>()
        );
        let median = &all_hands[all_hands.len() / 2];
        println!(
            "median: {:?} via {:?} {:?}",
            (median.0).0,
            (median.0).1,
            median
                .1
                .iter()
                .map(|x| cities_by_pos.get(x).unwrap().name)
                .collect::<Vec<_>>()
        );
        let worst = all_hands.last().unwrap();
        println!(
            "worst: {:?} via {:?} {:?}",
            (worst.0).0,
            (worst.0).1,
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
            stats.stdev_pop(),
            stats.stdev_pop() - all_stats.stdev_pop(),
        );
        let best = hands.first().unwrap();
        println!(
            "best: {:?} via {:?} {:?}",
            (best.0).0,
            (best.0).1,
            best.1
                .iter()
                .map(|x| cities_by_pos.get(x).unwrap().name)
                .collect::<Vec<_>>()
        );
        let median = &hands[hands.len() / 2];
        println!(
            "median: {:?} via {:?} {:?}",
            (median.0).0,
            (median.0).1,
            median
                .1
                .iter()
                .map(|x| cities_by_pos.get(x).unwrap().name)
                .collect::<Vec<_>>()
        );
        let worst = hands.last().unwrap();
        println!(
            "worst: {:?} via {:?} {:?}",
            (worst.0).0,
            (worst.0).1,
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

    fn var_corrected(self, c: f64) -> f64 {
        self.m / (self.n - c)
    }

    fn var_pop(self) -> f64 {
        self.var_corrected(0.0)
    }

    fn stdev_corrected(self, c: f64) -> f64 {
        self.var_corrected(c).sqrt()
    }

    fn stdev_pop(self) -> f64 {
        self.stdev_corrected(0.0)
    }

    fn standardize_corrected(self, x: f64, c: f64) -> f64 {
        let s = self.stdev_corrected(c);
        (x - self.mean()) / s
    }

    fn standardize_pop(self, x: f64) -> f64 {
        self.standardize_corrected(x, 0.0)
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

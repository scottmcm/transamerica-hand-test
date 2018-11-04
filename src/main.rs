#![allow(dead_code)]

use itertools::{iproduct, Itertools};
use petgraph::graphmap::UnGraphMap;
use std::cmp::Reverse;
use std::collections::{hash_map::Entry, BinaryHeap, HashMap, HashSet};
use std::ops::Add;
use std::time::Instant;

type Graph = UnGraphMap<Node, Edge>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Cost {
    Zero,
    One,
    Two,
    Inf,
}
use self::Cost::*;

impl Add<Cost> for usize {
    type Output = usize;
    fn add(self, other: Cost) -> usize {
        self + match other {
            Zero => 0,
            One => 1,
            Two => 2,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Right,
    UpRight,
    UpLeft,
    Left,
    DownLeft,
    DownRight,
}
use self::Direction::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Color {
    Green,
    Yellow,
    Red,
    Blue,
    Orange,
}
use self::Color::*;

#[derive(Debug, Clone, Copy)]
struct City {
    pos: Node,
    color: Color,
    name: &'static str,
    // true if not used in 2-or-3-player games
    dashed: bool,
}

#[rustfmt::skip]
const CITIES: &[City] = &[
    City { pos: Node(0, 2), color: Green, name: "San Diego", dashed: true },
    City { pos: Node(0, 3), color: Green, name: "Los Angeles", dashed: false },
    City { pos: Node(0, 6), color: Green, name: "San Francisco", dashed: false },
    City { pos: Node(1, 7), color: Green, name: "Sacramento", dashed: false },
    City { pos: Node(2, 9), color: Green, name: "Medford", dashed: false },
    City { pos: Node(3, 11), color: Green, name: "Portland", dashed: false },
    City { pos: Node(4, 12), color: Green, name: "Seattle", dashed: true },

    City { pos: Node(4, 4), color: Yellow, name: "Santa Fe", dashed: false },
    City { pos: Node(4, 8), color: Yellow, name: "Salt Lake City", dashed: false },
    City { pos: Node(6, 7), color: Yellow, name: "Denver", dashed: true },
    City { pos: Node(7, 4), color: Yellow, name: "Oklahoma City", dashed: false },
    City { pos: Node(9, 6), color: Yellow, name: "Kansas City", dashed: true },
    City { pos: Node(9, 8), color: Yellow, name: "Omaha", dashed: false },
    City { pos: Node(11, 6), color: Yellow, name: "St. Louis", dashed: false },

    City { pos: Node(2, 3), color: Red, name: "Phoenix", dashed: false },
    City { pos: Node(3, 1), color: Red, name: "El Paso", dashed: false },
    City { pos: Node(6, 0), color: Red, name: "Houston", dashed: true },
    City { pos: Node(7, 2), color: Red, name: "Dallas", dashed: false },
    City { pos: Node(8, 0), color: Red, name: "New Orleans", dashed: false },
    City { pos: Node(10, 3), color: Red, name: "Memphis", dashed: false },
    City { pos: Node(11, 2), color: Red, name: "Atlanta", dashed: true },

    City { pos: Node(6, 11), color: Blue, name: "Helena", dashed: false },
    City { pos: Node(10, 11), color: Blue, name: "Bismark", dashed: false },
    City { pos: Node(12, 10), color: Blue, name: "Minneapolis", dashed: false },
    City { pos: Node(13, 11), color: Blue, name: "Duluth", dashed: true },
    City { pos: Node(14, 7), color: Blue, name: "Cincinnati", dashed: false },
    City { pos: Node(14, 9), color: Blue, name: "Chicago", dashed: false },
    City { pos: Node(17, 10), color: Blue, name: "Buffalo", dashed: true },

    City { pos: Node(11, 0), color: Orange, name: "Jacksonville", dashed: false },
    City { pos: Node(13, 2), color: Orange, name: "Charleston", dashed: false },
    City { pos: Node(13, 4), color: Orange, name: "Winston", dashed: false },
    City { pos: Node(15, 5), color: Orange, name: "Richmond", dashed: true },
    City { pos: Node(16, 7), color: Orange, name: "Washington", dashed: false },
    City { pos: Node(17, 8), color: Orange, name: "New York", dashed: false },
    City { pos: Node(19, 10), color: Orange, name: "Boston", dashed: true },
];

fn cities(c: Color, d: Option<bool>) -> impl Iterator<Item = Node> + Clone {
    CITIES
        .iter()
        .filter(move |x| x.color == c && (d.is_none() || Some(x.dashed) == d))
        .map(|x| x.pos)
}

fn hands(d: Option<bool>) -> impl Iterator<Item = [Node; 5]> + Clone {
    iproduct!(
        cities(Green, d),
        cities(Red, d),
        cities(Yellow, d),
        cities(Blue, d),
        cities(Orange, d)
    ).map(|(c0, c1, c2, c3, c4)| [c0, c1, c2, c3, c4])
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(align(2))]
struct Node(u8, u8);

impl Add<Direction> for Node {
    type Output = Node;
    fn add(self, d: Direction) -> Node {
        let Node(x, y) = self;
        match d {
            Right => Node(x + 1, y),
            UpRight => Node(x + 1, y + 1),
            UpLeft => Node(x, y + 1),
            Left => Node(x - 1, y),
            DownLeft => Node(x - 1, y - 1),
            DownRight => Node(x, y - 1),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    cost: Cost,
}

fn make_board() -> Graph {
    let mut g = Graph::with_capacity(188, 509);
    let columns: &[&[_]] = &[
        // 0
        &[
            [Inf, Inf, Inf], // 0
            [Inf, Inf, Inf], // 1
            [Two, One, One], // 2
            [Two, One, One], // 3
            [Two, One, One], // 4
            [Two, Two, One], // 5
            [One, One, Inf], // 6
        ],
        // 1
        &[
            [Inf, Inf, Inf], // 0
            [One, One, One], // 1
            [One, One, Two], // 2
            [Two, Two, One], // 3
            [Two, One, One], // 4
            [One, One, One], // 5
            [Two, One, Two], // 6
            [Two, Two, One], // 7
            [Two, One, Inf], // 8
        ],
        // 2
        &[
            [Inf, Inf, Inf], // 0
            [One, One, One], // 1
            [One, Two, One], // 2
            [Two, Two, One], // 3
            [Two, One, Two], // 4
            [Two, One, One], // 5
            [One, One, Two], // 6
            [Two, Two, One], // 7
            [One, One, Two], // 8
            [Two, Two, One], // 9
            [Two, One, Inf], // 10
        ],
        // 3
        &[
            [One, One, One], // 0
            [One, One, One], // 1
            [One, One, Two], // 2
            [One, One, One], // 3
            [One, One, One], // 4
            [One, One, Two], // 5
            [Two, One, One], // 6
            [One, One, One], // 7
            [One, One, One], // 8
            [One, One, One], // 9
            [One, One, Two], // 10
            [Two, One, Inf], // 11
        ],
        // 4
        &[
            [One, One, One], // 0
            [One, One, One], // 1
            [One, One, One], // 2
            [One, One, One], // 3
            [One, One, One], // 4
            [Two, One, One], // 5
            [One, Two, Two], // 6
            [One, Two, One], // 7
            [One, One, One], // 8
            [One, One, One], // 9
            [One, One, One], // 10
            [One, One, Two], // 11
            [Two, Inf, Inf], // 12
        ],
        // 5
        &[
            [One, One, One], // 0
            [One, One, One], // 1
            [One, One, One], // 2
            [One, One, One], // 3
            [One, One, One], // 4
            [One, One, Two], // 5
            [Two, Two, One], // 6
            [Two, One, One], // 7
            [One, One, One], // 8
            [One, One, One], // 9
            [Two, Two, One], // 10
            [Two, Two, One], // 11
            [One, Inf, Inf], // 12
        ],
        // 6
        &[
            [One, One, One], // 0
            [One, One, One], // 1
            [One, One, One], // 2
            [One, One, One], // 3
            [One, One, One], // 4
            [One, One, One], // 5
            [One, One, One], // 6
            [One, One, One], // 7
            [One, One, One], // 8
            [Two, Two, One], // 9
            [Two, Two, One], // 10
            [Two, One, One], // 11
            [One, Inf, Inf], // 12
        ],
        // 7
        &[
            [Two, One, One], // 0
            [One, One, One], // 1
            [One, One, One], // 2
            [One, One, One], // 3
            [One, One, One], // 4
            [One, One, One], // 5
            [One, One, One], // 6
            [One, One, One], // 7
            [One, One, One], // 8
            [One, One, One], // 9
            [One, One, One], // 10
            [One, Two, One], // 11
            [One, Inf, Inf], // 12
        ],
        // 8
        &[
            [One, One, Two], // 0
            [Two, Two, One], // 1
            [Two, One, One], // 2
            [One, One, One], // 3
            [One, One, One], // 4
            [One, One, One], // 5
            [One, One, One], // 6
            [One, One, One], // 7
            [One, One, One], // 8
            [One, One, One], // 9
            [One, One, One], // 10
            [One, Two, Two], // 11
            [One, Inf, Inf], // 12
        ],
        // 9
        &[
            [One, One, One], // 0
            [One, One, One], // 1
            [One, One, Two], // 2
            [Two, One, One], // 3
            [One, One, One], // 4
            [One, One, One], // 5
            [One, Two, One], // 6
            [Two, Two, One], // 7
            [Two, Two, One], // 8
            [Two, Two, One], // 9
            [Two, Two, One], // 10
            [Two, Two, Two], // 11
            [One, Inf, Inf], // 12
        ],
        // 10
        &[
            [One, One, One], // 0
            [One, One, One], // 1
            [One, One, One], // 2
            [One, One, Two], // 3
            [Two, One, One], // 4
            [One, One, One], // 5
            [One, Two, Two], // 6
            [One, One, One], // 7
            [One, One, One], // 8
            [One, One, One], // 9
            [One, One, One], // 10
            [One, One, One], // 11
            [One, Inf, Inf], // 12
        ],
        // 11
        &[
            [Inf, One, One], // 0
            [One, One, One], // 1
            [One, One, One], // 2
            [Two, One, One], // 3
            [One, One, Two], // 4
            [Two, Two, One], // 5
            [Two, Two, Two], // 6
            [Two, Two, One], // 7
            [Two, One, One], // 8
            [One, One, One], // 9
            [One, One, One], // 10
            [One, One, One], // 11
            [One, Inf, Inf], // 12
        ],
        // 12
        &[
            [Inf, Inf, Inf], // 0
            [Inf, One, One], // 1
            [One, One, One], // 2
            [One, One, Two], // 3
            [Two, One, One], // 4
            [One, One, Two], // 5
            [Two, One, One], // 6
            [One, One, One], // 7
            [One, One, Two], // 8
            [Two, Two, One], // 9
            [Two, Two, One], // 10
            [Two, One, One], // 11
            [One, Inf, Inf], // 12
        ],
        // 13
        &[
            [Inf, Inf, Inf], // 0
            [Inf, Inf, Inf], // 1
            [Inf, One, One], // 2
            [One, One, One], // 3
            [One, One, Two], // 4
            [Two, One, One], // 5
            [One, Two, Two], // 6
            [One, One, One], // 7
            [One, One, One], // 8
            [One, One, One], // 9
            [One, One, One], // 10
            [One, Inf, One], // 11
        ],
        // 14
        &[
            [Inf, Inf, Inf], // 0
            [Inf, Inf, Inf], // 1
            [Inf, Inf, Inf], // 2
            [Inf, One, One], // 3
            [One, One, One], // 4
            [One, One, Two], // 5
            [Two, One, Two], // 6
            [One, One, One], // 7
            [One, One, One], // 8
            [One, Inf, One], // 9
            [Inf, Inf, One], // 10
        ],
        // 15
        &[
            [Inf, Inf, Inf], // 0
            [Inf, Inf, Inf], // 1
            [Inf, Inf, Inf], // 2
            [Inf, Inf, Inf], // 3
            [Inf, Inf, One], // 4
            [Inf, Inf, One], // 5
            [Inf, One, Two], // 6
            [Two, One, One], // 7
            [One, One, One], // 8
            [One, Inf, Inf], // 9
        ],
        // 16
        &[
            [Inf, Inf, Inf], // 0
            [Inf, Inf, Inf], // 1
            [Inf, Inf, Inf], // 2
            [Inf, Inf, Inf], // 3
            [Inf, Inf, Inf], // 4
            [Inf, Inf, Inf], // 5
            [Inf, Inf, Inf], // 6
            [Inf, One, Two], // 7
            [Two, One, One], // 8
            [One, One, Inf], // 9
        ],
        // 17
        &[
            [Inf, Inf, Inf], // 0
            [Inf, Inf, Inf], // 1
            [Inf, Inf, Inf], // 2
            [Inf, Inf, Inf], // 3
            [Inf, Inf, Inf], // 4
            [Inf, Inf, Inf], // 5
            [Inf, Inf, Inf], // 6
            [Inf, Inf, Inf], // 7
            [Inf, One, Two], // 8
            [One, One, One], // 9
            [One, One, Inf], // 10
        ],
        // 18
        &[
            [Inf, Inf, Inf], // 0
            [Inf, Inf, Inf], // 1
            [Inf, Inf, Inf], // 2
            [Inf, Inf, Inf], // 3
            [Inf, Inf, Inf], // 4
            [Inf, Inf, Inf], // 5
            [Inf, Inf, Inf], // 6
            [Inf, Inf, Inf], // 7
            [Inf, Inf, Inf], // 8
            [Inf, One, One], // 9
            [One, One, One], // 10
            [One, Inf, Inf], // 11
        ],
        // 19
        &[
            [Inf, Inf, Inf], // 0
            [Inf, Inf, Inf], // 1
            [Inf, Inf, Inf], // 2
            [Inf, Inf, Inf], // 3
            [Inf, Inf, Inf], // 4
            [Inf, Inf, Inf], // 5
            [Inf, Inf, Inf], // 6
            [Inf, Inf, Inf], // 7
            [Inf, Inf, Inf], // 8
            [Inf, Inf, Inf], // 9
            [Inf, Inf, One], // 10
        ],
    ];
    for (c, i) in columns.iter().zip(0..) {
        for (r, j) in c.iter().zip(0..) {
            let n = Node(i, j);
            for (k, d) in [Right, UpRight, UpLeft].iter().cloned().enumerate() {
                if r[k] != Inf {
                    //println!("{:?} <-> {:?}: {:?}", n, n + d, r[k]);
                    g.add_edge(n, n + d, Edge { cost: r[k] });
                }
            }
        }
    }
    g
}

fn shortest_path(g: &Graph, a: Node, b: Node) -> (usize, Vec<Node>) {
    shortest_path_any(g, a, |c| c == b)
}

fn shortest_path_any(g: &Graph, a: Node, f: impl Fn(Node) -> bool) -> (usize, Vec<Node>) {
    if f(a) {
        return (0, vec![a]);
    }

    let mut distances = HashMap::with_capacity(g.node_count());
    let mut heap = BinaryHeap::with_capacity(g.edge_count());
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

            for (_, m, e) in g.edges(n) {
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

fn smallest_tree(g: &Graph, ns: &[Node]) -> (usize, HashSet<Node>) {
    let first = ns
        .iter()
        .cloned()
        .map(|p| shortest_path_any(&g, p, |x| x != p && ns.contains(&x)))
        .min()
        .unwrap();
    let mut cost = first.0;
    let mut reached = first.1.into_iter().collect::<HashSet<_>>();
    let mut unreached = ns
        .iter()
        .cloned()
        .filter(|p| !reached.contains(p))
        .collect::<HashSet<_>>();
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

    // let p = shortest_path(&g, Node(0, 4), Node(3, 4));
    // println!("{:?}", p);

    // let p = shortest_path(&g, Node(3, 4), Node(0, 4));
    // println!("{:?}", p);

    let cities_by_pos = CITIES
        .iter()
        .map(|x| (x.pos, *x))
        .collect::<HashMap<_, _>>();

    let mut all_hands = Vec::new();
    let mut hands_by_city = HashMap::<_, Vec<_>>::new();
    for a in hands(None) {
        let mut g = g.clone();

        // Don't go through cities you don't own
        for c in CITIES {
            if !a.contains(&c.pos) {
                g.remove_node(c.pos);
            }
        }

        let hand = (smallest_tree(&g, &a), a);
        for c in a.iter().cloned() {
            hands_by_city.entry(c).or_default().push(hand.clone());
        }
        all_hands.push(hand);
    }

    println!("*** Everything ***");
    all_hands.sort_unstable_by_key(|x| (x.0).0);
    let all_stats = all_hands
        .iter()
        .map(|x| ((x.0).0 as f64).into())
        .tree_fold1(ParallelVariance::merge)
        .unwrap();
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
    }

    for c in CITIES {
        println!("*** {:?}: {} ({:?}) ***", c.color, c.name, c.pos);
        let hands = hands_by_city.get_mut(&c.pos).unwrap();
        // let avg = hands.iter().map(|x| (x.0).0).sum::<usize>() as f32 / hands.len() as f32;
        // println!("average: {:?}", avg);
        hands.sort_unstable_by_key(|x| (x.0).0);
        let stats = hands
            .iter()
            .map(|x| ((x.0).0 as f64).into())
            .tree_fold1(ParallelVariance::merge)
            .unwrap();
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
    println!("Done in {:?}", elapsed);
}

#[derive(Debug, Clone, Copy)]
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

    fn pop_stdev(self) -> f64 {
        self.m / self.n
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

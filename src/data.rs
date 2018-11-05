use itertools::iproduct;
use std::hash::{Hash, Hasher};
use std::ops::Add;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Color {
    Green,
    Yellow,
    Red,
    Blue,
    Orange,
}
use self::Color::*;

#[derive(Debug, Clone, Copy)]
pub struct City {
    pub pos: Position,
    pub color: Color,
    pub name: &'static str,
    // true if not used in 2-or-3-player games
    pub dashed: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(align(2))]
pub struct Position(pub u8, pub u8);

impl Hash for Position {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_u16(((self.0 as u16) << 8) | self.1 as u16);
    }
}

#[rustfmt::skip]
pub const CITIES: &[City] = &[
    City { pos: Position(0, 2), color: Green, name: "San Diego", dashed: true },
    City { pos: Position(0, 3), color: Green, name: "Los Angeles", dashed: false },
    City { pos: Position(0, 6), color: Green, name: "San Francisco", dashed: false },
    City { pos: Position(1, 7), color: Green, name: "Sacramento", dashed: false },
    City { pos: Position(2, 9), color: Green, name: "Medford", dashed: false },
    City { pos: Position(3, 11), color: Green, name: "Portland", dashed: false },
    City { pos: Position(4, 12), color: Green, name: "Seattle", dashed: true },

    City { pos: Position(4, 4), color: Yellow, name: "Santa Fe", dashed: false },
    City { pos: Position(4, 8), color: Yellow, name: "Salt Lake City", dashed: false },
    City { pos: Position(6, 7), color: Yellow, name: "Denver", dashed: true },
    City { pos: Position(7, 4), color: Yellow, name: "Oklahoma City", dashed: false },
    City { pos: Position(9, 6), color: Yellow, name: "Kansas City", dashed: true },
    City { pos: Position(9, 8), color: Yellow, name: "Omaha", dashed: false },
    City { pos: Position(11, 6), color: Yellow, name: "St. Louis", dashed: false },

    City { pos: Position(2, 3), color: Red, name: "Phoenix", dashed: false },
    City { pos: Position(3, 1), color: Red, name: "El Paso", dashed: false },
    City { pos: Position(6, 0), color: Red, name: "Houston", dashed: true },
    City { pos: Position(7, 2), color: Red, name: "Dallas", dashed: false },
    City { pos: Position(8, 0), color: Red, name: "New Orleans", dashed: false },
    City { pos: Position(10, 3), color: Red, name: "Memphis", dashed: false },
    City { pos: Position(11, 2), color: Red, name: "Atlanta", dashed: true },

    City { pos: Position(6, 11), color: Blue, name: "Helena", dashed: false },
    City { pos: Position(10, 11), color: Blue, name: "Bismark", dashed: false },
    City { pos: Position(12, 10), color: Blue, name: "Minneapolis", dashed: false },
    City { pos: Position(13, 11), color: Blue, name: "Duluth", dashed: true },
    City { pos: Position(14, 7), color: Blue, name: "Cincinnati", dashed: false },
    City { pos: Position(14, 9), color: Blue, name: "Chicago", dashed: false },
    City { pos: Position(17, 10), color: Blue, name: "Buffalo", dashed: true },

    City { pos: Position(11, 0), color: Orange, name: "Jacksonville", dashed: false },
    City { pos: Position(13, 2), color: Orange, name: "Charleston", dashed: false },
    City { pos: Position(13, 4), color: Orange, name: "Winston", dashed: false },
    City { pos: Position(15, 5), color: Orange, name: "Richmond", dashed: true },
    City { pos: Position(16, 7), color: Orange, name: "Washington", dashed: false },
    City { pos: Position(17, 8), color: Orange, name: "New York", dashed: false },
    City { pos: Position(19, 10), color: Orange, name: "Boston", dashed: true },
];

fn cities(c: Color, d: Option<bool>) -> impl DoubleEndedIterator<Item = Position> + Clone {
    CITIES
        .iter()
        .filter(move |x| x.color == c && (d.is_none() || Some(x.dashed) == d))
        .map(|x| x.pos)
}

pub fn hands(d: Option<bool>) -> impl Iterator<Item = [Position; 5]> + Clone {
    iproduct!(
        cities(Green, d),
        cities(Red, d),
        cities(Yellow, d),
        cities(Blue, d),
        cities(Orange, d)
    ).map(|(c0, c1, c2, c3, c4)| [c0, c1, c2, c3, c4])
}

pub type BoardGraph = crate::graph::UnGraph<Position, (), Edge, fnv::FnvBuildHasher>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Cost {
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

impl Add<Direction> for Position {
    type Output = Position;
    fn add(self, d: Direction) -> Position {
        let Position(x, y) = self;
        match d {
            Right => Position(x + 1, y),
            UpRight => Position(x + 1, y + 1),
            UpLeft => Position(x, y + 1),
            Left => Position(x - 1, y),
            DownLeft => Position(x - 1, y - 1),
            DownRight => Position(x, y - 1),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub cost: Cost,
}

pub fn make_board() -> BoardGraph {
    let mut g = BoardGraph::with_capacity(188, 509);
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
            let n = Position(i, j);
            for (k, d) in [Right, UpRight, UpLeft].iter().cloned().enumerate() {
                if r[k] != Inf {
                    //println!("{:?} <-> {:?}: {:?}", n, n + d, r[k]);
                    g.try_add_node(n, ());
                    g.try_add_node(n + d, ());
                    g.add_edge(n, n + d, Edge { cost: r[k] });
                }
            }
        }
    }
    g
}

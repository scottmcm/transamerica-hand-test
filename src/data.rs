use std::hash::{Hash, Hasher};
use itertools::iproduct;

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

fn cities(c: Color, d: Option<bool>) -> impl Iterator<Item = Position> + Clone {
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

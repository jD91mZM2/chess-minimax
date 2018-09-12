#[macro_use] extern crate failure;

use std::fmt;

pub mod board;
pub mod minimax;
pub mod piece;
pub(crate) mod utils;

/// A position on the board
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos(pub i8, pub i8);
impl Pos {
    /// Returns true if the position is actually within the boundaries of a board
    pub fn is_valid(self) -> bool {
        let Pos(x, y) = self;
        x >= 0 && x < board::WIDTH
            && y >= 0 && y < board::WIDTH
    }
}
macro_rules! impl_op {
    ($($trait:ident, $fn:ident, $op:tt, $trait_assign:ident, $fn_assign:ident, $op_assign:tt;)*) => {
        $(impl std::ops::$trait<Pos> for Pos {
            type Output = Self;

            fn $fn(self, other: Self) -> Self::Output {
                let Pos(x1, y1) = self;
                let Pos(x2, y2) = other;
                Pos(
                    x1 $op x2,
                    y1 $op y2
                )
            }
        }
        impl std::ops::$trait_assign<Pos> for Pos {
            fn $fn_assign(&mut self, other: Self) {
                let Pos(ref mut x1, ref mut y1) = self;
                let Pos(x2, y2) = other;

                *x1 $op_assign x2;
                *y1 $op_assign y2;
            }
        })*
    }
}
impl_op! {
    Add, add, +, AddAssign, add_assign, +=;
    Sub, sub, -, SubAssign, sub_assign, -=;
}
impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        assert!(self.is_valid());
        let Pos(x, y) = *self;
        write!(f, "{}{}", ('A' as u8 + x as u8) as char, board::WIDTH-y)
    }
}
/// An error parsing a position from a string
#[derive(Debug, Fail)]
#[fail(display = "invalid position string")]
pub struct ParsePosError;
impl std::str::FromStr for Pos {
    type Err = ParsePosError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let x = match chars.next() {
            Some(c @ 'a'..='h') => c as u8 - b'a',
            Some(c @ 'A'..='H') => c as u8 - b'A',
            _ => return Err(ParsePosError)
        };
        let y = match chars.next() {
            Some(c @ '1'..='8') => b'8' - c as u8,
            _ => return Err(ParsePosError)
        };
        Ok(Pos(x as i8, y as i8))
    }
}

/// What side a piece belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Black,
    White
}
impl std::ops::Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Side::Black => Side::White,
            Side::White => Side::Black
        }
    }
}

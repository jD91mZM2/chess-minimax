use std::fmt;

pub mod board;
pub mod piece;
crate mod utils;

#[derive(Debug, Default, Clone, Copy)]
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
            type Output = Pos;

            fn $fn(self, other: Pos) -> Self::Output {
                let Pos(x1, y1) = self;
                let Pos(x2, y2) = other;
                Pos(
                    x1 $op x2,
                    y1 $op y2
                )
            }
        }
        impl std::ops::$trait_assign<Pos> for Pos {
            fn $fn_assign(&mut self, other: Pos) {
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

/// What side a piece belongs to
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Black,
    White
}

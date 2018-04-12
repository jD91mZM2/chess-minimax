#![cfg_attr(feature = "threaded", feature(integer_atomics))]

#[cfg(feature = "websocket")] extern crate websocket;
#[cfg(feature = "cpuprofiler")] extern crate cpuprofiler;
#[cfg(feature = "cpuprofiler")] use cpuprofiler::PROFILER;

#[cfg(feature = "websocket")] pub mod ws;
#[cfg(not(feature = "websocket"))] pub mod input;
pub mod board;
pub mod piece;
pub mod search;

pub use board::*;
pub use piece::*;
pub use search::*;

#[derive(Debug)]
pub enum Direction {
    UpLeft,
    UpRight,
    LeftUp,
    LeftDown,
    RightUp,
    RightDown,
    DownLeft,
    DownRight,
}

pub const DIRECTIONS_ALL: [Direction; 8] = [
    Direction::UpLeft,
    Direction::UpRight,
    Direction::LeftUp,
    Direction::LeftDown,
    Direction::RightUp,
    Direction::RightDown,
    Direction::DownLeft,
    Direction::DownRight
];


pub fn rotate(rel: Pos, direction: &Direction) -> Pos {
    // Assumes current rotation is DownRight

    let (x, y) = rel;
    match *direction {
        Direction::UpLeft => (-x, -y),
        Direction::UpRight => (x, -y),
        Direction::LeftUp => (-y, -x),
        Direction::LeftDown => (y, -x),
        Direction::RightUp => (-y, x),
        Direction::RightDown => (y, x),
        Direction::DownLeft => (-x, y),
        Direction::DownRight => (x, y),
    }
}

pub fn position_string(input: Pos) -> String {
    let mut output = String::with_capacity(2);
    #[cfg(not(feature = "white"))]
    output.push(std::char::from_u32((7 - input.0) as u32 + 'A' as u32).unwrap());
    #[cfg(feature = "white")]
    output.push(std::char::from_u32(input.0 as u32 + 'A' as u32).unwrap());

    #[cfg(not(feature = "white"))]
    output.push(std::char::from_u32(input.1 as u32 + '1' as u32).unwrap());
    #[cfg(feature = "white")]
    output.push(std::char::from_u32((7 - input.1) as u32 + '1' as u32).unwrap());

    output
}
pub fn parse_position(input: &str) -> Option<Pos> {
    let (mut x, mut y) = (None, None);

    for c in input.chars() {
        let code = c as u32;

        // The following blocks are required
        // because #[cfg]s on expressions are
        // apparently experimental.

        if code >= 'a' as u32 && code <= 'h' as u32 {
            #[cfg(not(feature = "white"))]
            { x = Some(('h' as u32 - code) as i8); }
            #[cfg(feature = "white")]
            { x = Some((code - 'a' as u32) as i8); }
        } else if code >= 'A' as u32 && code <= 'H' as u32 {
            #[cfg(not(feature = "white"))]
            { x = Some(('H' as u32 - code) as i8); }
            #[cfg(feature = "white")]
            { x = Some((code - 'A' as u32) as i8); }
        } else if code >= '1' as u32 && code <= '8' as u32 {
            #[cfg(not(feature = "white"))]
            { y = Some((code - '1' as u32) as i8); }
            #[cfg(feature = "white")]
            { y = Some(('8' as u32 - code) as i8); }
        } else {
            return None;
        }
    }

    if x.is_none() || y.is_none() {
        return None;
    }
    Some((x.unwrap(), y.unwrap()))
}

pub fn main() {
    #[cfg(all(feature = "websocket", feature = "cpuprofiler"))]
    compile_error!("Oh no, you can't have both websocket and cpuprofiler.");

    #[cfg(all(feature = "public", not(feature = "websocket")))]
    compile_error!("Oh no, you can't have public without websocket.");

    #[cfg(not(feature = "websocket"))]
    input::main();
    #[cfg(feature = "websocket")]
    ws::main();
}

use *;

const MAX_DEPTH: u8 = 4;

pub fn score(board: &Board) -> i32 {
	let mut score = 0;
	for line in board {
		for piece in line {
			if piece.is_mine() {
				score += piece.worth() as i32;
			} else {
				score -= piece.worth() as i32;
			}
		}
	}

	score
}
pub fn search(board: &mut Board, mine: bool, depth: u8, mut alpha: i32, mut beta: i32) -> (i32, Pos, Pos) {
	#[cfg(feature = "cache")]
	let mut bytes = None;
	#[cfg(feature = "cache")]
	{
		if depth == 0 {
			bytes = Some(board_bytes(&board));
			if let Ok((from, to)) = read_move(bytes.as_ref().unwrap()) {
				return (0, from, to);
			}
		}
	}

	let mut myking   = false;
	let mut yourking = false;
	for line in &*board {
		for piece in line {
			if let Piece::King(mine) = *piece {
				if mine {
					myking = true;
				} else {
					yourking = true;
				}
			}
		}
	}

	if !myking {
		// Play for as long as possible
		return (-(999 + depth as i32), (0, 0), (0, 0));
	} else if !yourking {
		// Play for as short as possible
		return (999 - depth as i32, (0, 0), (0, 0));
	}

	// I used to make my king worth 1000 and your king worth 100.
	// But then I realized:
	// If the game ended, don't go any further.

	if depth > MAX_DEPTH {
		return (score(board), (0, 0), (0, 0));
	}
	let possible = possible_moves(board, mine);

	let mut max_or_min = if mine { std::i32::MIN } else { std::i32::MAX };
	let mut selected   = ((0, 0), (0, 0));
	let mut found      = false;

	for (old, moves2) in possible {
		for new in &moves2 {
			let new = *new;
			let score;

			let (diff, _) = board_move(board, old, new);

			score = search(board, !mine, depth + 1, alpha, beta).0;

			board_apply(board, diff); // undo

			if (mine && score > max_or_min) || (!mine && score < max_or_min) {
				max_or_min = score;
				selected   = (old, new);
				found      = true;

				if mine && max_or_min > alpha {
					alpha = max_or_min;
				} else if !mine && max_or_min < beta {
					beta = max_or_min;
				}
				if beta <= alpha {
					break;
				}
			}
		}
	}

	if found {
		#[cfg(feature = "cache")]
		{
			if depth == 0 {
				// let _ = write_move(bytes.as_ref().unwrap(), selected.0, selected.1);
				write_move(bytes.as_ref().unwrap(), selected.0, selected.1).unwrap();
			}
		}
		(max_or_min, selected.0, selected.1)
	} else {
		(score(board), (0, 0), (0, 0))
	}
}

#[cfg(feature = "cache")]
#[derive(Debug)]
pub struct CorruptedFileError;

#[cfg(feature = "cache")]
impl std::error::Error for CorruptedFileError {
	fn description(&self) -> &str {
		"Corrupted move file"
	}
}
#[cfg(feature = "cache")]
impl std::fmt::Display for CorruptedFileError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		use std::error::Error;
		write!(f, "{}", self.description())
	}
}

#[cfg(feature = "cache")]
fn read_move(board: &[u8; 64]) -> Result<(Pos, Pos), Box<std::error::Error>> {
	use std::ffi::OsStr;
	use std::fs::File;
	use std::io::Read;
	use std::os::unix::ffi::OsStrExt;
	use std::path::Path;

	let path = Path::new("cache").join(OsStr::from_bytes(board));
	let mut bytes = Vec::new();
	{
		let mut file = File::open(path)?;
		file.read_to_end(&mut bytes)?;
	}

	if bytes.len() != 4 {
		return Err(Box::new(CorruptedFileError));
	}
	if !bytes.iter().all(|i| *i < 8) {
		return Err(Box::new(CorruptedFileError));
	}

	Ok(((bytes[0] as i8, bytes[1] as i8), (bytes[2] as i8, bytes[3] as i8)))
}
#[cfg(feature = "cache")]
fn write_move(board: &[u8; 64], from: Pos, to: Pos) -> Result<(), Box<std::error::Error>> {
	use std::ffi::OsStr;
	use std::fs::{self, OpenOptions};
	use std::io::Write;
	use std::os::unix::ffi::OsStrExt;
	use std::path::Path;

	// This will probably not even compile on Windows.
	// Guess what? I don't even care.

	let _ = fs::create_dir("cache");
	let path = Path::new("cache").join(OsStr::from_bytes(board));

	let mut file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(path)?;
	file.write_all(&[from.0 as u8, from.1 as u8, to.0 as u8, to.1 as u8])?;

	Ok(())
}

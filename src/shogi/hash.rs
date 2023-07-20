use super::consts::{BLACK_PIECES, BLACK_POCKETS, WHITE_PIECES, WHITE_POCKETS};
use super::piece;

pub fn get_piece_hash(piece: i8, cell: usize) -> u64 {
  assert_ne!(piece, piece::NONE);
  let q = if piece > 0 {
    &BLACK_PIECES
  } else {
    &WHITE_PIECES
  };
  let piece = piece.abs();
  if piece >= piece::PROMOTED {
    q[cell] ^ q[81 * (piece - piece::PROMOTED) as usize + cell]
  } else {
    q[81 * piece as usize + cell]
  }
}

fn get_pocket_hash(p: &[u64], abs_piece: i8, c: u8) -> u64 {
  assert!(abs_piece > 0);
  assert!(c > 0);
  if abs_piece == piece::PAWN {
    p[23 + (c as usize)]
  } else {
    p[4 * (abs_piece - 2) as usize + (c as usize) - 1]
  }
}

pub fn get_black_pocket_hash(abs_piece: i8, c: u8) -> u64 {
  get_pocket_hash(&BLACK_POCKETS, abs_piece, c)
}

pub fn get_cumulative_black_pocket_hash(abs_piece: i8, c: u8) -> u64 {
  (1..=c).fold(0, |acc, i| {
    acc ^ get_pocket_hash(&BLACK_POCKETS, abs_piece, i)
  })
}

pub fn get_white_pocket_hash(abs_piece: i8, c: u8) -> u64 {
  get_pocket_hash(&WHITE_POCKETS, abs_piece, c)
}

pub fn get_cumulative_white_pocket_hash(abs_piece: i8, c: u8) -> u64 {
  (1..=c).fold(0, |acc, i| {
    acc ^ get_pocket_hash(&WHITE_POCKETS, abs_piece, i)
  })
}

pub fn compute_black_pockets_hash(black_pockets: &[u8]) -> u64 {
  black_pockets
    .iter()
    .enumerate()
    .skip(1)
    .filter(|&(_, &c)| c > 0)
    .fold(0, |acc, (p, &c)| {
      acc ^ get_cumulative_black_pocket_hash(p as i8, c)
    })
}

pub fn compute_white_pockets_hash(white_pockets: &[u8]) -> u64 {
  white_pockets
    .iter()
    .enumerate()
    .skip(1)
    .filter(|&(_, &c)| c > 0)
    .fold(0, |acc, (p, &c)| {
      acc ^ get_cumulative_white_pocket_hash(p as i8, c)
    })
}

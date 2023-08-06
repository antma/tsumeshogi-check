use super::{consts, hash, piece};

pub fn find_king_position(board: &[i8], s: i8) -> Option<usize> {
  let king = super::piece::KING * s;
  board
    .iter()
    .enumerate()
    .find_map(|(i, v)| if *v == king { Some(i) } else { None })
}

pub fn count_pieces(board: &[i8]) -> (Vec<u32>, Vec<u32>) {
  let mut b = vec![0u32; 9];
  let mut w = b.clone();
  for p in board.iter() {
    if *p == 0 {
      continue;
    }
    if *p > 0 {
      b[piece::unpromote(*p) as usize] += 1;
    } else {
      w[(-piece::unpromote(*p)) as usize] += 1;
    }
  }
  (b, w)
}

pub fn compute_hash(board: &[i8]) -> u64 {
  board
    .iter()
    .enumerate()
    .filter(|(_, &p)| p != piece::NONE)
    .fold(0, |acc, (cell, p)| acc ^ hash::get_piece_hash(*p, cell))
}

pub fn compute_all_pieces(board: &[i8]) -> (u128, u128, u128, u128, u128, u128) {
  board
    .iter()
    .enumerate()
    .filter(|(_, p)| **p != piece::NONE)
    .fold((0, 0, 0, 0, 0, 0), |acc, (i, p)| {
      let bit = 1u128 << i;
      (
        acc.0 | bit,
        acc.1 | consts::MASKS2[i],
        acc.2 | consts::MASKS3[i],
        acc.3 | consts::MASKS4[i],
        if *p > 0 { acc.4 | bit } else { acc.4 },
        if *p < 0 { acc.5 | bit } else { acc.5 },
      )
    })
}

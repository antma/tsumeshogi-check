use super::hash;
use super::piece;

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

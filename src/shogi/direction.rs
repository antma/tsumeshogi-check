use super::{
  cell::unpack,
  piece::{flags_to_drop_mask, flags_to_mask},
};
pub type Direction = (isize, isize);
pub const SILVER_MOVES: [Direction; 5] = [(-1, -1), (-1, 0), (-1, 1), (1, -1), (1, 1)];
pub const GOLD_MOVES: [Direction; 6] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, 0)];
pub const ROOK_MOVES: [Direction; 4] = [(-1, 0), (0, -1), (0, 1), (1, 0)];
pub const BISHOP_MOVES: [Direction; 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
pub const KING_MOVES: [Direction; 8] = [
  (-1, -1),
  (-1, 0),
  (-1, 1),
  (0, -1),
  (0, 1),
  (1, -1),
  (1, 0),
  (1, 1),
];
pub const OFFSETS: [isize; 8] = [-10, -9, -8, -1, 1, 8, 9, 10];
//flags: +1 - bishop
//flags: +2 - rook
//flags: +4 - general (forward)
pub const BLACK_MASKS: [(u32, u32); 8] = [
  flags_to_mask(1 + 4),
  flags_to_mask(2 + 4),
  flags_to_mask(1 + 4),
  flags_to_mask(2),
  flags_to_mask(2),
  flags_to_mask(1),
  flags_to_mask(2),
  flags_to_mask(1),
];
pub const WHITE_MASKS: [(u32, u32); 8] = [
  flags_to_mask(1),
  flags_to_mask(2),
  flags_to_mask(1),
  flags_to_mask(2),
  flags_to_mask(2),
  flags_to_mask(1 + 4),
  flags_to_mask(2 + 4),
  flags_to_mask(1 + 4),
];
pub const BLACK_DROP_MASKS: [(u8, u8); 8] = [
  flags_to_drop_mask(1 + 4),
  flags_to_drop_mask(2 + 4),
  flags_to_drop_mask(1 + 4),
  flags_to_drop_mask(2),
  flags_to_drop_mask(2),
  flags_to_drop_mask(1),
  flags_to_drop_mask(2),
  flags_to_drop_mask(1),
];
pub const WHITE_DROP_MASKS: [(u8, u8); 8] = [
  flags_to_drop_mask(1),
  flags_to_drop_mask(2),
  flags_to_drop_mask(1),
  flags_to_drop_mask(2),
  flags_to_drop_mask(2),
  flags_to_drop_mask(1 + 4),
  flags_to_drop_mask(2 + 4),
  flags_to_drop_mask(1 + 4),
];

fn delta(from: usize, to: usize) -> Direction {
  let (row1, col1) = unpack(from);
  let (row2, col2) = unpack(to);
  (row2 as isize - row1 as isize, col2 as isize - col1 as isize)
}

pub fn delta_direction(from: usize, to: usize) -> Direction {
  let (delta_row, delta_col) = delta(from, to);
  (delta_row.signum(), delta_col.signum())
}

pub fn to_usize(d: Direction) -> usize {
  let k = 3 * d.0 + d.1 + 4;
  (if k > 4 { k - 1 } else { k }) as _
}

pub fn delta_direction_no(from: usize, to: usize) -> usize {
  to_usize(delta_direction(from, to))
}

pub fn try_to_find_delta_direction_no(from: usize, to: usize) -> Option<usize> {
  let d = delta(from, to);
  if d.0 == 0 || d.1 == 0 || d.0.abs() == d.1.abs() {
    Some(to_usize((d.0.signum(), d.1.signum())))
  } else {
    None
  }
}

#[test]
fn test_direction_to_usize() {
  for (i, p) in KING_MOVES.iter().enumerate() {
    assert_eq!(to_usize(p.clone()), i);
  }
}

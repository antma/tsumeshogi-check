pub fn promotion_zone(cell: usize, side: i8) -> bool {
  if side > 0 {
    cell < 27
  } else {
    debug_assert!(side < 0);
    cell >= 54
  }
}

pub fn unpack(cell: usize) -> (usize, usize) {
  (cell / 9, cell % 9)
}

pub fn push_cell_as_en_str(s: &mut String, cell: usize, numeric: bool) {
  let (row, col) = super::cell::unpack(cell);
  s.push((49 + col as u8) as char);
  s.push(((if numeric { 49 } else { 97 }) + row as u8) as char);
}

pub fn to_string(cell: usize) -> String {
  let mut s = String::with_capacity(2);
  push_cell_as_en_str(&mut s, cell, false);
  s
}

pub fn between(cell1: usize, cell2: usize) -> u128 {
  use super::consts::SLIDING_MASKS;
  let k = super::direction::delta_direction_no(cell2, cell1);
  SLIDING_MASKS[8 * cell1 + k] ^ SLIDING_MASKS[8 * cell2 + k] ^ (1u128 << cell1)
}

pub fn mirror(cell: usize) -> usize {
  let (row, col) = unpack(cell);
  9 * (8 - row) + (8 - col)
}

pub(super) fn pawn_could_attack_king_by_move_with_promotion(
  pawn: usize,
  side: i8,
  king_row: usize,
  king_col: usize,
) -> bool {
  if !promotion_zone(pawn, side) {
    return false;
  }
  let (row, col) = unpack(pawn);
  if (king_col as isize - col as isize).abs() > 1 {
    return false;
  }
  let delta_row = row as isize - king_row as isize;
  delta_row == 0 || delta_row == (side as isize)
}

#[cfg(test)]
mod test {
  #[test]
  fn test_between() {
    assert_eq!(
      super::between(3, 9 * 3 + 0),
      (1u128 << (9 * 2 + 1)) + (1u128 << (9 * 1 + 2))
    );
  }
}

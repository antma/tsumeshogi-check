pub type Direction = (isize, isize);

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

fn delta(from: usize, to: usize) -> Direction {
  let (row1, col1) = unpack(from);
  let (row2, col2) = unpack(to);
  (row2 as isize - row1 as isize, col2 as isize - col1 as isize)
}
pub fn delta_direction(from: usize, to: usize) -> Direction {
  let (delta_row, delta_col) = delta(from, to);
  (delta_row.signum(), delta_col.signum())
}
pub fn between(cell1: usize, cell2: usize) -> u128 {
  use super::consts::SLIDING_MASKS;
  let d = delta_direction(cell2, cell1);
  let k = super::bitboards::dir_to_usize(&d);
  SLIDING_MASKS[8 * cell1 + k] ^ SLIDING_MASKS[8 * cell2 + k] ^ (1u128 << cell1)
}

pub fn mirror(cell: usize) -> usize {
  let (row, col) = unpack(cell);
  9 * (8 - row) + (8 - col)
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

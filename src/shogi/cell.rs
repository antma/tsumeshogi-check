pub type Direction = (isize, isize);

pub fn promotion_zone(cell: usize, side: i8) -> bool {
  if side == 1 {
    cell < 27
  } else {
    assert_eq!(side, -1);
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

fn delta(cell1: usize, cell2: usize) -> Direction {
  let (row1, col1) = unpack(cell1);
  let (row2, col2) = unpack(cell2);
  (row1 as isize - row2 as isize, col1 as isize - col2 as isize)
}
pub fn delta_direction(cell1: usize, cell2: usize) -> Direction {
  let (delta_row, delta_col) = delta(cell1, cell2);
  (delta_row.signum(), delta_col.signum())
}
pub fn between(cell1: usize, cell2: usize) -> u128 {
  let mut r = 0u128;
  let (delta_row, delta_col) = delta_direction(cell1, cell2);
  let delta = 9 * delta_row as isize + delta_col;
  let mut k = cell1;
  loop {
    k = ((k as isize) - delta) as usize;
    if k == cell2 {
      break;
    }
    r |= 1u128 << k;
  }
  r
}

#[test]
fn test_between() {
  assert_eq!(
    between(3, 9 * 3 + 0),
    (1u128 << (9 * 2 + 1)) + (1u128 << (9 * 1 + 2))
  );
}

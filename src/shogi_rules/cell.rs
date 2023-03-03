pub type Direction = (isize, isize);

pub fn to_string(row: usize, col: usize) -> String {
  format!("{}{}", col + 1, row + 1)
}
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
fn delta(cell1: usize, cell2: usize) -> Direction {
  let (row1, col1) = unpack(cell1);
  let (row2, col2) = unpack(cell2);
  (row1 as isize - row2 as isize, col1 as isize - col2 as isize)
}
pub fn delta_direction(cell1: usize, cell2: usize) -> Direction {
  let (delta_row, delta_col) = delta(cell1, cell2);
  (delta_row.signum(), delta_col.signum())
}

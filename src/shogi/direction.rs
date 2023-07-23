use super::cell::unpack;
pub type Direction = (isize, isize);

fn delta(from: usize, to: usize) -> Direction {
  let (row1, col1) = unpack(from);
  let (row2, col2) = unpack(to);
  (row2 as isize - row1 as isize, col2 as isize - col1 as isize)
}

pub fn delta_direction(from: usize, to: usize) -> Direction {
  let (delta_row, delta_col) = delta(from, to);
  (delta_row.signum(), delta_col.signum())
}

pub fn to_usize(d: &Direction) -> usize {
  let k = 3 * d.0 + d.1 + 4;
  (if k > 4 { k - 1 } else { k }) as _
}

#[test]
fn test_direction_to_usize() {
  for (i, p) in super::piece::BLACK_DIRECTIONS.iter().enumerate() {
    let d = (p.0, p.1);
    assert_eq!(to_usize(&d), i);
  }
}

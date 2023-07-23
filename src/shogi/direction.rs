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
  for (i, p) in super::piece::BLACK_DIRECTIONS.iter().enumerate() {
    assert_eq!(to_usize((p.0, p.1)), i);
  }
}

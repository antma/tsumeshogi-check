use super::cell::Direction;

pub fn dir_to_usize(d: &Direction) -> usize {
  let k = 3 * d.0 + d.1 + 4;
  (if k > 4 { k - 1 } else { k }) as _
}

pub fn first(b: u128) -> usize {
  b.trailing_zeros() as _
}

pub fn last(b: u128) -> usize {
  127 - b.leading_zeros() as usize
}

#[test]
fn test_dir_to_usize() {
  for (i, p) in super::piece::BLACK_DIRECTIONS.iter().enumerate() {
    assert_eq!(dir_to_usize(&(p.0, p.1)), i);
  }
}

#[test]
fn test_first_last() {
  assert_eq!(first(1), 0);
  assert_eq!(first(2), 1);
  assert_eq!(last(3), 1);
}

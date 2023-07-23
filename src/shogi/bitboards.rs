pub fn first(b: u128) -> usize {
  b.trailing_zeros() as _
}

pub fn last(b: u128) -> usize {
  127 - b.leading_zeros() as usize
}

#[test]
fn test_first_last() {
  assert_eq!(first(1), 0);
  assert_eq!(first(2), 1);
  assert_eq!(last(3), 1);
}

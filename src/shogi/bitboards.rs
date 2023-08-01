pub fn first(b: u128) -> usize {
  b.trailing_zeros() as _
}

fn last(b: u128) -> usize {
  127 - b.leading_zeros() as usize
}

pub fn scan(b: u128, direction_no: usize) -> usize {
  if direction_no < 4 {
    last(b)
  } else {
    first(b)
  }
}

#[test]
fn test_first_last() {
  assert_eq!(first(1), 0);
  assert_eq!(first(2), 1);
  assert_eq!(last(3), 1);
}

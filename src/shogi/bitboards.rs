use super::cell::Direction;

pub fn dir_to_usize(d: &Direction) -> usize {
  let k = 3 * d.0 + d.1 + 4;
  (if k > 4 { k - 1 } else { k }) as _
}

/*
const fn build() -> [u128; 81 * 8] {
  let mut a = [0; 81 * 8];
  for (i, p) in super::piece::BLACK_DIRECTIONS.iter().enumerate() {
    let d: Direction = (p.0, p.1);
  }
  a
}
*/

#[test]
fn test_dir_to_usize() {
  for (i, p) in super::piece::BLACK_DIRECTIONS.iter().enumerate() {
    assert_eq!(dir_to_usize(&(p.0, p.1)), i);
  }
}

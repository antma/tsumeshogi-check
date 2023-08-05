use super::{cell, consts};

pub const BLACK_PROMOTION: u128 = (1u128 << 27) - 1;
pub const WHITE_PROMOTION: u128 = BLACK_PROMOTION << 54;

pub fn promotion_zone(side: i8) -> u128 {
  if side > 0 {
    BLACK_PROMOTION
  } else {
    WHITE_PROMOTION
  }
}

#[derive(Clone)]
pub struct Bits128(pub u128);
impl Iterator for Bits128 {
  type Item = usize;
  fn next(&mut self) -> Option<Self::Item> {
    if self.0 == 0 {
      None
    } else {
      let i = self.0.trailing_zeros() as usize;
      self.0 ^= 1 << i;
      Some(i)
    }
  }
}

impl std::fmt::Display for Bits128 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{{")?;
    for (i, k) in self.clone().enumerate() {
      if i > 0 {
        write!(f, ", ")?
      }
      write!(f, "{}", cell::to_string(k))?
    }
    write!(f, "}}")
  }
}

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

pub fn rook(pos: usize, b1: u128, b2: u128) -> u128 {
  let (row, col) = cell::unpack(pos);
  let s = 9 * row;
  let i = ((b1 >> (s + 1)) & 127) as usize;
  let j = ((b2 >> (9 * col + 1)) & 127) as usize;
  ((consts::ROOK_HORIZONTAL_MASKS[128 * col + i] as u128) << s)
    | (consts::ROOK_VERTICAL_MASKS[128 * row + j] << col)
}

pub fn bishop(pos: usize, b3: u128, b4: u128) -> u128 {
  let u = &consts::BISHOP3[pos];
  let i = ((b3 >> u.shift) & (u.mask as u128)) as usize;
  let v = &consts::BISHOP4[pos];
  let j = ((b4 >> v.shift) & (v.mask as u128)) as usize;
  consts::DATA3[u.offset + i] | consts::DATA4[v.offset + j]
}

#[test]
fn test_first_last() {
  assert_eq!(first(1), 0);
  assert_eq!(first(2), 1);
  assert_eq!(last(3), 1);
}

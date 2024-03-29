use super::{cell, consts, piece};
use std::iter::{Chain, Map};

pub const ALL_BITS: u128 = (1u128 << 81) - 1;
const BLACK_PROMOTION_ZONE_MASK: u128 = (1u128 << 27) - 1;
const WHITE_PROMOTION_ZONE_MASK: u128 = BLACK_PROMOTION_ZONE_MASK << 54;
const BLACK_UNPROMOTED_PAWN: u128 = (ALL_BITS << 9) & ALL_BITS;
const WHITE_UNPROMOTED_PAWN: u128 = ALL_BITS >> 9;
pub const BLACK_UNPROMOTED_KNIGHT: u128 = (BLACK_UNPROMOTED_PAWN << 9) & ALL_BITS;
pub const WHITE_UNPROMOTED_KNIGHT: u128 = WHITE_UNPROMOTED_PAWN >> 9;

pub fn promotion_zone(side: i8) -> u128 {
  if side > 0 {
    BLACK_PROMOTION_ZONE_MASK
  } else {
    WHITE_PROMOTION_ZONE_MASK
  }
}

pub fn unpromoted_zone(piece: i8) -> u128 {
  match piece {
    //unpromoted pawn and lance could be on last rank
    piece::PAWN | piece::LANCE => BLACK_UNPROMOTED_PAWN,
    piece::WHITE_PAWN | piece::WHITE_LANCE => WHITE_UNPROMOTED_PAWN,
    piece::KNIGHT => BLACK_UNPROMOTED_KNIGHT,
    piece::WHITE_KNIGHT => WHITE_UNPROMOTED_KNIGHT,
    _ => ALL_BITS,
  }
}

pub struct Bits32(pub u32);
impl Iterator for Bits32 {
  type Item = usize;
  fn next(&mut self) -> Option<Self::Item> {
    if self.0 == 0 {
      None
    } else {
      let i = self.0.trailing_zeros() as usize;
      self.0 &= self.0 - 1;
      Some(i)
    }
  }
}

pub struct Bits64(pub u64);
impl Iterator for Bits64 {
  type Item = usize;
  fn next(&mut self) -> Option<Self::Item> {
    if self.0 == 0 {
      None
    } else {
      let i = self.0.trailing_zeros() as usize;
      self.0 &= self.0 - 1;
      Some(i)
    }
  }
}

#[derive(Clone)]
pub struct Bits128(pub u128);
impl std::iter::IntoIterator for Bits128 {
  type Item = usize;
  type IntoIter = Chain<Bits64, Map<Bits64, fn(usize) -> usize>>;
  fn into_iter(self) -> Self::IntoIter {
    fn add64(x: usize) -> usize {
      x + 64
    }
    Bits64((self.0 & 0xffff_ffff_ffff_ffff) as u64)
      .chain(Bits64((self.0 >> 64) as u64).map(add64 as _))
  }
}
impl std::fmt::Display for Bits128 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{{")?;
    for (i, k) in self.clone().into_iter().enumerate() {
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

pub fn contains(b: u128, k: usize) -> bool {
  (b & (1 << k)) != 0
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

pub fn lance(pos: usize, v: i8, b2: u128) -> u128 {
  let (row, col) = cell::unpack(pos);
  let j = ((b2 >> (9 * col + 1)) & 127) as usize;
  (consts::ROOK_VERTICAL_MASKS[128 * row + j] << col)
    & consts::SLIDING_MASKS[8 * pos + (if v > 0 { 1 } else { 6 })]
}

#[test]
fn test_first_last() {
  assert_eq!(first(1), 0);
  assert_eq!(first(2), 1);
  assert_eq!(last(3), 1);
}

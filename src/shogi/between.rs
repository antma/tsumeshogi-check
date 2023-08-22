use super::{consts, direction};
use std::num::NonZeroU128;

pub(super) struct SlidingIterator {
  delta: isize,
  last: usize,
  end: usize,
  drops_mask: u8,
}

impl Iterator for SlidingIterator {
  type Item = (usize, u8);
  fn next(&mut self) -> Option<Self::Item> {
    let a = (self.last as isize + self.delta) as usize;
    if a == self.end {
      None
    } else {
      self.last = a;
      Some((a, self.drops_mask))
    }
  }
}

impl SlidingIterator {
  pub(super) fn new(attacking_piece: usize, king_pos: usize, drops_mask: u8) -> Self {
    let (delta_row, delta_col) = direction::delta_direction(king_pos, attacking_piece);
    SlidingIterator {
      delta: 9 * delta_row + delta_col,
      last: king_pos,
      end: attacking_piece,
      drops_mask,
    }
  }
}

pub struct Between([Option<NonZeroU128>; 2 * 81 * 81]);
impl Default for Between {
  fn default() -> Self {
    Self([None; 2 * 81 * 81])
  }
}

impl Between {
  pub fn f(&mut self, from: usize, to: usize, side: i8) -> u128 {
    let k = from * 81 + to;
    let k = if side > 0 { k + 81 * 81 } else { k };
    if let Some(p) = &self.0[k] {
      p.get()
    } else {
      let mut w = consts::KING_MASKS[from]
        | (if side > 0 {
          consts::WHITE_KNIGHT_MASKS[from]
        } else {
          consts::BLACK_KNIGHT_MASKS[from]
        });
      if let Some(i) = direction::try_to_find_delta_direction_no(from, to) {
        let cell = (from as isize + direction::OFFSETS[i]) as usize;
        if cell != to {
          w |= self.f(cell, to, side);
        }
      }
      self.0[k] = NonZeroU128::new(w);
      w
    }
  }
}

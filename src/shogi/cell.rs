pub type Direction = (isize, isize);

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

pub fn push_cell_as_en_str(s: &mut String, cell: usize, numeric: bool) {
  let (row, col) = super::cell::unpack(cell);
  s.push((49 + col as u8) as char);
  s.push(((if numeric { 49 } else { 97 }) + row as u8) as char);
}

pub fn to_string(cell: usize) -> String {
  let mut s = String::with_capacity(2);
  push_cell_as_en_str(&mut s, cell, false);
  s
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

pub struct SlidingPieceMovesIterator {
  k: usize,
  delta: isize,
  r: usize,
}

fn distance_to_edge(x: usize, delta: isize) -> usize {
  if delta > 0 {
    8 - x
  } else if delta < 0 {
    x
  } else {
    8
  }
}

impl SlidingPieceMovesIterator {
  pub fn new(cell: usize, delta_row: isize, delta_col: isize) -> Self {
    let (row, col) = unpack(cell);
    Self {
      k: cell,
      delta: 9 * delta_row + delta_col,
      r: distance_to_edge(row, delta_row).min(distance_to_edge(col, delta_col)),
    }
  }
}

impl Iterator for SlidingPieceMovesIterator {
  type Item = usize;
  fn next(&mut self) -> Option<Self::Item> {
    if self.r == 0 {
      None
    } else {
      self.r -= 1;
      self.k = ((self.k as isize) + self.delta) as usize;
      Some(self.k)
    }
  }
}

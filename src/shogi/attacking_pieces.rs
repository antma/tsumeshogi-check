#[derive(Debug, PartialEq)]

pub enum AttackingPieces {
  TinyArray([usize; 2], usize),
  Bitset(u128),
}

impl Iterator for AttackingPieces {
  type Item = usize;
  fn next(&mut self) -> Option<Self::Item> {
    match self {
      AttackingPieces::TinyArray(a, n) => {
        if *n == 0 {
          None
        } else {
          *n -= 1;
          Some(a[*n])
        }
      }
      AttackingPieces::Bitset(b) => {
        if *b == 0 {
          None
        } else {
          let k = super::bitboards::first(*b);
          *b ^= 1u128 << k;
          Some(k)
        }
      }
    }
  }
}

impl Default for AttackingPieces {
  fn default() -> Self {
    AttackingPieces::TinyArray([usize::MAX; 2], 0)
  }
}

impl From<&[usize]> for AttackingPieces {
  fn from(v: &[usize]) -> Self {
    let n = v.len();
    if n <= 2 {
      let mut a = [usize::MAX; 2];
      for (k, q) in v.iter().enumerate() {
        a[k] = *q;
      }
      AttackingPieces::TinyArray(a, n)
    } else {
      AttackingPieces::Bitset(v.iter().fold(0, |acc, x| acc | (1u128 << (*x))))
    }
  }
}

impl AttackingPieces {
  pub fn len(&self) -> usize {
    match self {
      AttackingPieces::TinyArray(_, n) => *n,
      AttackingPieces::Bitset(s) => s.count_ones() as usize,
    }
  }
  pub fn once(x: usize) -> Self {
    AttackingPieces::TinyArray([x, usize::MAX], 1)
  }
  pub fn push(&mut self, x: usize) {
    match self {
      AttackingPieces::TinyArray(a, n) => {
        if *n < 2 {
          a[*n] = x;
          *n += 1;
        } else {
          *self =
            AttackingPieces::Bitset(a.iter().fold(1u128 << x, |acc, x| acc | (1u128 << (*x))));
        }
      }
      AttackingPieces::Bitset(s) => {
        *s |= 1u128 << x;
      }
    }
  }
  pub fn contains(&self, x: &usize) -> bool {
    match self {
      AttackingPieces::TinyArray(a, n) => a.iter().take(*n).any(|y| *y == *x),
      AttackingPieces::Bitset(s) => (s & (1u128 << x)) != 0,
    }
  }
  pub fn is_empty(&self) -> bool {
    match self {
      AttackingPieces::TinyArray(_, n) => *n == 0,
      _ => false,
    }
  }
  pub fn first(&self) -> Option<usize> {
    match self {
      AttackingPieces::TinyArray(a, n) => {
        if *n == 0 {
          None
        } else {
          a.first().cloned()
        }
      }
      AttackingPieces::Bitset(s) => Some(s.trailing_zeros() as usize),
    }
  }
}

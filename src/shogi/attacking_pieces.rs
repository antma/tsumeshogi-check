use std::collections::BTreeSet;

#[derive(Debug, PartialEq)]

pub enum AttackingPieces {
  TinyArray([usize; 2], usize),
  TreeSet(BTreeSet<usize>),
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
      AttackingPieces::TreeSet(v.iter().cloned().collect())
    }
  }
}

impl AttackingPieces {
  pub fn len(&self) -> usize {
    match self {
      AttackingPieces::TinyArray(_, n) => *n,
      AttackingPieces::TreeSet(s) => s.len(),
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
          let mut s: BTreeSet<usize> = a.iter().cloned().collect();
          s.insert(x);
          *self = AttackingPieces::TreeSet(s);
        }
      }
      AttackingPieces::TreeSet(s) => {
        s.insert(x);
      }
    }
  }
  pub fn contains(&self, x: &usize) -> bool {
    match self {
      AttackingPieces::TinyArray(a, n) => a.iter().take(*n).any(|y| *y == *x),
      AttackingPieces::TreeSet(s) => s.contains(x),
    }
  }
  pub fn is_empty(&self) -> bool {
    match self {
      AttackingPieces::TinyArray(_, n) => *n == 0,
      _ => false,
    }
  }
  pub fn first(&self) -> Option<&usize> {
    match self {
      AttackingPieces::TinyArray(a, n) => {
        if *n == 0 {
          None
        } else {
          a.first()
        }
      }
      AttackingPieces::TreeSet(s) => s.first(),
    }
  }
}

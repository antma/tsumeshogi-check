use crate::shogi::moves::Move;
use crate::shogi::Position;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub enum BestMove {
  None,
  One(u32),
  Many,
}

impl BestMove {
  pub fn is_none(&self) -> bool {
    match *self {
      BestMove::None => true,
      _ => false,
    }
  }
  pub fn is_one(&self) -> bool {
    match *self {
      BestMove::One(_) => true,
      _ => false,
    }
  }
  pub fn is_many(&self) -> bool {
    match *self {
      BestMove::Many => true,
      _ => false,
    }
  }
  pub fn is_some(&self) -> bool {
    match *self {
      BestMove::None => false,
      _ => true,
    }
  }
  fn update(&mut self, m: &Move, bm: BestMove) {
    let x = match self {
      BestMove::None => match bm {
        BestMove::None => panic!(""),
        BestMove::One(_) => BestMove::One(u32::from(m)),
        BestMove::Many => BestMove::Many,
      },
      BestMove::One(_) => BestMove::Many,
      BestMove::Many => return,
    };
    *self = x;
  }
  fn get_move(&self) -> Option<Move> {
    match self {
      BestMove::One(v) => Some(Move::from(*v)),
      _ => None,
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult {
  pub best_move: BestMove,
  pub nodes: u64,
  pub depth: u8,
}

impl SearchResult {
  pub fn new(depth: u8) -> Self {
    Self {
      best_move: BestMove::None,
      nodes: 0,
      depth,
    }
  }
  pub fn update_best_move(&mut self, m: &Move, ev: SearchResult) {
    self.best_move.update(m, ev.best_move);
  }
  pub fn get_move(&self) -> Option<Move> {
    if self.depth == 0 {
      None
    } else {
      self.best_move.get_move()
    }
  }
  pub fn gote_cmp(&self, other: &SearchResult, pos: &Position) -> Ordering {
    if self.best_move.is_none() {
      //update
      return Ordering::Less;
    }
    debug_assert!(other.best_move.is_some());
    let c = self.depth.cmp(&other.depth);
    if c != Ordering::Equal {
      //update if self.depth < other.depth
      return c;
    }
    let o1 = self.best_move.is_one();
    let o2 = other.best_move.is_one();
    let c = o1.cmp(&o2);
    if c != Ordering::Equal {
      //update if other.pv.is_one() and self.pv.is_many()
      return c;
    }
    if !o1 {
      return Ordering::Equal;
    }
    let m1 = self.get_move().unwrap();
    let m2 = other.get_move().unwrap();
    let t1 = pos.is_take(&m1);
    let t2 = pos.is_take(&m2);
    let c = t1.cmp(&t2);
    if c != Ordering::Equal {
      return c;
    }
    let t1 = !m1.is_drop();
    let t2 = !m2.is_drop();
    let c = t1.cmp(&t2);
    if c != Ordering::Equal {
      return c;
    }
    let c = self.nodes.cmp(&other.nodes);
    if c != Ordering::Equal {
      //update if self.nodes < other.nodes
      return c;
    }
    Ordering::Equal
  }
}

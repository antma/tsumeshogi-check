mod hash;
pub mod it;

use super::shogi;
use shogi::moves::{Move, Moves};
use shogi::{Checks, Position};
use std::cmp::Ordering;

#[derive(Clone)]
pub enum BestMove {
  None,
  One(Move),
  Many,
}

impl BestMove {
  fn is_none(&self) -> bool {
    match *self {
      BestMove::None => true,
      _ => false,
    }
  }
  fn is_one(&self) -> bool {
    match *self {
      BestMove::One(_) => true,
      _ => false,
    }
  }
  fn is_many(&self) -> bool {
    match *self {
      BestMove::Many => true,
      _ => false,
    }
  }
  fn is_some(&self) -> bool {
    match *self {
      BestMove::None => false,
      _ => true,
    }
  }
  fn update_best_move(&mut self, m: Move, bm: BestMove) {
    let x = {
      if self.is_some() {
        BestMove::Many
      } else {
        match bm {
          BestMove::None => panic!(""),
          BestMove::One(_) => BestMove::One(m),
          BestMove::Many => BestMove::Many,
        }
      }
    };
    *self = x;
  }
  fn get_move(&self) -> Option<&Move> {
    match self {
      BestMove::One(ref v) => Some(v),
      _ => None,
    }
  }
}

#[derive(Clone)]
struct SearchResult {
  depth: u8,
  best_move: BestMove,
  nodes: u64,
}

impl SearchResult {
  fn new(depth: u8) -> Self {
    Self {
      depth,
      best_move: BestMove::None,
      nodes: 0,
    }
  }
  fn gote_cmp(&self, other: &SearchResult, pos: &Position) -> Ordering {
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
    let m1 = self.best_move.get_move().unwrap();
    let m2 = other.best_move.get_move().unwrap();
    let t1 = pos.is_take(m1);
    let t2 = pos.is_take(m2);
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

type Hash = hash::SearchHash<SearchResult>;

pub struct Search {
  sente_hash: Hash,
  gote_hash: Hash,
  pub nodes: u64,
  generation: u8,
}

impl Default for Search {
  fn default() -> Self {
    Self {
      sente_hash: Hash::default(),
      gote_hash: Hash::default(),
      nodes: 0,
      generation: 0,
    }
  }
}

impl Search {
  pub fn hashes_clear(&mut self) {
    self.sente_hash.clear();
    self.gote_hash.clear();
  }
  pub fn hashes_retain(&mut self, margin: u8) {
    self.sente_hash.retain(self.generation, margin);
    self.gote_hash.retain(self.generation, margin);
  }
  fn increment_generation(&mut self) {
    self.generation = self.generation.wrapping_add(1);
  }
  fn nodes_increment(&mut self) -> u64 {
    let r = self.nodes;
    self.nodes += 1;
    r
  }
  fn gote_search(
    &mut self,
    pos: &mut Position,
    ochecks: Option<Checks>,
    depth: u8,
  ) -> SearchResult {
    debug_assert_eq!(depth % 2, 0);
    if let Some(q) = self.gote_hash.get(pos.hash) {
      if q.best_move.is_some() || q.depth >= depth {
        return q.clone();
      }
    }
    let nodes = self.nodes_increment();
    let hash_best_move = None;
    let sente = false;
    let allow_futile_drops = false;
    let mut it = it::MovesIterator::new(pos, ochecks, hash_best_move, sente, allow_futile_drops);
    let mut res = SearchResult::new(0);
    if depth == 0 {
      let mut mate = true;
      while let Some((m, u, _)) = it.do_next_move(pos) {
        pos.undo_move(&m, &u);
        mate = false;
        break;
      }
      if mate {
        res.best_move = BestMove::One(Move::default());
      }
    } else {
      let next_depth = depth - 1;
      while let Some((m, u, oc)) = it.do_next_move(pos) {
        let mut ev = self.sente_search(pos, oc, next_depth);
        debug_assert_eq!(ev.depth % 2, 1);
        pos.undo_move(&m, &u);
        if ev.best_move.is_none() {
          res.depth = depth;
          res.best_move = BestMove::None;
          break;
        }
        ev.depth += 1;
        if res.gote_cmp(&ev, pos) == Ordering::Less {
          res.depth = ev.depth;
          res.best_move.update_best_move(m, ev.best_move);
        }
      }
      if it.legal_moves == 0 {
        res.depth = 0;
        res.best_move = BestMove::One(Move::default());
      }
    }
    res.nodes = self.nodes - nodes;
    self.gote_hash.set(pos.hash, res.clone(), self.generation);
    res
  }
  fn sente_search(
    &mut self,
    pos: &mut Position,
    ochecks: Option<Checks>,
    depth: u8,
  ) -> SearchResult {
    debug_assert_eq!(depth % 2, 1);
    if let Some(q) = self.sente_hash.get(pos.hash) {
      if q.best_move.is_some() || q.depth >= depth {
        return q.clone();
      }
    }
    let nodes = self.nodes_increment();
    let hash_best_move = None;
    let sente = true;
    let allow_futile_drops = true;
    let mut it = it::MovesIterator::new(pos, ochecks, hash_best_move, sente, allow_futile_drops);
    let mut res = SearchResult::new(depth);
    while let Some((m, u, oc)) = it.do_next_move(pos) {
      let next_depth = if res.best_move.is_many() {
        res.depth - 3
      } else {
        res.depth - 1
      };
      let ev = self.gote_search(pos, oc, next_depth);
      pos.undo_move(&m, &u);
      if !ev.best_move.is_some() {
        //not mated
        continue;
      }
      if ev.depth == 0 && m.is_pawn_drop() {
        //mate by pawn drop (illegal)
        continue;
      }
      debug_assert!(ev.depth <= next_depth);
      let mate_in = ev.depth + 1;
      if res.depth > mate_in {
        res.depth = mate_in;
        res.best_move = BestMove::None;
      } else {
        debug_assert_eq!(res.depth, mate_in);
      }
      res.best_move.update_best_move(m, ev.best_move);
      if res.depth == 1 && res.best_move.is_many() {
        break;
      }
    }
    res.nodes = self.nodes - nodes;
    self.sente_hash.set(pos.hash, res.clone(), self.generation);
    res
  }
  fn extract_pv_from_hash(&self, pos: &mut Position, depth: usize) -> Vec<Move> {
    let mut r = Moves::with_capacity(depth);
    loop {
      let o = if pos.side > 0 {
        self.sente_hash.get(pos.hash)
      } else {
        self.gote_hash.get(pos.hash)
      };
      if let Some(p) = o {
        if p.depth > 0 && p.best_move.is_one() {
          let m = p.best_move.get_move().unwrap();
          r.push(pos, m.clone());
          continue;
        }
      }
      break;
    }
    r.undo(pos);
    r.only_moves()
  }
  pub fn search(&mut self, pos: &mut Position, max_depth: u8) -> (Option<u8>, Option<Vec<Move>>) {
    log::debug!("search(pos: {}, max_depth: {})", pos, max_depth);
    assert!(pos.side > 0);
    self.increment_generation();
    let hash = pos.hash;
    for depth in (1..=max_depth).step_by(2) {
      let ev = self.sente_search(pos, None, depth);
      assert_eq!(hash, pos.hash);
      if ev.best_move.is_some() {
        if ev.best_move.is_one() {
          let pv = self.extract_pv_from_hash(pos, depth as usize);
          assert_eq!(hash, pos.hash);
          return (
            Some(ev.depth),
            Some(pv),
          );
        } else {
          return (Some(ev.depth), None);
        }
      }
    }
    (None, None)
  }
}

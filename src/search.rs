mod hash;
pub mod it;

use super::shogi;
use shogi::moves::Move;
use shogi::{Checks, Position};
use std::cmp::Ordering;

#[derive(Clone)]
pub enum PV {
  None,
  One(Vec<Move>),
  Many,
}

impl PV {
  fn is_none(&self) -> bool {
    match *self {
      PV::None => true,
      _ => false,
    }
  }
  fn is_one(&self) -> bool {
    match *self {
      PV::One(_) => true,
      _ => false,
    }
  }
  fn is_many(&self) -> bool {
    match *self {
      PV::Many => true,
      _ => false,
    }
  }
  fn is_some(&self) -> bool {
    match *self {
      PV::None => false,
      _ => true,
    }
  }
  fn update_pv(&mut self, m: Move, pv: PV) {
    if self.is_some() {
      *self = PV::Many;
    } else {
      match pv {
        PV::None => panic!(""),
        PV::One(mut v) => {
          v.push(m);
          *self = PV::One(v);
        }
        PV::Many => *self = PV::Many,
      }
    }
  }
  fn last(&self) -> Option<&Move> {
    match self {
      PV::One(v) => v.last(),
      _ => None,
    }
  }
}

#[derive(Clone)]
pub struct SearchResult {
  pub depth: u8,
  pub pv: PV,
  nodes: u64,
}

impl SearchResult {
  fn reverse(&mut self) {
    if let PV::One(v) = &mut self.pv {
      v.reverse();
    }
  }
  fn new(depth: u8) -> Self {
    Self {
      depth,
      pv: PV::None,
      nodes: 0,
    }
  }
  #[allow(dead_code)]
  fn is_mated(&self) -> bool {
    self.depth == 0 && self.pv.is_some()
  }
  fn gote_cmp(&self, other: &SearchResult, pos: &Position) -> Ordering {
    if self.pv.is_none() {
      //update
      return Ordering::Less;
    }
    debug_assert!(other.pv.is_some());
    let c = self.depth.cmp(&other.depth);
    if c != Ordering::Equal {
      //update if self.depth < other.depth
      return c;
    }
    let o1 = self.pv.is_one();
    let o2 = other.pv.is_one();
    let c = o1.cmp(&o2);
    if c != Ordering::Equal {
      //update if other.pv.is_one() and self.pv.is_many()
      return c;
    }
    if !o1 {
      return Ordering::Equal;
    }
    let m1 = self.pv.last().unwrap();
    let m2 = other.pv.last().unwrap();
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
      if q.pv.is_some() || q.depth >= depth {
        return q.clone();
      }
    }
    let nodes = self.nodes_increment();
    let hash_best_move = None;
    let sente = false;
    let allow_futile_drops = true;
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
        res.pv = PV::One(Vec::new());
      }
    } else {
      while let Some((m, u, oc)) = it.do_next_move(pos) {
        let next_depth = res.depth - 1;
        let ev = self.sente_search(pos, oc, next_depth);
        pos.undo_move(&m, &u);
        if ev.pv.is_none() {
          res.depth = depth;
          res.pv = PV::None;
          break;
        }
        if res.gote_cmp(&ev, pos) == Ordering::Less {
          res.depth = ev.depth;
          res.pv.update_pv(m, ev.pv);
        }
      }
      if it.legal_moves == 0 {
        res.depth = 0;
        res.pv = PV::One(Vec::new());
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
      if q.pv.is_some() || q.depth >= depth {
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
      let next_depth = if res.pv.is_many() {
        res.depth - 3
      } else {
        res.depth - 1
      };
      let ev = self.gote_search(pos, oc, next_depth);
      pos.undo_move(&m, &u);
      if !ev.pv.is_some() {
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
        res.pv = PV::None;
      } else {
        debug_assert_eq!(res.depth, mate_in);
      }
      res.pv.update_pv(m, ev.pv);
      if res.depth == 1 && res.pv.is_many() {
        break;
      }
    }
    res.nodes = self.nodes - nodes;
    self.sente_hash.set(pos.hash, res.clone(), self.generation);
    res
  }
  pub fn search(&mut self, pos: &mut Position, max_depth: u8) -> Option<SearchResult> {
    self.increment_generation();
    let hash = pos.hash;
    for depth in (1..=max_depth).step_by(2) {
      let mut ev = self.sente_search(pos, None, depth);
      assert_eq!(hash, pos.hash);
      if ev.pv.is_some() {
        ev.reverse();
        return Some(ev);
      }
    }
    None
  }
}

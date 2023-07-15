mod hash;
pub mod it;

use super::shogi;
use shogi::moves::{Move, Moves};
use shogi::{Checks, Position};
use std::cmp::Ordering;

#[derive(Clone)]
pub enum BestMove {
  None,
  One(u32),
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
  fn update(&mut self, m: Move, bm: BestMove) {
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

#[derive(Clone)]
struct SearchResult {
  best_move: BestMove,
  nodes: u64,
  depth: u8,
}

impl SearchResult {
  fn new(depth: u8) -> Self {
    Self {
      best_move: BestMove::None,
      nodes: 0,
      depth,
    }
  }
  fn update_best_move(&mut self, m: Move, ev: SearchResult) {
    self.best_move.update(m, ev.best_move);
  }
  fn get_move(&self) -> Option<Move> {
    if self.depth == 0 {
      None
    } else {
      self.best_move.get_move()
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

type SenteHash = hash::SearchHash<SearchResult>;
type GoteHash = hash::SearchHash<(SearchResult, Option<Move>)>;

pub struct Search {
  sente_hash: SenteHash,
  gote_hash: GoteHash,
  pub nodes: u64,
  generation: u8,
}

impl Default for Search {
  fn default() -> Self {
    log::debug!("sizeof(SearchResult)={}",std::mem::size_of::<SearchResult>());
    Self {
      sente_hash: SenteHash::default(),
      gote_hash: GoteHash::default(),
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
    let mut hash_best_move = None;
    if let Some((q, m)) = self.gote_hash.get(pos.hash) {
      if q.best_move.is_some() || q.depth >= depth {
        return q.clone();
      }
      hash_best_move = m.clone();
    }
    let nodes = self.nodes_increment();
    let sente = false;
    let allow_futile_drops = false;
    let mut it = it::MovesIterator::new(pos, ochecks, hash_best_move, sente, allow_futile_drops);
    let mut res = SearchResult::new(0);
    hash_best_move = None;
    if depth == 0 {
      let mut mate = true;
      while let Some((m, u, _)) = it.do_next_move(pos) {
        pos.undo_move(&m, &u);
        mate = false;
        break;
      }
      if mate {
        res.best_move = BestMove::One(0);
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
          hash_best_move = Some(m);
          break;
        }
        ev.depth += 1;
        if res.gote_cmp(&ev, pos) == Ordering::Less {
          res.depth = ev.depth;
          res.update_best_move(m, ev);
        }
      }
      if it.legal_moves == 0 {
        res.depth = 0;
        res.best_move = BestMove::One(0);
      }
    }
    res.nodes = self.nodes - nodes;
    self
      .gote_hash
      .set(pos.hash, (res.clone(), hash_best_move), self.generation);
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
      let mate_in = ev.depth + 1;
      if res.depth > mate_in {
        res.depth = mate_in;
        res.best_move = BestMove::None;
      } else if res.depth < mate_in {
        continue;
      }
      res.update_best_move(m, ev);
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
        self.gote_hash.get(pos.hash).map(|p| &p.0)
      };
      if let Some(p) = o {
        if let Some(m) = p.get_move() {
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
          return (Some(ev.depth), Some(pv));
        } else {
          return (Some(ev.depth), None);
        }
      }
    }
    (None, None)
  }
}

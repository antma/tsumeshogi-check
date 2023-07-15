mod hash;
pub mod it;
mod result;

use super::shogi;
use hash::SearchHash;
use result::{BestMove, SearchResult};
use shogi::moves::{Move, Moves};
use shogi::{Checks, Position};
use std::cmp::Ordering;

pub struct Search {
  sente_hash: SearchHash,
  gote_hash: SearchHash,
  pub nodes: u64,
  generation: u8,
}

impl Default for Search {
  fn default() -> Self {
    //log::debug!("sizeof(SearchResult)={}",std::mem::size_of::<SearchResult>());
    Self {
      sente_hash: SearchHash::default(),
      gote_hash: SearchHash::default(),
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
    if let Some((q, m)) = self.gote_hash.get_gote(pos.hash) {
      if q.best_move.is_some() || q.depth >= depth {
        return q;
      }
      hash_best_move = m;
    }
    let nodes = self.nodes_increment();
    let sente = false;
    let allow_futile_drops = false;
    let mut it = it::MovesIterator::new(pos, ochecks, hash_best_move, sente, allow_futile_drops);
    let mut res = SearchResult::new(0);
    hash_best_move = None;
    if depth == 0 {
      while let Some((m, u, _)) = it.do_next_move(pos) {
        pos.undo_move(&m, &u);
        hash_best_move = Some(m);
        break;
      }
      if hash_best_move.is_none() {
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
      .insert_gote(pos.hash, &res, hash_best_move, self.generation);
    res
  }
  fn sente_search(
    &mut self,
    pos: &mut Position,
    ochecks: Option<Checks>,
    depth: u8,
  ) -> SearchResult {
    debug_assert_eq!(depth % 2, 1);
    if let Some(q) = self.sente_hash.get_sente(pos.hash) {
      if q.best_move.is_some() || q.depth >= depth {
        return q;
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
    self
      .sente_hash
      .insert_sente(pos.hash, &res, self.generation);
    res
  }
  fn extract_pv_from_hash(&self, pos: &mut Position, depth: usize) -> Vec<Move> {
    let mut r = Moves::with_capacity(depth);
    loop {
      let o = if pos.side > 0 {
        self.sente_hash.get_sente(pos.hash)
      } else {
        self.gote_hash.get_gote(pos.hash).map(|p| p.0)
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

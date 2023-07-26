mod hash;
mod history;
mod it;
mod result;

use super::shogi;
use hash::SearchHash;
use history::HistoryTable;
use result::{BestMove, SearchResult};
use shogi::moves::{Move, Moves};
use shogi::{Checks, Position};
use std::cmp::Ordering;

pub struct Search {
  sente_hash: SearchHash,
  gote_hash: SearchHash,
  gote_history_global_tables: Vec<HistoryTable>,
  gote_history_local_tables: Vec<HistoryTable>,
  pub nodes: u64,
  hash_nodes: u64,
  generation: u8,
}

impl Default for Search {
  fn default() -> Self {
    //log::debug!("sizeof(SearchResult)={}",std::mem::size_of::<SearchResult>());
    //log::debug!("sizeof(AttackingPieces)={}", std::mem::size_of::<shogi::attacking_pieces::AttackingPieces>());
    Self {
      sente_hash: SearchHash::default(),
      gote_hash: SearchHash::default(),
      gote_history_global_tables: Vec::new(),
      gote_history_local_tables: Vec::new(),
      nodes: 0,
      hash_nodes: 0,
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
  fn history_resize(&mut self, depth: u8) {
    let d = depth as usize / 2;
    while d >= self.gote_history_global_tables.len() {
      self
        .gote_history_global_tables
        .push(HistoryTable::default());
      self.gote_history_local_tables.push(HistoryTable::default());
    }
    assert_eq!(
      self.gote_history_global_tables.len(),
      self.gote_history_local_tables.len()
    );
  }
  fn update_global_history(&mut self) {
    for i in 0..self.gote_history_global_tables.len() {
      let mut t = history::HistoryTable::default();
      std::mem::swap(&mut t, &mut self.gote_history_local_tables[i]);
      self.gote_history_global_tables[i].merge(t);
    }
  }
  fn nodes_increment(&mut self) -> u64 {
    let r = self.nodes;
    self.nodes += 1;
    r
  }
  fn gote_search(&mut self, pos: &mut Position, checks: Checks, depth: u8) -> SearchResult {
    debug_assert_eq!(depth % 2, 0);
    let mut hash_best_move = None;
    if let Some((q, m)) = self.gote_hash.get_gote(pos.hash) {
      if q.best_move.is_some() || q.depth >= depth {
        self.hash_nodes += q.nodes;
        return q;
      }
      hash_best_move = m;
    }
    let nodes = self.nodes_increment();
    let hash_nodes = self.hash_nodes;
    let allow_futile_drops = false;
    let mut it = it::GoteMovesIterator::new(checks, hash_best_move, allow_futile_drops);
    let mut res = SearchResult::new(0);
    hash_best_move = None;
    let d = depth as usize / 2;
    if depth == 0 {
      while let Some((m, u)) = it.do_next_move(pos, |mv| {
        let x = u32::from(mv);
        self.gote_history_global_tables[d].get(x) * self.gote_history_local_tables[d].get(x)
      }) {
        pos.undo_move(&m, &u);
        hash_best_move = Some(m);
        break;
      }
      if hash_best_move.is_none() {
        res.best_move = BestMove::One(0);
      }
    } else {
      let next_depth = depth - 1;
      while let Some((m, u)) = it.do_next_move(pos, |mv| {
        let x = u32::from(mv);
        self.gote_history_global_tables[d].get(x) * self.gote_history_local_tables[d].get(x)
      }) {
        let mut ev = self.sente_search(pos, next_depth, Some(&m));
        log::debug!(
          "self.sente_search({}, next_depth: {}) = {:?} after move {}.{}",
          pos,
          next_depth,
          ev,
          pos.move_no - 1,
          shogi::moves::PSNMove::new(&m, &u),
        );
        debug_assert_eq!(ev.depth % 2, 1);
        pos.undo_move(&m, &u);
        if ev.best_move.is_none() {
          res.depth = depth;
          res.best_move = BestMove::None;
          if !m.is_drop() {
            self.gote_history_local_tables[d].success(u32::from(&m));
          }
          hash_best_move = Some(m);
          break;
        }
        if !m.is_drop() {
          self.gote_history_local_tables[d].fail(u32::from(&m));
        }
        ev.depth += 1;
        if res.gote_cmp(&ev, pos) == Ordering::Less {
          res.depth = ev.depth;
          res.best_move = BestMove::None;
          res.update_best_move(m, ev);
        }
      }
      if it.legal_moves == 0 {
        res.depth = 0;
        res.best_move = BestMove::One(0);
      }
    }
    res.nodes = (self.nodes - nodes) + (self.hash_nodes - hash_nodes);
    self
      .gote_hash
      .insert_gote(pos.hash, &res, hash_best_move, self.generation);
    res
  }
  fn sente_search(
    &mut self,
    pos: &mut Position,
    depth: u8,
    last_move: Option<&Move>,
  ) -> SearchResult {
    debug_assert_eq!(depth % 2, 1);
    log::debug!("entering sente_search(pos:{}, depth: {})", pos, depth);
    let none_depth = if let Some(q) = self.sente_hash.get_sente(pos.hash) {
      if q.best_move.is_some() || q.depth >= depth {
        self.hash_nodes += q.nodes;
        return q;
      }
      q.depth + 2
    } else {
      2
    };
    let nodes = self.nodes_increment();
    let hash_nodes = self.hash_nodes;
    let mut it = it::SenteMovesIterator::new(pos, last_move);
    let mut res = SearchResult::new(depth);
    while let Some((m, u, oc)) = it.do_next_move(pos) {
      let next_depth = if res.best_move.is_many() {
        res.depth - 3
      } else {
        res.depth - 1
      };
      let ev = self.gote_search(pos, oc, next_depth);
      log::debug!(
        "self.gote_search({}, next_depth: {}) = {:?} after move {}.{}",
        pos,
        next_depth,
        ev,
        pos.move_no - 1,
        shogi::moves::PSNMove::new(&m, &u),
      );
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
      if res.best_move.is_many() && none_depth >= res.depth {
        break;
      }
    }
    res.nodes = (self.nodes - nodes) + (self.hash_nodes - hash_nodes);
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
      log::debug!("depth = {}", depth);
      self.history_resize(depth);
      let ev = self.sente_search(pos, depth, None);
      assert_eq!(hash, pos.hash);
      if ev.best_move.is_some() {
        self.update_global_history();
        if ev.best_move.is_one() {
          let pv = self.extract_pv_from_hash(pos, depth as usize);
          assert_eq!(hash, pos.hash);
          return (Some(ev.depth), Some(pv));
        } else {
          return (Some(ev.depth), None);
        }
      }
    }
    self.update_global_history();
    (None, None)
  }
}

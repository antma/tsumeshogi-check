mod hash;
mod history;
mod it;
mod result;

use super::{shogi, stats};
use result::{BestMove, SearchResult};
use shogi::between::Between;
use shogi::moves::{moves_to_kif, Move, Moves};
use shogi::{alloc::PositionMovesAllocator, Checks, Position};
use std::cmp::Ordering;

#[cfg(feature = "stats")]
#[derive(Default, Debug)]
struct Stats {
  sente_take_mates: u64,
  sente_drop_mates: u64,
  sente_promotion_mates: u64,
  sente_take_cuts: u64,
  sente_drop_cuts: u64,
  sente_promotion_cuts: u64,
  sente_cache_cuts: u64,
  mates_by_pawn_drop: u64,
  skipped_gote_searches_after_pawn_drop: u64,
  gote_cache_cuts: u64,
  max_sente_hash_len: usize,
  max_gote_hash_len: usize,
  sente_skipped_moves: u64,
  sente_skipped_moves_percent: f64,
  sente_illegal_moves: u64,
  sente_illegal_moves_percent: f64,
  sente_legal_moves: u64,
  //gote_skipped_moves: u64,
  //gote_skipped_moves_percent: f64,
  gote_legal_moves: u64,
  gote_is_futile_drop_true: u64,
  gote_is_futile_drop_false: u64,
  gote_is_futile_drop_true_percent: f64,
  //sente
  compute_check_candidates_average: stats::Average,
  compute_drops_with_checks_average: stats::Average,
  compute_drops_no_pawns_with_checks_average: stats::Average,
  //gote
  compute_moves_after_non_blocking_check_average: stats::Average,
  compute_moves_after_sliding_piece_check_average: stats::Average,
  compute_legal_king_moves_average: stats::Average,
  compute_drops_after_sliding_piece_check_average: stats::Average,
}

#[cfg(not(feature = "stats"))]
#[derive(Default, Debug)]
struct Stats {}

pub struct Search {
  sente_hash: hash::SenteHashTable,
  gote_hash: hash::GoteHashTable,
  gote_history: Vec<history::History>,
  allocator: PositionMovesAllocator,
  b: Between,
  pub nodes: u64,
  hash_nodes: u64,
  stats: Stats,
}

impl Search {
  pub fn new(cache_memory: usize) -> Self {
    let m = cache_memory / 2;
    Self {
      sente_hash: hash::SenteHashTable::new(m),
      gote_hash: hash::GoteHashTable::new(m),
      gote_history: Vec::new(),
      allocator: PositionMovesAllocator::default(),
      b: Between::default(),
      nodes: 0,
      hash_nodes: 0,
      stats: Stats::default(),
    }
  }
  fn gote_history_len(&self) -> usize {
    self.gote_history.iter().fold(0, |acc, p| acc + p.len())
  }
  pub fn log_stats(&mut self, puzzles: u32, t: f64) {
    self.hashes_clear();
    if cfg!(feature = "stats") {
      stats::percent!(
        self.stats.sente_skipped_moves_percent,
        self.stats.sente_skipped_moves,
        self.stats.sente_skipped_moves + self.stats.sente_legal_moves
      );
      stats::percent!(
        self.stats.sente_illegal_moves_percent,
        self.stats.sente_illegal_moves,
        self.stats.sente_skipped_moves + self.stats.sente_legal_moves
      );
      stats::percent!(
        self.stats.gote_is_futile_drop_true_percent,
        self.stats.gote_is_futile_drop_true,
        self.stats.gote_is_futile_drop_true + self.stats.gote_is_futile_drop_false
      );
      log::info!("search.stats = {:#?}", self.stats);
    }
    log::info!("{} history tables items", self.gote_history_len());
    log::info!(
      "{} puzzles, {} nodes, {:.3} nps",
      puzzles,
      self.nodes,
      self.nodes as f64 / t
    );
  }
  fn history_resize(&mut self, depth: u8) {
    let d = depth as usize / 2;
    while d >= self.gote_history.len() {
      self.gote_history.push(history::History::default());
    }
  }
  fn history_merge(&mut self) {
    for p in &mut self.gote_history {
      p.merge();
    }
  }
  pub fn hashes_approximate_used_memory(&self) -> u64 {
    self.sente_hash.memory() + self.gote_hash.memory()
  }
  pub fn hashes_clear(&mut self) {
    stats::max!(self.stats.max_sente_hash_len, self.sente_hash.len());
    stats::max!(self.stats.max_gote_hash_len, self.gote_hash.len());
    self.sente_hash.clear();
    self.gote_hash.clear();
  }
  pub fn hashes_remove_unused_entries(&mut self) -> usize {
    self.sente_hash.remove_unused() + self.gote_hash.remove_unused()
  }
  fn next_generation(&mut self) {
    self.sente_hash.next_generation();
    self.gote_hash.next_generation();
  }
  fn on_search_end(&mut self) {
    self.history_merge();
    #[allow(unused)]
    let a = std::mem::take(&mut self.allocator);
    stats::incr!(
      self.stats.compute_check_candidates_average,
      &a.compute_check_candidates_allocator
    );
    stats::incr!(
      self.stats.compute_drops_with_checks_average,
      &a.compute_drops_with_checks_allocator
    );
    stats::incr!(
      self.stats.compute_drops_no_pawns_with_checks_average,
      &a.compute_drops_no_pawns_with_checks_allocator
    );
    stats::incr!(
      self.stats.compute_moves_after_non_blocking_check_average,
      &a.compute_moves_after_non_blocking_check_allocator
    );
    stats::incr!(
      self.stats.compute_moves_after_sliding_piece_check_average,
      &a.compute_moves_after_sliding_piece_check_allocator
    );
    stats::incr!(
      self.stats.compute_legal_king_moves_average,
      &a.compute_legal_king_moves_allocator
    );
    stats::incr!(
      self.stats.compute_drops_after_sliding_piece_check_average,
      &a.compute_drops_after_sliding_piece_check_allocator
    );
  }
  fn nodes_increment(&mut self) -> u64 {
    let r = self.nodes;
    self.nodes += 1;
    r
  }
  fn gote_search(&mut self, pos: &mut Position, checks: Checks, depth: u8) -> SearchResult {
    debug_assert_eq!(depth % 2, 0);
    let mut hash_best_move = None;
    if let Some((q, m)) = self.gote_hash.get(pos.hash) {
      if q.best_move.is_some() || q.depth >= depth {
        self.hash_nodes += q.nodes;
        stats::incr!(self.stats.gote_cache_cuts, 1);
        return q;
      }
      hash_best_move = m;
    }
    let nodes = self.nodes_increment();
    let hash_nodes = self.hash_nodes;
    let mut res = SearchResult::new(0);
    if depth == 0 {
      hash_best_move = pos.is_checkmate_after_check(&mut self.allocator, &checks, &mut self.b);
      if hash_best_move.is_none() {
        res.best_move = BestMove::One(0);
      }
    } else {
      let mut it = it::GoteMovesIterator::new(checks, hash_best_move);
      hash_best_move = None;
      let d = depth as usize / 2;
      let next_depth = depth - 1;
      while let Some((m, u)) =
        it.do_next_move(pos, &mut self.allocator, &self.gote_history[d], &mut self.b)
      {
        let mut ev = self.sente_search(pos, next_depth, Some(&m));
        log::debug!(
          "self.sente_search({}, next_depth: {}) = {:?} after move {}.{}",
          pos,
          next_depth,
          ev,
          pos.move_no - 1,
          shogi::moves::PSNMove::from_undo(&m, &u),
        );
        debug_assert_eq!(ev.depth % 2, 1);
        pos.undo_move(&m, &u);
        let packed_move = if u.taken_piece != 0 {
          m.packed_take_move(u.taken_piece)
        } else {
          u32::from(&m)
        };
        if ev.best_move.is_none() {
          res.depth = depth;
          res.best_move = BestMove::None;
          self.gote_history[d].success(packed_move);
          hash_best_move = Some(m);
          break;
        }
        self.gote_history[d].fail(packed_move);
        ev.depth += 1;
        if res.gote_cmp(&ev, pos) == Ordering::Less {
          res.depth = ev.depth;
          res.store_best_move(&m, ev);
        }
      }
      if it.legal_moves == 0 {
        res.depth = 0;
        res.best_move = BestMove::One(0);
        stats::incr!(
          self.stats.gote_is_futile_drop_true,
          it.stats.is_futile_drop_true as u64
        );
        stats::incr!(
          self.stats.gote_is_futile_drop_false,
          it.stats.is_futile_drop_false as u64
        );
      }
      stats::incr!(self.stats.gote_legal_moves, it.legal_moves as u64);
    }
    res.nodes = (self.nodes - nodes) + (self.hash_nodes - hash_nodes);
    self.gote_hash.insert(pos.hash, &res, hash_best_move);
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
    let none_depth = if let Some(q) = self.sente_hash.get(pos.hash) {
      if q.best_move.is_some() || q.depth >= depth {
        self.hash_nodes += q.nodes;
        stats::incr!(self.stats.sente_cache_cuts, 1);
        return q;
      }
      q.depth + 2
    } else {
      2
    };
    let nodes = self.nodes_increment();
    let hash_nodes = self.hash_nodes;
    let mut it = it::SenteMovesIterator::new(pos, &mut self.allocator, last_move, depth > 1);
    let mut res = SearchResult::new(depth);
    let mut next_depth = res.depth - 1;
    while let Some((m, u, oc)) = it.do_next_move(pos, &mut self.allocator) {
      if next_depth == 0 && m.is_pawn_drop() {
        stats::incr!(self.stats.skipped_gote_searches_after_pawn_drop);
        pos.undo_move(&m, &u);
        continue;
      }
      let ev = self.gote_search(pos, oc, next_depth);
      log::debug!(
        "self.gote_search({}, next_depth: {}) = {:?} after move {}.{}",
        pos,
        next_depth,
        ev,
        pos.move_no - 1,
        shogi::moves::PSNMove::from_undo(&m, &u),
      );
      pos.undo_move(&m, &u);
      if !ev.best_move.is_some() {
        //not mated
        continue;
      }
      if ev.depth == 0 && m.is_pawn_drop() {
        //mate by pawn drop (illegal)
        stats::incr!(self.stats.mates_by_pawn_drop);
        continue;
      }

      if u.is_take() {
        stats::incr!(self.stats.sente_take_mates);
      } else if m.is_drop() {
        stats::incr!(self.stats.sente_drop_mates);
      } else if m.is_promotion() {
        stats::incr!(self.stats.sente_promotion_mates);
      }
      let mate_in = ev.depth + 1;
      if res.depth < mate_in {
        continue;
      }
      if res.depth > mate_in {
        res.depth = mate_in;
        res.store_best_move(&m, ev);
      } else {
        res.update_best_move(&m, ev);
      }
      if res.best_move.is_many() && none_depth >= res.depth {
        if u.is_take() {
          stats::incr!(self.stats.sente_take_cuts);
        } else if m.is_drop() {
          stats::incr!(self.stats.sente_drop_cuts);
        } else if m.is_promotion() {
          stats::incr!(self.stats.sente_promotion_cuts);
        }
        break;
      }
      next_depth = if res.best_move.is_many() {
        res.depth - 3
      } else {
        res.depth - 1
      };
    }
    stats::incr!(
      self.stats.sente_skipped_moves,
      it.stats.skipped_moves as u64
    );
    stats::incr!(
      self.stats.sente_illegal_moves,
      it.stats.illegal_moves as u64
    );
    stats::incr!(self.stats.sente_legal_moves, it.legal_moves as u64);
    res.nodes = (self.nodes - nodes) + (self.hash_nodes - hash_nodes);
    self.sente_hash.insert(pos.hash, &res);
    res
  }
  fn extract_pv_from_hash(&mut self, pos: &mut Position, depth: usize) -> Vec<Move> {
    let mut r = Moves::with_capacity(depth);
    loop {
      let o = if pos.side > 0 {
        self.sente_hash.get(pos.hash)
      } else {
        self.gote_hash.get(pos.hash).map(|p| p.0)
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
    let r = r.only_moves();
    assert_eq!(
      r.len(),
      depth,
      "pos = {}, r = {}",
      pos,
      moves_to_kif(&r, pos.side)
    );
    r
  }
  pub fn search(&mut self, pos: &mut Position, max_depth: u8) -> (Option<u8>, Option<Vec<Move>>) {
    log::debug!("search(pos: {}, max_depth: {})", pos, max_depth);
    assert!(pos.side > 0);
    self.next_generation();
    let hash = pos.hash;
    let mut res = (None, None);
    for depth in (1..=max_depth).step_by(2) {
      log::debug!("depth = {}", depth);
      self.history_resize(depth);
      let ev = self.sente_search(pos, depth, None);
      assert_eq!(hash, pos.hash);
      if ev.best_move.is_some() {
        res.0 = Some(ev.depth);
        if ev.best_move.is_one() {
          let pv = self.extract_pv_from_hash(pos, ev.depth as usize);
          assert_eq!(hash, pos.hash);
          res.1 = Some(pv);
        }
        break;
      }
    }
    self.on_search_end();
    res
  }
}

mod cache;
mod history;
mod it;
mod result;

use super::{shogi, stats};
use result::{BestMove, SearchResult};
use shogi::between::Between;
use shogi::moves::{Move, Moves};
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
  mates_by_pawn_drop: u64,
  skipped_gote_searches_after_pawn_drop: u64,
  cache_size: usize,
  sente_skipped_moves: u64,
  sente_skipped_moves_percent: f64,
  sente_illegal_moves: u64,
  sente_illegal_moves_percent: f64,
  sente_legal_moves: u64,
  //gote_skipped_moves: u64,
  //gote_skipped_moves_percent: f64,
  gote_legal_moves: u64,
  allocator: PositionMovesAllocator,
  //sente
  compute_check_candidates_average: f64,
  compute_drops_with_checks_average: f64,
  compute_drops_no_pawns_with_checks_average: f64,
  //gote
  compute_moves_after_non_blocking_check_average: f64,
  compute_moves_after_sliding_piece_check_average: f64,
  compute_legal_king_moves_average: f64,
  compute_drops_after_sliding_piece_check_average: f64,
}

#[cfg(not(feature = "stats"))]
#[derive(Default, Debug)]
struct Stats {}

pub struct Search {
  sente_hash: cache::SenteCache,
  gote_hash: cache::GoteCache,
  gote_history: Vec<history::History>,
  allocator: PositionMovesAllocator,
  b: Between,
  pub nodes: u64,
  hash_nodes: u64,
  generation: u8,
  stats: Stats,
}

impl Search {
  pub fn new(memory: usize) -> Self {
    let cache_memory = memory / 2;
    //log::debug!("sizeof(SearchResult)={}",std::mem::size_of::<SearchResult>());
    //log::debug!("sizeof(AttackingPieces)={}", std::mem::size_of::<shogi::attacking_pieces::AttackingPieces>());
    Self {
      sente_hash: cache::SenteCache::new(cache_memory),
      gote_hash: cache::GoteCache::new(cache_memory),
      gote_history: Vec::new(),
      allocator: PositionMovesAllocator::default(),
      b: Between::default(),
      nodes: 0,
      hash_nodes: 0,
      generation: 0,
      stats: Stats::default(),
    }
  }
  fn increment_generation(&mut self) {
    self.generation = self.generation.wrapping_add(1);
  }
  fn gote_history_len(&self) -> usize {
    self.gote_history.iter().fold(0, |acc, p| acc + p.len())
  }
  pub fn log_stats(&mut self, puzzles: u32, t: f64) {
    if cfg!(feature = "stats") {
      stats::max!(
        self.stats.cache_size,
        self.sente_hash.len() + self.gote_hash.len()
      );
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
      stats::average!(
        self.stats.compute_check_candidates_average,
        self
          .stats
          .allocator
          .compute_check_candidates_allocator
          .total_moves,
        self
          .stats
          .allocator
          .compute_check_candidates_allocator
          .total_calls
      );
      stats::average!(
        self.stats.compute_drops_with_checks_average,
        self
          .stats
          .allocator
          .compute_drops_with_checks_allocator
          .total_moves,
        self
          .stats
          .allocator
          .compute_drops_with_checks_allocator
          .total_calls
      );
      stats::average!(
        self.stats.compute_drops_no_pawns_with_checks_average,
        self
          .stats
          .allocator
          .compute_drops_no_pawns_with_checks_allocator
          .total_moves,
        self
          .stats
          .allocator
          .compute_drops_no_pawns_with_checks_allocator
          .total_calls
      );
      stats::average!(
        self.stats.compute_drops_after_sliding_piece_check_average,
        self
          .stats
          .allocator
          .compute_drops_after_sliding_piece_check_allocator
          .total_moves,
        self
          .stats
          .allocator
          .compute_drops_after_sliding_piece_check_allocator
          .total_calls
      );
      stats::average!(
        self.stats.compute_moves_after_non_blocking_check_average,
        self
          .stats
          .allocator
          .compute_moves_after_non_blocking_check_allocator
          .total_moves,
        self
          .stats
          .allocator
          .compute_moves_after_non_blocking_check_allocator
          .total_calls
      );
      stats::average!(
        self.stats.compute_moves_after_sliding_piece_check_average,
        self
          .stats
          .allocator
          .compute_moves_after_sliding_piece_check_allocator
          .total_moves,
        self
          .stats
          .allocator
          .compute_moves_after_sliding_piece_check_allocator
          .total_calls
      );
      stats::average!(
        self.stats.compute_legal_king_moves_average,
        self
          .stats
          .allocator
          .compute_legal_king_moves_allocator
          .total_moves,
        self
          .stats
          .allocator
          .compute_legal_king_moves_allocator
          .total_calls
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
  fn on_search_end(&mut self) {
    self.history_merge();
    #[allow(unused)]
    let a = std::mem::take(&mut self.allocator);
    stats::incr!(self.stats.allocator, a);
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
      }
      stats::incr!(self.stats.gote_legal_moves, it.legal_moves as u64);
    }
    res.nodes = (self.nodes - nodes) + (self.hash_nodes - hash_nodes);
    self
      .gote_hash
      .insert(pos.hash, &res, hash_best_move, self.generation);
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
      if res.depth > mate_in {
        res.depth = mate_in;
        res.best_move = BestMove::None;
      } else if res.depth < mate_in {
        continue;
      }
      res.update_best_move(&m, ev);
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
    self.sente_hash.insert(pos.hash, &res, self.generation);
    res
  }
  fn extract_pv_from_hash(&self, pos: &mut Position, depth: usize) -> Vec<Move> {
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
        self.on_search_end();
        if ev.best_move.is_one() {
          let pv = self.extract_pv_from_hash(pos, depth as usize);
          assert_eq!(hash, pos.hash);
          return (Some(ev.depth), Some(pv));
        } else {
          return (Some(ev.depth), None);
        }
      }
    }
    self.on_search_end();
    (None, None)
  }
}

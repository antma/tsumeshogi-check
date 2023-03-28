use crate::shogi;
use shogi::moves::Moves;
use shogi::{Checks, Move, Position};
use std::collections::HashMap;
use std::fmt;

use log::debug;

struct MovesLine {
  a: Vec<String>,
}

impl MovesLine {
  fn push(&mut self, s: String) {
    self.a.push(s);
  }
  fn pop(&mut self) {
    self.a.pop();
  }
}

impl fmt::Display for MovesLine {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (i, s) in self.a.iter().enumerate() {
      if i > 0 {
        write!(f, " ")?;
      }
      write!(f, "{}. {}", i + 1, s)?;
    }
    Ok(())
  }
}

#[derive(Default, Debug)]
struct SearchStats {
  hash_cuts: u64,
  alpha_cuts: u64,
  beta_cuts: u64,
  repetition_cuts: u64,
}

struct HashSlotValue {
  best_move: Option<Move>,
  nodes: u64,
  ev: i32,
}

#[derive(Default)]
struct MateHash {
  h: HashMap<u64, HashSlotValue>,
}

impl MateHash {
  fn get<'a>(&'a self, pos: &Position) -> Option<&'a HashSlotValue> {
    self.h.get(&pos.hash)
    /*
        if let Some(ev) = self.h.get(&pos.hash) {
          //assert_eq!(*(self.fen_hash.get(&pos.hash).unwrap()), pos.to_string());
          Some(*ev)
        } else {
          None
        }
    */
  }
  fn insert(&mut self, pos: &Position, ev: i32, nodes: u64, best_move: Option<Move>) {
    assert!(ev < 255);
    debug!("insert {} ev={}", pos, ev);
    self.h.insert(
      pos.hash,
      HashSlotValue {
        nodes,
        ev,
        best_move,
      },
    );
    //self.fen_hash.insert(pos.hash, pos.to_string());
  }
}

pub struct Search {
  checks: Vec<Checks>,
  line: MovesLine,
  stats: SearchStats,
  mate_hash: MateHash,
  positions_hashes: Vec<u64>,
  pub nodes: u64,
  max_depth: usize,
  allow_futile_drops: bool,
  debug_log: bool,
}

impl Search {
  fn set_max_depth(&mut self, max_depth: usize) {
    //self.cur_line = vec![Move::default(); max_depth + 1];
    self.checks = vec![Checks::default(); max_depth + 1];
    if self.line.a.len() < max_depth + 1 {
      self.line.a.reserve(max_depth + 1 - self.line.a.len());
    }
    self.max_depth = max_depth;
  }
  pub fn new(allow_futile_drops: bool) -> Self {
    Self {
      checks: Vec::default(),
      line: MovesLine { a: Vec::default() },
      stats: SearchStats::default(),
      mate_hash: MateHash::default(),
      positions_hashes: Vec::new(),
      nodes: 0,
      max_depth: 0,
      allow_futile_drops,
      debug_log: log::log_enabled!(log::Level::Debug),
    }
  }
  fn push(&mut self, pos: &Position, cur_depth: usize) {
    assert_eq!(self.positions_hashes.len(), cur_depth);
    self.positions_hashes.push(pos.hash);
  }
  fn pop(&mut self, cur_depth: usize) {
    self.positions_hashes.pop();
    assert_eq!(self.positions_hashes.len(), cur_depth);
  }
  fn repetition(&self, pos: &Position) -> bool {
    let h = pos.hash;
    self
      .positions_hashes
      .iter()
      .step_by(2)
      .find(|&&p| p == h)
      .is_some()
  }
  //maximize
  fn gote_search(&mut self, pos: &mut Position, cur_depth: usize, alpha: i32, beta: i32) -> i32 {
    let nodes = self.nodes;
    self.nodes += 1;
    if beta <= cur_depth as i32 {
      return cur_depth as i32;
    }
    if let Some(q) = self.mate_hash.get(&pos) {
      self.stats.hash_cuts += 1;
      return q.ev + cur_depth as i32;
    }
    let moves = pos.compute_moves(&self.checks[cur_depth]);
    let (takes, king_escapes): (Vec<_>, Vec<_>) = moves.iter().partition(|m| pos.is_take(*m));
    let mut legal_moves = 0;
    let orig_alpha = alpha;
    let mut alpha = alpha;
    let mut best_move: Option<Move> = None;
    let mut best_nodes = 0u64;
    self.push(pos, cur_depth);
    for m in takes.into_iter().chain(king_escapes.into_iter()) {
      if self.debug_log {
        self.line.push(pos.move_to_string(&m, &moves));
      }
      let u = pos.do_move(&m);
      if pos.is_legal() {
        //self.cur_line[cur_depth] = m.clone();
        if cur_depth >= self.max_depth {
          //no mate
          pos.undo_move(&m, &u);
          if self.debug_log {
            self.line.pop();
          }
          self.pop(cur_depth);
          return i32::MAX;
        }
        self.checks[cur_depth + 1] = pos.compute_checks();
        let t = self.nodes;
        let ev = self.sente_search(pos, cur_depth + 1, alpha, beta);
        let t = self.nodes - t;
        if self.debug_log {
          debug!("{}, ev = {}", self.line, ev);
        }
        if alpha < ev || (alpha == ev && best_nodes < t) {
          alpha = ev;
          best_move = Some(m.clone());
          best_nodes = t;
        }
        legal_moves += 1;
      }
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
      if beta <= alpha {
        self.stats.alpha_cuts += 1;
        self.pop(cur_depth);
        return alpha;
      }
    }
    let drops = pos.compute_drops(&self.checks[cur_depth]);
    for m in &drops {
      if self.debug_log {
        self.line.push(pos.move_to_string(m, &moves));
      }
      let u = pos.do_move(&m);
      if pos.is_legal() {
        //self.cur_line[cur_depth] = m.clone();
        if legal_moves == 0
          && !self.allow_futile_drops
          && pos.is_futile_drop(&self.checks[cur_depth], &m)
        {
          //mate
          pos.undo_move(&m, &u);
          if self.debug_log {
            self.line.pop();
          }
          self.mate_hash.insert(&pos, 0, self.nodes - nodes, None);
          self.pop(cur_depth);
          return cur_depth as i32;
        }
        if cur_depth >= self.max_depth {
          //no mate
          pos.undo_move(&m, &u);
          if self.debug_log {
            self.line.pop();
          }
          self.pop(cur_depth);
          return i32::MAX;
        }
        self.checks[cur_depth + 1] = pos.compute_checks();
        let t = self.nodes;
        let ev = self.sente_search(pos, cur_depth + 1, alpha, beta);
        let t = self.nodes - t;
        if self.debug_log {
          debug!("{}, ev = {}", self.line, ev);
        }
        if alpha < ev || (alpha == ev && best_nodes < t) {
          alpha = ev;
          best_move = Some(m.clone());
          best_nodes = t;
        }
        legal_moves += 1;
      }
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
      if beta <= alpha {
        self.stats.alpha_cuts += 1;
        self.pop(cur_depth);
        return alpha;
      }
    }
    if legal_moves == 0 {
      //mate
      self
        .mate_hash
        .insert(&pos, 0, self.nodes - nodes, best_move);
      self.pop(cur_depth);
      return cur_depth as i32;
    }
    if alpha > orig_alpha && alpha <= beta {
      self.mate_hash.insert(
        &pos,
        alpha as i32 - cur_depth as i32,
        self.nodes - nodes,
        best_move,
      );
    }
    self.pop(cur_depth);
    alpha
  }
  //minimize
  fn sente_search(&mut self, pos: &mut Position, cur_depth: usize, alpha: i32, beta: i32) -> i32 {
    let nodes = self.nodes;
    self.nodes += 1;
    if let Some(q) = self.mate_hash.get(&pos) {
      self.stats.hash_cuts += 1;
      return q.ev + cur_depth as i32;
    }
    let eval_lowerbound = (cur_depth + 1) as i32;
    if beta <= eval_lowerbound {
      return eval_lowerbound;
    }
    if self.repetition(pos) {
      self.stats.repetition_cuts += 1;
      return i32::MAX;
    }
    let drops = pos.compute_drops_with_check();
    let moves = pos.compute_moves(&self.checks[cur_depth]);
    let orig_beta = beta;
    let mut beta = beta;
    let mut best_move: Option<Move> = None;
    self.push(pos, cur_depth);
    for m in drops.iter().chain(moves.iter()) {
      if self.debug_log {
        self.line.push(pos.move_to_string(m, &moves));
      }
      let u = pos.do_move(&m);
      if pos.is_legal() {
        //self.cur_line[cur_depth] = m.clone();
        self.checks[cur_depth + 1] = pos.compute_checks();
        if self.checks[cur_depth + 1].is_check() {
          let ev = self.gote_search(pos, cur_depth + 1, alpha, beta);
          if self.debug_log {
            debug!("{}, ev = {}", self.line, ev);
          }
          if !(ev == cur_depth as i32 + 1 && m.is_pawn_drop()) && beta > ev {
            beta = ev;
            best_move = Some(m.clone());
          }
        }
      }
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
      if beta <= alpha {
        /*
        if beta >= alpha && beta < orig_beta {
          self.mate_hash.insert(&pos, beta as u32 - cur_depth as u32);
        }
        */
        self.stats.beta_cuts += 1;
        self.pop(cur_depth);
        return beta;
      }
    }
    if beta >= alpha && beta < orig_beta {
      self.mate_hash.insert(
        &pos,
        beta as i32 - cur_depth as i32,
        self.nodes - nodes,
        best_move,
      );
    }
    self.pop(cur_depth);
    beta
  }
  fn sente_root_search(&mut self, pos: &mut Position, skip_move: Option<Move>) -> i32 {
    let cur_depth = 0;
    let alpha = -1;
    let orig_beta = self.max_depth as i32 + 1;
    let nodes = self.nodes;
    self.nodes += 1;
    let drops = pos.compute_drops_with_check();
    let moves = pos.compute_moves(&self.checks[cur_depth]);
    let mut beta = orig_beta;
    let mut best_move = None;
    self.push(pos, cur_depth);
    for m in drops.iter().chain(moves.iter()) {
      if self.debug_log {
        self.line.push(pos.move_to_string(m, &moves));
      }
      if let Some(u) = skip_move.as_ref() {
        if *u == *m {
          continue;
        }
      }
      let u = pos.do_move(&m);
      if pos.is_legal() {
        self.checks[cur_depth + 1] = pos.compute_checks();
        if self.checks[cur_depth + 1].is_check() {
          let ev = self.gote_search(pos, cur_depth + 1, alpha, beta);
          if self.debug_log {
            debug!("{}, ev = {}", self.line, ev);
          }
          if !(ev == cur_depth as i32 + 1 && m.is_pawn_drop()) && beta > ev {
            beta = ev;
            best_move = Some(m.clone());
          }
        }
      }
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
      if beta <= alpha {
        /*
        if beta >= alpha && beta < orig_beta {
          self.mate_hash.insert(&pos, beta as u32 - cur_depth as u32);
        }
        */
        self.stats.beta_cuts += 1;
        self.pop(cur_depth);
        return beta;
      }
    }
    if skip_move.is_none() && beta >= alpha && beta < orig_beta {
      self.mate_hash.insert(
        &pos,
        beta as i32 - cur_depth as i32,
        self.nodes - nodes,
        best_move,
      );
    }
    self.pop(cur_depth);
    beta
  }
  fn search(&mut self, pos: &mut Position) -> i32 {
    self.checks[0] = pos.compute_checks();
    assert!(self.positions_hashes.is_empty());
    let res = self.sente_root_search(pos, None);
    assert!(self.positions_hashes.is_empty());
    res
  }
  pub fn get_pv_from_hash(&self, pos: &mut Position) -> Option<Vec<Move>> {
    let mut moves = Moves::with_capacity(self.max_depth);
    for _ in 0..self.max_depth {
      if let Some(q) = self.mate_hash.get(&pos) {
        if let Some(m) = q.best_move.as_ref() {
          moves.push(pos, m);
        } else {
          break;
        }
      } else {
        break;
      }
    }
    let l = moves.len();
    moves.undo(pos);
    if l != self.max_depth {
      None
    } else {
      Some(moves.only_moves())
    }
  }
  //returns Some(depth) if tsume in depth moves isn't unique in the line m
  pub fn is_unique_mate(&mut self, pos: &mut Position, m: &Vec<Move>) -> Option<usize> {
    assert_eq!(m.len() % 2, 1);
    let mut moves = Moves::with_capacity(m.len());
    for p in m {
      moves.push(pos, p);
    }
    let mut depth = 1;
    while let Some(p) = moves.pop(pos) {
      self.set_max_depth(depth);
      if self.sente_root_search(pos, Some(p)) == depth as i32 {
        moves.undo(pos);
        return Some(depth);
      }
      moves.pop(pos);
      depth += 2;
    }
    assert_eq!(moves.len(), 0);
    None
  }
  pub fn iterative_search(&mut self, pos: &mut Position, max_depth: usize) -> Option<i32> {
    for depth in (1..=max_depth).step_by(2) {
      self.set_max_depth(depth);
      debug!("depth = {}", depth);
      let ev = self.search(pos);
      if ev == (depth as i32) {
        debug!("stats = {:?}", self.stats);
        return Some(ev);
      }
    }
    None
  }
}

pub fn search_ext(mut pos: Position, max_depth: usize, allow_futile_drops: bool) -> Option<i32> {
  let mut s = Search::new(allow_futile_drops);
  s.iterative_search(&mut pos, max_depth)
}

pub fn search(pos: Position, max_depth: usize) -> Option<i32> {
  search_ext(pos, max_depth, false)
}

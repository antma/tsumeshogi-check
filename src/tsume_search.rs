use crate::shogi;
use moves::{Move, Moves, UndoMove};
use shogi::{moves, Checks, Position};
use std::collections::HashMap;
use std::num::NonZeroU32;

use log::debug;

/*
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
*/

#[derive(Default, Debug)]
struct SearchStats {
  hash_cuts: u64,
  beta_cuts: u64,
  eval_out_of_range_cuts: u64,
  repetition_cuts: u64,
  no_legal_moves_nodes: u64,
  hash_nodes: u64,
  max_hash_nodes: u64,
}

#[derive(Debug)]
struct HashSlotValue {
  nodes: u64,
  best_move: Option<NonZeroU32>,
  lo_ev: i16,
  hi_ev: i16,
  h: u8,
}

#[derive(Debug, Clone)]
enum EvalType {
  Lobound,
  Hibound,
  Exact,
}

fn validate_eval(ev: i16) -> bool {
  let ev = ev.abs();
  ev.abs() <= EVAL_MATE || ev == EVAL_INF
}

impl HashSlotValue {
  fn cut(&self, alpha: i16, beta: i16, h: u8) -> Option<i16> {
    log::trace!("cut(self={:?}, alpha = {}, beta = {})", self, alpha, beta);
    debug_assert!(validate_eval(alpha), "alpha = {}", alpha);
    debug_assert!(validate_eval(beta), "beta = {}", beta);
    assert!(alpha <= beta, "alpha = {}, beta = {}", alpha, beta);
    if self.h < h {
      return None;
    }
    if self.lo_ev == self.hi_ev {
      return Some(self.lo_ev);
    } else {
      if self.lo_ev > -EVAL_INF && beta <= self.lo_ev {
        return Some(beta);
      }
      if self.hi_ev < EVAL_INF && self.hi_ev >= alpha {
        return Some(alpha);
      }
    }
    None
  }
  fn new(et: EvalType, ev: i16, nodes: u64, best_move: Option<NonZeroU32>, h: u8) -> Self {
    let (lo_ev, hi_ev) = match et {
      EvalType::Lobound => (ev, EVAL_INF),
      EvalType::Hibound => (-EVAL_INF, ev),
      EvalType::Exact => (ev, ev),
    };
    HashSlotValue {
      nodes,
      lo_ev,
      hi_ev,
      best_move,
      h,
    }
  }
}

#[derive(Default)]
struct MateHash(HashMap<u64, HashSlotValue>);

fn to_hash_eval(ev: i16, ply: usize) -> i16 {
  if ev.abs() < EVAL_INF {
    let ev = ev + (ply as i16) * ev.signum();
    debug_assert!(ev.abs() <= EVAL_MATE);
    ev
  } else {
    ev
  }
}

fn from_hash_eval(ev: i16, ply: usize) -> i16 {
  if ev.abs() < EVAL_INF {
    let ev = ev - (ply as i16) * ev.signum();
    debug_assert!(ev.abs() <= EVAL_MATE);
    ev
  } else {
    ev
  }
}

fn option_move_to_kif(o: &Option<Move>) -> String {
  match o.as_ref() {
    Some(m) => m.to_kif(&None),
    None => String::from("None"),
  }
}

impl MateHash {
  fn len(&self) -> usize {
    self.0.len()
  }
  fn clear(&mut self) {
    self.0.clear();
  }
  fn get<'a>(&'a self, hash: u64) -> Option<&'a HashSlotValue> {
    self.0.get(&hash)
  }
  fn store(
    &mut self,
    pos: &Position,
    et: EvalType,
    ev: i16,
    nodes: u64,
    best_move: Option<NonZeroU32>,
    h: u8,
  ) {
    debug!(
      "store {}, hash = {:16x}, et = {:?}, ev = {}, best_move = {}, h = {}",
      pos,
      pos.hash,
      et,
      ev,
      option_move_to_kif(&best_move.map(|x| Move::from(x.get()))),
      h
    );
    self
      .0
      .insert(pos.hash, HashSlotValue::new(et, ev, nodes, best_move, h));
  }
}

pub struct Search {
  validate_hash: ValidateHash,
  line: Vec<Move>,
  stats: SearchStats,
  mate_hash: MateHash,
  positions_hashes: Vec<u64>,
  skip_move: Option<Move>,
  pub nodes: u64,
  max_depth: usize,
  mating_side: i8,
  allow_futile_drops: bool,
  debug_log: bool,
}

struct MovesIterator {
  moves: Vec<Move>,
  checks: Checks,
  best_move: Option<Move>,
  k: usize,
  state: u32,
  legal_moves: u32,
  sente: bool,
  expect_futile_drop_check: bool,
}

impl MovesIterator {
  fn compute_moves(&mut self, pos: &Position) {
    self.moves = if self.sente {
      pos.compute_moves(&self.checks)
    } else {
      let mut moves = pos.compute_moves(&self.checks);
      pos.reorder_takes_to_front(&mut moves);
      moves
    };
  }
  fn compute_drops(&mut self, pos: &Position) {
    self.moves = if self.sente {
      pos.compute_drops_with_check()
    } else {
      pos.compute_drops(&self.checks)
    };
  }
  fn new(
    pos: &Position,
    ochecks: Option<Checks>,
    best_move: Option<Move>,
    sente: bool,
    allow_futile_drops: bool,
  ) -> Self {
    Self {
      moves: best_move.clone().into_iter().collect(),
      checks: ochecks.unwrap_or_else(|| pos.compute_checks()),
      best_move: best_move,
      k: 0,
      state: 0,
      legal_moves: 0,
      sente,
      expect_futile_drop_check: !sente && !allow_futile_drops,
    }
  }
  fn next(&mut self, pos: &mut Position) -> Option<(Move, bool)> {
    loop {
      if self.k < self.moves.len() {
        let r = self.moves[self.k].clone();
        self.k += 1;
        if let Some(t) = self.best_move.as_ref() {
          if self.state > 0 && *t == r {
            //don't process best move (from hash) twice
            break Some((r, false));
          }
        }
        break Some((r, true));
      }
      self.moves.clear();
      self.state += 1;
      self.k = 0;
      match self.state {
        1 => self.compute_moves(pos),
        2 => self.compute_drops(pos),
        _ => break None,
      }
    }
  }
  fn do_next_move(&mut self, pos: &mut Position) -> Option<(Move, UndoMove, Option<Checks>)> {
    while let Some((m, unprocessed)) = self.next(pos) {
      let u = pos.do_move(&m);
      let legal = if m.is_drop() && (!self.sente || !self.checks.is_check()) {
        debug_assert!(pos.is_legal());
        true
      } else {
        pos.is_legal()
      };
      if legal {
        let (good, ochecks) = if self.sente {
          let c = if self.sente && m.is_drop() {
            pos.compute_checks_after_drop_with_check(&m)
          } else {
            pos.compute_checks()
          };
          (c.is_check(), Some(c))
        } else {
          if self.k == 1
            && self.state == 2
            && self.expect_futile_drop_check
            && pos.is_futile_drop(&self.checks, &m)
          {
            self.k = self.moves.len();
            pos.undo_move(&m, &u);
            return None;
          } else {
            self.expect_futile_drop_check = false;
            (true, None)
          }
        };
        if good {
          self.legal_moves += 1;
          if !m.is_drop() {
            self.expect_futile_drop_check = false;
          }
          if unprocessed {
            return Some((m, u, ochecks));
          }
        }
      }
      pos.undo_move(&m, &u);
    }
    return None;
  }
}

const EVAL_INF: i16 = i16::MAX - 1;
const EVAL_MATE: i16 = 30000;

#[derive(Default)]
struct ValidateHash(HashMap<u64, String>);
impl ValidateHash {
  fn check(&mut self, pos: &Position) -> bool {
    let mut fen = pos.to_string();
    if let Some((prefix, _)) = fen.rsplit_once(' ') {
      fen.truncate(prefix.len());
    } else {
      return false;
    }
    let mut res = true;
    self
      .0
      .entry(pos.hash)
      .and_modify(|e| {
        if *e != fen {
          res = false;
        }
      })
      .or_insert(fen);
    res
  }
}

impl Search {
  pub fn log_stats(&mut self) {
    self.reset();
    log::info!("stats = {:#?}", self.stats);
  }
  pub fn reset(&mut self) {
    let l = self.mate_hash.len() as u64;
    self.stats.hash_nodes += l;
    self.stats.max_hash_nodes = self.stats.max_hash_nodes.max(l);
    self.nodes = 0;
    self.mate_hash.clear();
  }
  fn set_max_depth(&mut self, max_depth: usize) {
    debug!("set_max_depth({})", max_depth);
    self.max_depth = max_depth;
  }
  pub fn new(allow_futile_drops: bool) -> Self {
    Self {
      validate_hash: ValidateHash::default(),
      line: Vec::new(),
      stats: SearchStats::default(),
      mate_hash: MateHash::default(),
      positions_hashes: Vec::new(),
      skip_move: None,
      nodes: 0,
      max_depth: 0,
      mating_side: 0,
      allow_futile_drops,
      debug_log: log::log_enabled!(log::Level::Debug),
    }
  }
  fn push(&mut self, hash: u64, ply: usize) {
    debug_assert_eq!(self.positions_hashes.len(), ply);
    self.positions_hashes.push(hash);
  }
  fn pop(&mut self, ply: usize) {
    self.positions_hashes.pop();
    debug_assert_eq!(self.positions_hashes.len(), ply);
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
  fn nega_max_search(
    &mut self,
    pos: &mut Position,
    ochecks: Option<Checks>,
    ply: usize,
    mut alpha: i16,
    beta: i16,
  ) -> i16 {
    debug!(
      "nega_max_search(pos: \"{}\", hash: {:16x}, ply: {}, alpha: {}, beta: {}",
      pos, pos.hash, ply, alpha, beta
    );
    debug_assert!(validate_eval(alpha), "alpha = {}", alpha);
    debug_assert!(validate_eval(beta), "beta = {}", beta);
    debug_assert!(alpha <= beta);
    //debug_assert!(self.validate_hash.check(pos));
    let nodes = self.nodes;
    self.nodes += 1;
    let sente = (ply & 1) == 0;
    if sente {
      let max_possible_score = EVAL_MATE - (ply + 1) as i16;
      if max_possible_score <= alpha {
        self.stats.eval_out_of_range_cuts += 1;
        return max_possible_score;
      }
    } else {
      let min_possible_score = -EVAL_MATE + ply as i16;
      if beta <= min_possible_score {
        self.stats.eval_out_of_range_cuts += 1;
        return min_possible_score;
      }
    }
    if self.repetition(pos) {
      self.stats.repetition_cuts += 1;
      return if sente { -EVAL_INF } else { EVAL_INF };
    }
    let h = (self.max_depth - ply) as u8;
    //hash probe and fix mate eval according ply
    let hash = pos.hash;
    let use_hash = ply > 0 || self.skip_move.is_none();
    let mut hash_best_move: Option<Move> = None;
    if use_hash {
      if let Some(q) = self.mate_hash.get(hash) {
        if let Some(ev) = q.cut(to_hash_eval(alpha, ply), to_hash_eval(beta, ply), h) {
          let ev = from_hash_eval(ev, ply);
          debug_assert!(validate_eval(ev), "ev = {}", ev);
          debug!(
            "hash cutoff in position {}, hash = {:16x}, ev = {}, slot.h = {}, h = {}",
            pos, hash, ev, q.h, h
          );
          self.stats.hash_cuts += 1;
          return ev;
        }
        hash_best_move = q.best_move.map(|x| Move::from(x.get()));
      }
    }
    let mut best_move: Option<Move> = None;
    let mut best_nodes = 0;
    let mut it = MovesIterator::new(pos, ochecks, hash_best_move, sente, self.allow_futile_drops);
    self.push(hash, ply);
    while let Some((m, u, oc)) = it.do_next_move(pos) {
      if ply == 0 {
        if let Some(q) = self.skip_move.as_ref() {
          if m == *q {
            pos.undo_move(&m, &u);
            continue;
          }
        }
      }
      if h == 0 {
        debug_assert_eq!(sente, false);
        pos.undo_move(&m, &u);
        self.pop(ply);
        return EVAL_INF;
      }
      if self.debug_log {
        self.line.push(m.clone());
      }
      let t = self.nodes;
      let ev = -self.nega_max_search(pos, oc, ply + 1, -beta, -alpha);
      debug_assert!(validate_eval(ev));
      let t = self.nodes - t;
      debug!(
        "{}: h = {}, ev = {}, nodes = {}",
        moves::moves_to_kif(&self.line, self.mating_side),
        h,
        ev,
        t
      );
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
      debug_assert_eq!(hash, pos.hash);
      if sente && ev == EVAL_MATE - (ply + 1) as i16 && m.is_pawn_drop() {
        //mate by pawn drop
        continue;
      }
      if alpha <= ev {
        if alpha < ev
          || (!sente && self.skip_move.is_none() && best_nodes < t)
          || best_move.is_none()
        {
          alpha = ev;
          best_move = Some(m);
          best_nodes = t;
        }
      }
      if alpha >= beta {
        self.stats.beta_cuts += 1;
        if use_hash {
          self.mate_hash.store(
            pos,
            EvalType::Lobound,
            to_hash_eval(alpha, ply),
            self.nodes - nodes,
            best_move.map(|m| NonZeroU32::new(u32::from(m)).unwrap()),
            h,
          );
        }
        self.pop(ply);
        return alpha;
      }
    }
    assert_eq!(hash, pos.hash);
    self.pop(ply);
    if it.legal_moves == 0 {
      //sente(no legal check moves), gote(mate)
      let alpha = if sente {
        -EVAL_INF
      } else {
        -EVAL_MATE + ply as i16
      };
      self.mate_hash.store(
        pos,
        EvalType::Exact,
        to_hash_eval(alpha, ply),
        self.nodes - nodes,
        None,
        u8::MAX, /* store forever */
      );
      self.stats.no_legal_moves_nodes += 1;
      return alpha;
    }
    if use_hash {
      let ev = to_hash_eval(alpha, ply);
      let nodes = self.nodes - nodes;
      match best_move {
        None => self
          .mate_hash
          .store(pos, EvalType::Hibound, ev, nodes, None, h),
        Some(m) => self.mate_hash.store(
          pos,
          EvalType::Exact,
          ev,
          nodes,
          NonZeroU32::new(u32::from(m)),
          h,
        ),
      }
    }
    return alpha;
  }
  fn search(&mut self, pos: &mut Position) -> i16 {
    self.mating_side = pos.side;
    assert!(self.positions_hashes.is_empty());
    let res = self.nega_max_search(pos, None, 0, -EVAL_INF, EVAL_INF);
    assert!(self.positions_hashes.is_empty());
    res
  }
  pub fn get_pv_from_hash(&self, pos: &mut Position) -> Option<Vec<Move>> {
    let hash = pos.hash;
    let mut moves = Moves::with_capacity(self.max_depth);
    for _ in 0..self.max_depth {
      if let Some(q) = self.mate_hash.get(pos.hash) {
        if let Some(m) = q.best_move {
          moves.push(pos, Move::from(m.get()));
        } else {
          debug!("hash {:16x}, best_move = None", pos.hash);
          break;
        }
      } else {
        debug!(
          "sfen = \"{}\", hash {:16x} isn't in hashtable",
          pos, pos.hash
        );
        break;
      }
    }
    let l = moves.len();
    moves.undo(pos);
    assert_eq!(pos.hash, hash);
    if l != self.max_depth {
      debug!("Fail to find move in hash after {}", moves.to_kif(pos.side));
      None
    } else {
      Some(moves.only_moves())
    }
  }
  //returns Some(depth) if tsume in depth moves isn't unique in the line m
  pub fn is_unique_mate(
    &mut self,
    pos: &mut Position,
    m: &Vec<Move>,
    depth_extention: usize,
  ) -> Option<usize> {
    let hash = pos.hash;
    assert_eq!(m.len() % 2, 1);
    let mut moves = Moves::with_capacity(m.len());
    for p in m {
      moves.push(pos, p.clone());
    }
    let mut depth = 1;
    loop {
      let o = moves.pop(pos);
      if o.is_none() {
        break;
      }
      self.skip_move = o;
      if self
        .iterative_search(pos, depth, depth + depth_extention)
        .is_some()
      {
        moves.undo(pos);
        assert_eq!(pos.hash, hash);
        self.skip_move = None;
        return Some(depth);
      }
      moves.pop(pos);
      depth += 2;
    }
    self.skip_move = None;
    assert_eq!(moves.len(), 0);
    assert_eq!(pos.hash, hash);
    None
  }
  pub fn iterative_search(
    &mut self,
    pos: &mut Position,
    min_depth: usize,
    max_depth: usize,
  ) -> Option<i16> {
    let hash = pos.hash;
    for depth in (min_depth..=max_depth).step_by(2) {
      self.set_max_depth(depth);
      let ev = self.search(pos);
      debug!("depth = {}, ev = {}", depth, ev);
      assert_eq!(pos.hash, hash);
      if ev == (EVAL_MATE - depth as i16) {
        debug!("stats = {:?}", self.stats);
        return Some(depth as i16);
      }
    }
    None
  }
}

pub fn search_ext(mut pos: Position, max_depth: usize, allow_futile_drops: bool) -> Option<i16> {
  let mut s = Search::new(allow_futile_drops);
  s.iterative_search(&mut pos, 1, max_depth)
}

pub fn search(pos: Position, max_depth: usize) -> Option<i16> {
  search_ext(pos, max_depth, false)
}

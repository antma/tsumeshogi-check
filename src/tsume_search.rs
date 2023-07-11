use crate::shogi;
use moves::{Move, Moves, PSNMove};
use shogi::{moves, Checks, Position};
use std::collections::HashMap;
use std::num::NonZeroU32;

use crate::search::it::MovesIterator;

use log::debug;

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
  et: EvalType,
  ev: i8,
  h: u8,
}

#[derive(Debug, Clone)]
enum EvalType {
  Exact,
  Lowerbound,
  Upperbound,
}

impl HashSlotValue {
  fn debug_str(&self) -> String {
    format!(
      "HashSlotValue {{ nodes: {}, best_move: {}, et: {:?}, ev: {}, h: {} }}",
      self.nodes,
      option_move_to_kif(&self.best_move),
      self.et,
      self.ev,
      self.h
    )
  }
  fn new(et: EvalType, ev: i8, nodes: u64, best_move: Option<NonZeroU32>, h: u8) -> Self {
    HashSlotValue {
      nodes,
      et,
      ev,
      best_move,
      h,
    }
  }
}

#[derive(Default)]
struct MateHash(HashMap<u64, HashSlotValue>);

fn to_hash_eval(ev: i32, ply: usize) -> i8 {
  if ev == EVAL_MAX {
    i8::MAX
  } else if ev == EVAL_MIN {
    i8::MIN
  } else {
    let sente = (ply & 1) == 0;
    if sente {
      debug_assert!(ev > 0);
      let d = (EVAL_MATE + 1) - ev;
      debug_assert!(d > 0);
      //debug_assert_eq!(d & 1, 0);
      let d = d - ply as i32;
      debug_assert!(d > 0);
      d as i8
    } else {
      debug_assert!(ev < 0);
      let d = ev + EVAL_MATE - ply as i32;
      debug_assert!(d >= 0);
      -((d) as i8)
    }
  }
}

fn from_hash_eval(ev: i8, ply: usize) -> i32 {
  if ev == i8::MAX {
    EVAL_MAX
  } else if ev == i8::MIN {
    EVAL_MIN
  } else if ev > 0 {
    //sente
    debug_assert_eq!(ply & 1, 0);
    EVAL_MATE + 1 - ev as i32 - ply as i32
  } else {
    debug_assert_eq!(ply & 1, 1);
    //gote
    -EVAL_MATE + ply as i32 - ev as i32
  }
}

fn option_move_to_kif(o: &Option<NonZeroU32>) -> String {
  match o.as_ref() {
    Some(m) => Move::from(m.get()).to_kif(&None),
    None => String::from("None"),
  }
}

fn option_move_to_psn(pos: &Position, o: &Option<Move>) -> String {
  match o.as_ref() {
    Some(m) => m.to_psn(pos.is_take(&m)),
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
    ev: i32,
    nodes: u64,
    best_move: Option<Move>,
    h: u8,
    ply: usize,
  ) {
    let hash_eval = to_hash_eval(ev, ply);
    debug!(
      "store {}, hash = {:16x}, et = {:?}, ev = {}, best_move = {}, h = {}, ply = {}, hash_eval = {}",
      pos,
      pos.hash,
      et,
      ev,
      option_move_to_psn(pos, &best_move),
      h, ply, hash_eval
    );
    debug_assert_eq!(from_hash_eval(hash_eval, ply), ev);
    let bm = match best_move {
      None => None,
      Some(m) => NonZeroU32::new(u32::from(m)),
    };
    self
      .0
      .insert(pos.hash, HashSlotValue::new(et, hash_eval, nodes, bm, h));
  }
}

pub struct Search {
  #[allow(dead_code)]
  validate_hash: ValidateHash,
  line: Vec<PSNMove>,
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

const EVAL_MAX: i32 = i32::MAX;
const EVAL_MIN: i32 = -EVAL_MAX;
const EVAL_MATE: i32 = 30000;

#[derive(Default)]
struct ValidateHash(HashMap<u64, String>);
impl ValidateHash {
  #[allow(dead_code)]
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
      .rev()
      .skip(1)
      .step_by(2)
      .find(|&&p| p == h)
      .is_some()
  }
  fn nega_max_search(
    &mut self,
    pos: &mut Position,
    ochecks: Option<Checks>,
    ply: usize,
    mut alpha: i32,
    mut beta: i32,
  ) -> i32 {
    debug!(
      "nega_max_search(pos: \"{}\", hash: {:16x}, ply: {}, alpha: {}, beta: {})",
      pos, pos.hash, ply, alpha, beta
    );
    debug_assert!(alpha <= beta);
    let nodes = self.nodes;
    self.nodes += 1;
    let sente = (ply & 1) == 0;
    if !sente && self.repetition(pos) {
      log::trace!("repetition cut: {}", moves::moves_to_psn(&self.line));
      self.stats.repetition_cuts += 1;
      return EVAL_MAX;
      //return if sente {EVAL_MIN} else {EVAL_MAX};
    }
    if sente {
      let max_possible_score = EVAL_MATE - (ply + 1) as i32;
      if beta > max_possible_score {
        beta = max_possible_score;
        if alpha >= beta {
          self.stats.eval_out_of_range_cuts += 1;
          return max_possible_score;
        }
      }
    } else {
      let min_possible_score = -EVAL_MATE + ply as i32;
      if alpha < min_possible_score {
        alpha = min_possible_score;
        if alpha >= beta {
          self.stats.eval_out_of_range_cuts += 1;
          return min_possible_score;
        }
      }
    }
    let h = (self.max_depth - ply) as u8;
    //hash probe and fix mate eval according ply
    let hash = pos.hash;
    let mut use_hash = ply > 0 || self.skip_move.is_none();
    let mut hash_best_move: Option<Move> = None;
    if use_hash {
      if let Some(q) = self.mate_hash.get(hash) {
        if q.h >= h {
          let ev = from_hash_eval(q.ev, ply);
          match q.et {
            EvalType::Exact => {
              debug!(
                "hash cutoff in position {}, hash = {:16x}, slot = {}, ev = {}",
                pos,
                hash,
                q.debug_str(),
                ev
              );
              self.stats.hash_cuts += 1;
              return ev;
            }
            EvalType::Lowerbound => {
              if alpha < ev {
                alpha = ev;
              }
            }
            EvalType::Upperbound => {
              if beta > ev {
                beta = ev;
              }
            }
          }
          if alpha >= beta {
            debug!(
              "hash cutoff in position {}, hash = {:16x}, slot = {}, alpha = {}, beta = {}",
              pos,
              hash,
              q.debug_str(),
              alpha,
              beta
            );
            self.stats.hash_cuts += 1;
            return ev;
          }
        }
        hash_best_move = q.best_move.map(|x| Move::from(x.get()));
        if q.h > h {
          use_hash = false;
        }
      }
    }
    let alpha_orig = alpha;
    let mut best_move: Option<Move> = None;
    let mut best_nodes = 0;
    let mut it = MovesIterator::new(pos, ochecks, hash_best_move, sente, self.allow_futile_drops);
    self.push(hash, ply);
    let mut value = EVAL_MIN;
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
        return EVAL_MAX;
      }
      if self.debug_log {
        self.line.push(PSNMove::new(&m, &u));
      }
      let t = self.nodes;
      let ev = -self.nega_max_search(pos, oc, ply + 1, -beta, -alpha);
      let t = self.nodes - t;
      debug!(
        "{}: h = {}, ev = {}, nodes = {}",
        moves::moves_to_psn(&self.line),
        h,
        ev,
        t
      );
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
      debug_assert_eq!(hash, pos.hash);
      if sente && ev == EVAL_MATE - (ply + 1) as i32 && m.is_pawn_drop() {
        //mate by pawn drop
        continue;
      }
      if value <= ev
        && (value < ev
          || (!sente && self.skip_move.is_none() && best_nodes < t)
          || best_move.is_none())
      {
        value = ev;
        best_move = Some(m);
        best_nodes = t;
        if alpha < value {
          alpha = value;
          if alpha >= beta {
            self.stats.beta_cuts += 1;
            break;
          }
        }
      }
    }
    debug_assert_eq!(hash, pos.hash);
    self.pop(ply);
    let nodes = self.nodes - nodes;
    if it.legal_moves == 0 {
      //sente(no legal check moves), gote(mate)
      let alpha = if sente {
        EVAL_MIN
      } else {
        -EVAL_MATE + ply as i32
      };
      self.mate_hash.store(
        pos,
        EvalType::Exact,
        alpha,
        nodes,
        None,
        u8::MAX, /* store forever */
        ply,
      );
      self.stats.no_legal_moves_nodes += 1;
      return alpha;
    }
    if use_hash {
      let et = if value <= alpha_orig {
        EvalType::Upperbound
      } else if value >= beta {
        EvalType::Lowerbound
      } else {
        EvalType::Exact
      };
      self
        .mate_hash
        .store(pos, et, value, nodes, best_move, h, ply);
    }
    value
  }
  fn search(&mut self, pos: &mut Position) -> i32 {
    self.mating_side = pos.side;
    assert!(self.positions_hashes.is_empty());
    let res = self.nega_max_search(pos, None, 0, EVAL_MIN, EVAL_MAX);
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
  ) -> Option<i32> {
    let hash = pos.hash;
    for depth in (min_depth..=max_depth).step_by(2) {
      self.set_max_depth(depth);
      let ev = self.search(pos);
      debug!("depth = {}, ev = {}", depth, ev);
      assert_eq!(pos.hash, hash);
      if ev == (EVAL_MATE - depth as i32) {
        debug!("stats = {:#?}", self.stats);
        return Some(depth as i32);
      }
    }
    debug!("stats = {:#?}", self.stats);
    None
  }
}

pub fn search_ext(mut pos: Position, max_depth: usize, allow_futile_drops: bool) -> Option<i32> {
  let mut s = Search::new(allow_futile_drops);
  s.iterative_search(&mut pos, 1, max_depth)
}

pub fn search(pos: Position, max_depth: usize) -> Option<i32> {
  search_ext(pos, max_depth, false)
}

#[cfg(test)]
mod test {
  use super::*;
  #[test]
  fn test_converting_hash_eval() {
    let h1 = to_hash_eval(EVAL_MATE - 3, 0);
    assert_eq!(from_hash_eval(h1, 2), EVAL_MATE - 5);
    let h2 = to_hash_eval(-(EVAL_MATE - 3), 1);
    assert_eq!(from_hash_eval(h2, 3), -EVAL_MATE + 5);
    assert_eq!(from_hash_eval(h2, 5), -EVAL_MATE + 7);
  }
  #[test]
  fn test_repetition() {
    let mut pos = Position::default();
    let mut s = Search::new(true);
    assert!(!s.repetition(&pos));
    s.push(pos.hash, 0);
    assert!(pos.do_san_move("R7h"));
    assert!(!s.repetition(&pos));
    s.push(pos.hash, 1);
    assert!(pos.do_san_move("R3b"));
    assert!(!s.repetition(&pos));
    s.push(pos.hash, 2);
    assert!(pos.do_san_move("R2h"));
    assert!(!s.repetition(&pos));
    s.push(pos.hash, 3);
    assert!(pos.do_san_move("R8b"));
    assert!(s.repetition(&pos));
    for i in (0..=3).rev() {
      s.pop(i);
    }
    pos = Position::default();
    assert!(!s.repetition(&pos));
    s.push(pos.hash, 0);
    assert!(pos.do_san_move("P9f"));
    assert!(!s.repetition(&pos));
    s.push(pos.hash, 1);
    assert!(pos.do_san_move("R3b"));
    assert!(!s.repetition(&pos));
    s.push(pos.hash, 2);
    assert!(pos.do_san_move("R7h"));
    assert!(!s.repetition(&pos));
    s.push(pos.hash, 3);
    assert!(pos.do_san_move("R8b"));
    assert!(!s.repetition(&pos));
    s.push(pos.hash, 4);
    assert!(pos.do_san_move("R2h"));
    assert!(s.repetition(&pos));
  }
}

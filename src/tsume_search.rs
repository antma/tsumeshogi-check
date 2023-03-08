use crate::shogi_rules;
use shogi_rules::{Checks, Move, Position};
use std::fmt;

#[cfg(not(test))]
use log::debug; // Use log crate when building application

#[cfg(test)]
use std::println as debug; // Workaround to use prinltn! for logs.

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

pub struct Search {
  cur_line: Vec<Move>,
  checks: Vec<Checks>,
  max_depth: usize,
  line: MovesLine,
  allow_futile_drops: bool,
  debug_log: bool,
}

impl Search {
  fn new(max_depth: usize, allow_futile_drops: bool, debug_log: bool) -> Self {
    Self {
      cur_line: vec![Move::default(); max_depth + 1],
      checks: vec![Checks::default(); max_depth + 1],
      max_depth,
      line: MovesLine {
        a: Vec::with_capacity(max_depth + 1),
      },
      allow_futile_drops,
      debug_log,
    }
  }
  //maximize
  fn gote_search(&mut self, pos: &mut Position, cur_depth: usize, alpha: i32, beta: i32) -> i32 {
    if beta < cur_depth as i32 {
      return cur_depth as i32;
    }
    let moves = pos.compute_moves(&self.checks[cur_depth]);
    let (takes, king_escapes): (Vec<_>, Vec<_>) = moves.iter().partition(|m| pos.is_take(*m));
    let mut legal_moves = 0;
    let mut alpha = alpha;
    for m in takes.into_iter().chain(king_escapes.into_iter()) {
      //println!("m = {:?}", m);
      if self.debug_log {
        self.line.push(pos.move_to_string(&m, &moves));
      }
      let u = pos.do_move(&m);
      if pos.is_legal() {
        self.cur_line[cur_depth] = m.clone();
        if cur_depth >= self.max_depth {
          //no mate
          pos.undo_move(&m, &u);
          if self.debug_log {
            self.line.pop();
          }
          return i32::MAX;
        }
        self.checks[cur_depth + 1] = pos.compute_checks();
        let ev = self.sente_search(pos, cur_depth + 1, alpha, beta);
        if self.debug_log {
          debug!("{}, ev = {}", self.line, ev);
        }
        if alpha < ev {
          alpha = ev;
        }
        legal_moves += 1;
      }
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
      if beta < alpha {
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
        self.cur_line[cur_depth] = m.clone();
        if legal_moves == 0
          && !self.allow_futile_drops
          && pos.is_futile_drop(&self.checks[cur_depth], &m)
        {
          //mate
          pos.undo_move(&m, &u);
          if self.debug_log {
            self.line.pop();
          }
          return cur_depth as i32;
        }
        if cur_depth >= self.max_depth {
          //no mate
          pos.undo_move(&m, &u);
          if self.debug_log {
            self.line.pop();
          }
          return i32::MAX;
        }
        self.checks[cur_depth + 1] = pos.compute_checks();
        let ev = self.sente_search(pos, cur_depth + 1, alpha, beta);
        if self.debug_log {
          debug!("{}, ev = {}", self.line, ev);
        }
        if alpha < ev {
          alpha = ev;
        }
        legal_moves += 1;
      }
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
      if beta < alpha {
        return alpha;
      }
    }
    if legal_moves == 0 {
      //mate
      return cur_depth as i32;
    }
    alpha
  }
  //minimize
  fn sente_search(&mut self, pos: &mut Position, cur_depth: usize, alpha: i32, beta: i32) -> i32 {
    let eval_lowerbound = (cur_depth + 1) as i32;
    if beta < eval_lowerbound {
      return eval_lowerbound;
    }
    let drops = pos.compute_drops_with_check();
    let moves = pos.compute_moves(&self.checks[cur_depth]);
    let mut beta = beta;
    for m in drops.iter().chain(moves.iter()) {
      if self.debug_log {
        self.line.push(pos.move_to_string(m, &moves));
      }
      let u = pos.do_move(&m);
      if pos.is_legal() {
        self.cur_line[cur_depth] = m.clone();
        self.checks[cur_depth + 1] = pos.compute_checks();
        if self.checks[cur_depth + 1].is_check() {
          let ev = self.gote_search(pos, cur_depth + 1, alpha, beta);
          if self.debug_log {
            debug!("{}, ev = {}", self.line, ev);
          }
          if !(ev == cur_depth as i32 + 1 && m.is_pawn_drop()) && beta > ev {
            beta = ev;
          }
        }
      }
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
      if beta < alpha {
        return beta;
      }
    }
    beta
  }
  fn search(&mut self, pos: &mut Position) -> i32 {
    self.checks[0] = pos.compute_checks();
    self.sente_search(pos, 0, i32::MIN, i32::MAX)
  }
}

pub fn search_ext(mut pos: Position, max_depth: usize, allow_futile_drops: bool) -> Option<i32> {
  let fen = pos.to_string();
  for depth in (1..=max_depth).step_by(2) {
    debug!("depth = {}", depth);
    let mut s = Search::new(
      depth,
      allow_futile_drops,
      log::log_enabled!(log::Level::Debug),
    );
    let ev = s.search(&mut pos);
    assert_eq!(fen, pos.to_string());
    if ev == (depth as i32) {
      return Some(ev);
    }
  }
  None
}

pub fn search(pos: Position, max_depth: usize) -> Option<i32> {
  search_ext(pos, max_depth, false)
}

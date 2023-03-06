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
  fn gote_search(&mut self, pos: &mut Position, cur_depth: usize) -> i32 {
    let moves = pos.compute_moves(&self.checks[cur_depth]);
    let mut legal_moves = 0;
    let mut best = i32::MIN;
    for m in &moves {
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
        let ev = self.sente_search(pos, cur_depth + 1);
        if self.debug_log {
          debug!("{}, ev = {}", self.line, ev);
        }
        if best < ev {
          best = ev;
        }
        legal_moves += 1;
      }
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
    }
    if legal_moves == 0
      && !self.allow_futile_drops
      && pos.is_unblockable_check(&self.checks[cur_depth])
    {
      //mate
      return cur_depth as i32;
    }
    let drops = pos.compute_drops(&self.checks[cur_depth]);
    for m in &drops {
      if self.debug_log {
        self.line.push(pos.move_to_string(m, &moves));
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
        let ev = self.sente_search(pos, cur_depth + 1);
        if self.debug_log {
          debug!("{}, ev = {}", self.line, ev);
        }
        if best < ev {
          best = ev;
        }
      }
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
      legal_moves += 1;
    }
    if legal_moves == 0 {
      //mate
      return cur_depth as i32;
    }
    best
  }
  //minimize
  fn sente_search(&mut self, pos: &mut Position, cur_depth: usize) -> i32 {
    let drops = pos.compute_drops(&self.checks[cur_depth]);
    let moves = pos.compute_moves(&self.checks[cur_depth]);
    let mut best = i32::MAX;
    for m in drops.iter().chain(moves.iter()) {
      if self.debug_log {
        self.line.push(pos.move_to_string(m, &moves));
      }
      let u = pos.do_move(&m);
      if pos.is_legal() {
        self.cur_line[cur_depth] = m.clone();
        self.checks[cur_depth + 1] = pos.compute_checks();
        if self.checks[cur_depth + 1].is_check() {
          let ev = self.gote_search(pos, cur_depth + 1);
          if self.debug_log {
            debug!("{}, ev = {}", self.line, ev);
          }
          if !(ev == cur_depth as i32 + 1 && m.is_pawn_drop()) && best > ev {
            best = ev;
          }
        }
      }
      pos.undo_move(&m, &u);
      if self.debug_log {
        self.line.pop();
      }
    }
    best
  }
  fn search(&mut self, pos: &mut Position) -> i32 {
    self.checks[0] = pos.compute_checks();
    self.sente_search(pos, 0)
  }
}

pub fn search_ext(
  mut pos: Position,
  max_depth: usize,
  allow_futile_drops: bool
) -> Option<i32> {
  let fen = pos.to_string();
  for depth in (1..=max_depth).step_by(2) {
    debug!("depth = {}", depth);
    let mut s = Search::new(depth, allow_futile_drops, log::log_enabled!(log::Level::Debug));
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

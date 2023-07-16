use crate::shogi;
use shogi::moves::{Move, UndoMove};
use shogi::{Checks, Position};

pub struct SenteMovesIterator {
  moves: Vec<Move>,
  checks: Checks,
  k: usize,
  state: u32,
  pub legal_moves: u32,
}

pub struct GoteMovesIterator {
  moves: Vec<Move>,
  checks: Checks,
  best_move: Option<Move>,
  k: usize,
  takes: usize,
  state: u32,
  pub legal_moves: u32,
  expect_futile_drop_check: bool,
}

impl SenteMovesIterator {
  fn compute_moves(&mut self, pos: &Position) {
    self.moves = pos.compute_moves(&self.checks)
  }
  fn compute_drops(&mut self, pos: &Position) {
    self.moves = pos.compute_drops_with_check()
  }
  pub fn new(pos: &Position, ochecks: Option<Checks>) -> Self {
    Self {
      moves: Vec::new(),
      checks: ochecks.unwrap_or_else(|| pos.compute_checks()),
      k: 0,
      state: 0,
      legal_moves: 0,
    }
  }
  fn next(&mut self, pos: &mut Position) -> Option<Move> {
    loop {
      if self.k < self.moves.len() {
        let r = self.moves[self.k].clone();
        self.k += 1;
        break Some(r);
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
  pub fn do_next_move(&mut self, pos: &mut Position) -> Option<(Move, UndoMove, Option<Checks>)> {
    while let Some(m) = self.next(pos) {
      let u = pos.do_move(&m);
      let legal = if m.is_drop() && !self.checks.is_check() {
        debug_assert!(pos.is_legal());
        true
      } else {
        pos.is_legal()
      };
      if legal {
        let (good, ochecks) = {
          let c = if m.is_drop() {
            pos.compute_checks_after_drop_with_check(&m)
          } else {
            pos.compute_checks()
          };
          (c.is_check(), Some(c))
        };
        if good {
          self.legal_moves += 1;
          return Some((m, u, ochecks));
        }
      }
      pos.undo_move(&m, &u);
    }
    return None;
  }
}

impl GoteMovesIterator {
  fn compute_moves(&mut self, pos: &Position) {
    self.moves = pos.compute_moves(&self.checks);
    let i = pos.reorder_takes_to_front(&mut self.moves);
    self.takes = i;
  }
  fn compute_drops(&mut self, pos: &Position) {
    self.moves = pos.compute_drops(&self.checks);
  }
  pub fn new(
    pos: &Position,
    ochecks: Option<Checks>,
    best_move: Option<Move>,
    allow_futile_drops: bool,
  ) -> Self {
    Self {
      moves: best_move.clone().into_iter().collect(),
      checks: ochecks.unwrap_or_else(|| pos.compute_checks()),
      best_move: best_move,
      k: 0,
      takes: usize::MAX,
      state: 0,
      legal_moves: 0,
      expect_futile_drop_check: !allow_futile_drops,
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
  pub fn do_next_move(&mut self, pos: &mut Position) -> Option<(Move, UndoMove, Option<Checks>)> {
    while let Some((m, unprocessed)) = self.next(pos) {
      let u = pos.do_move(&m);
      let legal = if m.is_drop() {
        debug_assert!(pos.is_legal());
        true
      } else {
        pos.is_legal()
      };
      if legal {
        let (good, ochecks) = {
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
          if !m.is_drop() {
            self.expect_futile_drop_check = false;
          }
          if unprocessed {
            self.legal_moves += 1;
            return Some((m, u, ochecks));
          }
        }
      }
      pos.undo_move(&m, &u);
    }
    return None;
  }
}

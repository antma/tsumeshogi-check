use super::history::History;
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
  fn compute_drops(&mut self, pos: &Position) {
    self.moves = pos.compute_drops_with_check()
  }
  pub fn new(pos: &Position, last_move: Option<&Move>) -> Self {
    let checks = if let Some(m) = last_move {
      pos.compute_checks_after_move(m)
    } else {
      pos.compute_checks()
    };
    Self {
      moves: pos.compute_check_candidates(&checks),
      checks,
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
      self.state += 1;
      self.k = 0;
      match self.state {
        1 => self.compute_drops(pos),
        _ => {
          self.moves.clear();
          break None;
        }
      }
    }
  }
  pub fn do_next_move(&mut self, pos: &mut Position) -> Option<(Move, UndoMove, Checks)> {
    while let Some(m) = self.next(pos) {
      let u = pos.do_move(&m);
      let checks = if m.is_drop() {
        pos.compute_checks_after_drop_with_check(&m)
      } else {
        pos.compute_checks_after_move(&m)
      };
      if checks.is_check() {
        let legal = if !self.checks.is_check() {
          if m.is_drop() {
            debug_assert!(pos.is_legal());
            true
          } else {
            pos.is_legal_after_move_in_checkless_position(&m)
          }
        } else {
          pos.is_legal()
        };
        if legal {
          self.legal_moves += 1;
          return Some((m, u, checks));
        }
      }
      pos.undo_move(&m, &u);
    }
    return None;
  }
}

impl GoteMovesIterator {
  fn compute_moves(&mut self, pos: &Position, history: &History) {
    self.moves = pos.compute_moves(&self.checks);
    let i = pos.reorder_takes_to_front(&mut self.moves);
    self.takes = i;
    history.sort(&mut self.moves[0..i]);
  }
  fn compute_drops(&mut self, pos: &Position, history: &History) {
    self.moves = pos.compute_drops(&self.checks);
    history.sort(&mut self.moves);
  }
  pub fn new(checks: Checks, best_move: Option<Move>, allow_futile_drops: bool) -> Self {
    Self {
      moves: best_move.clone().into_iter().collect(),
      checks,
      best_move: best_move,
      k: 0,
      takes: usize::MAX,
      state: 0,
      legal_moves: 0,
      expect_futile_drop_check: !allow_futile_drops,
    }
  }
  fn next(&mut self, pos: &mut Position, history: &History) -> Option<(Move, bool)> {
    loop {
      if self.k < self.moves.len() {
        if self.state == 1 && self.k == self.takes {
          let n = self.moves.len();
          history.sort(&mut self.moves[self.takes..n]);
        }
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
      self.state += 1;
      self.k = 0;
      match self.state {
        1 => self.compute_moves(pos, history),
        2 => self.compute_drops(pos, history),
        _ => {
          self.moves.clear();
          break None;
        }
      }
    }
  }
  pub fn do_next_move(
    &mut self,
    pos: &mut Position,
    history: &History,
  ) -> Option<(Move, UndoMove)> {
    while let Some((m, unprocessed)) = self.next(pos, history) {
      let u = pos.do_move(&m);
      let legal = if m.is_drop() {
        debug_assert!(pos.is_legal());
        true
      } else {
        pos.is_legal()
      };
      if legal {
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
        }
        if !m.is_drop() {
          self.expect_futile_drop_check = false;
        }
        if unprocessed {
          self.legal_moves += 1;
          return Some((m, u));
        }
      }
      pos.undo_move(&m, &u);
    }
    return None;
  }
}

#[cfg(test)]
fn perft_sente(
  pos: &mut Position,
  v: &mut [u32],
  depth: usize,
  last_move: Option<&Move>,
  history: &History,
) {
  v[depth] += 1;
  let next_depth = depth + 1;
  if next_depth >= v.len() {
    return;
  }
  let mut it = SenteMovesIterator::new(pos, last_move);
  while let Some((m, u, oc)) = it.do_next_move(pos) {
    perft_gote(pos, v, oc, next_depth, history);
    pos.undo_move(&m, &u);
  }
}

#[cfg(test)]
fn perft_gote(pos: &mut Position, v: &mut [u32], checks: Checks, depth: usize, history: &History) {
  v[depth] += 1;
  let next_depth = depth + 1;
  if next_depth >= v.len() {
    return;
  }
  let allow_futile_drops = false;
  let mut it = GoteMovesIterator::new(checks, None, allow_futile_drops);
  while let Some((m, u)) = it.do_next_move(pos, history) {
    perft_sente(pos, v, next_depth, Some(&m), history);
    pos.undo_move(&m, &u);
  }
}

#[cfg(test)]
fn perft(sfen: &str, depth: usize) -> Vec<u32> {
  let history = History::default();
  let mut v = vec![0; depth];
  let mut pos = Position::parse_sfen(sfen).unwrap();
  perft_sente(&mut pos, &mut v, 0, None, &history);
  v
}

#[test]
fn test_perft() {
  assert_eq!(
    perft(
      "G1+R4nl/2l+B1+N3/7pp/pgkpp1s2/1P1n1Pp2/g3P2RP/5+p3/2p3s1K/+b6NL b G2sl6p 1",
      5
    ),
    vec![1, 7, 22, 159, 654]
  );
}

#[test]
fn test_sente_iterator_unique() {
  let mut pos = Position::parse_sfen(
    "lnn5l/2g1S1+Bp1/bp1pk3p/pP1g2p2/4s4/P1R2PP1P/2KP2g2/2S2+r3/LN1G4L b P5pns 1",
  )
  .unwrap();
  let mut it = SenteMovesIterator::new(&pos, None);
  let mut s = std::collections::BTreeSet::new();
  while let Some((m, u, _)) = it.do_next_move(&mut pos) {
    assert!(
      s.insert(u32::from(&m)),
      "duplicate move {}",
      shogi::moves::PSNMove::new(&m, &u)
    );
    pos.undo_move(&m, &u);
  }
}

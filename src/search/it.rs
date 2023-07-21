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
  pub fn new(pos: &Position, ochecks: Option<Checks>) -> Self {
    let checks = ochecks.unwrap_or_else(|| pos.compute_checks());
    Self {
      moves: pos.compute_moves(&checks),
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

fn sort_by_history<F: Fn(&Move) -> f64>(a: &mut [Move], history: &F) {
  let mut b = a
    .iter()
    .map(|v| (v.clone(), history(v)))
    .collect::<Vec<_>>();
  b.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
  for (u, v) in a.iter_mut().zip(b.into_iter()) {
    *u = v.0;
  }
}

impl GoteMovesIterator {
  fn compute_moves<F: Fn(&Move) -> f64>(&mut self, pos: &Position, history: &F) {
    self.moves = pos.compute_moves(&self.checks);
    let i = pos.reorder_takes_to_front(&mut self.moves);
    self.takes = i;
    sort_by_history(&mut self.moves[0..i], history);
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
  fn next<F: Fn(&Move) -> f64>(&mut self, pos: &mut Position, history: &F) -> Option<(Move, bool)> {
    loop {
      if self.k < self.moves.len() {
        if self.state == 1 && self.k == self.takes {
          let n = self.moves.len();
          sort_by_history(&mut self.moves[self.takes..n], history);
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
        2 => self.compute_drops(pos),
        _ => {
          self.moves.clear();
          break None;
        }
      }
    }
  }
  pub fn do_next_move<F: Fn(&Move) -> f64>(
    &mut self,
    pos: &mut Position,
    history: F,
  ) -> Option<(Move, UndoMove, Option<Checks>)> {
    while let Some((m, unprocessed)) = self.next(pos, &history) {
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

#[cfg(test)]
fn perft_sente(pos: &mut Position, v: &mut [u32], ochecks: Option<Checks>, depth: usize) {
  v[depth] += 1;
  let next_depth = depth + 1;
  if next_depth >= v.len() {
    return;
  }
  let mut it = SenteMovesIterator::new(pos, ochecks);
  while let Some((m, u, oc)) = it.do_next_move(pos) {
    perft_gote(pos, v, oc, next_depth);
    pos.undo_move(&m, &u);
  }
}

#[cfg(test)]
fn perft_gote(pos: &mut Position, v: &mut [u32], ochecks: Option<Checks>, depth: usize) {
  v[depth] += 1;
  let next_depth = depth + 1;
  if next_depth >= v.len() {
    return;
  }
  let allow_futile_drops = false;
  let mut it = GoteMovesIterator::new(pos, ochecks, None, allow_futile_drops);
  while let Some((m, u, oc)) = it.do_next_move(pos, |_| 0.0) {
    perft_sente(pos, v, oc, next_depth);
    pos.undo_move(&m, &u);
  }
}

#[cfg(test)]
fn perft(sfen: &str, depth: usize) -> Vec<u32> {
  let mut v = vec![0; depth];
  let mut pos = Position::parse_sfen(sfen).unwrap();
  perft_sente(&mut pos, &mut v, None, 0);
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

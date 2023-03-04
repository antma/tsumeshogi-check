use crate::shogi_rules;
use shogi_rules::Position;

pub struct Search {
  cur_line: Vec<shogi_rules::Move>,
  checks: Vec<shogi_rules::Checks>,
}

impl Search {
  //maximize
  fn gote_search(&mut self, pos: &mut Position, cur_depth: usize) -> i32 {
    let moves = pos.compute_moves(&self.checks[cur_depth]);
    let mut legal_moves = 0;
    let mut best = i32::MIN;
    for m in moves {
      let u = pos.do_move(&m);
      if pos.is_legal() {
        self.cur_line[cur_depth] = m.clone();
        if cur_depth + 1 >= self.checks.len() {
          //no mate
          return i32::MAX;
        }
        self.checks[cur_depth + 1] = pos.compute_checks();
        let ev = self.sente_search(pos, cur_depth + 1);
        if best < ev {
          best = ev;
        }
        legal_moves += 1;
      }
      pos.undo_move(&m, &u);
    }
    if legal_moves == 0 && pos.is_unblockable_check(&self.checks[cur_depth]) {
      //mate
      return cur_depth as i32;
    }
    let drops = pos.compute_drops(&self.checks[cur_depth]);
    for m in drops {
      let u = pos.do_move(&m);
      if pos.is_legal() {
        self.cur_line[cur_depth] = m.clone();
        if cur_depth + 1 >= self.checks.len() {
          //no mate
          return i32::MAX;
        }
        self.checks[cur_depth + 1] = pos.compute_checks();
        let ev = self.sente_search(pos, cur_depth + 1);
        if best < ev {
          best = ev;
        }
      }
      pos.undo_move(&m, &u);
    }
    best
  }
  //minimize
  fn sente_search(&mut self, pos: &mut Position, cur_depth: usize) -> i32 {
    let drops = pos.compute_drops(&self.checks[cur_depth]);
    let moves = pos.compute_moves(&self.checks[cur_depth]);
    let mut best = i32::MAX;
    for m in drops.into_iter().chain(moves.into_iter()) {
      let u = pos.do_move(&m);
      if pos.is_legal() {
        self.cur_line[cur_depth] = m.clone();
        self.checks[cur_depth + 1] = pos.compute_checks();
        if self.checks[cur_depth + 1].is_check() {
          let ev = self.gote_search(pos, cur_depth + 1);
          if best > ev {
            best = ev;
          }
        }
      }
      pos.undo_move(&m, &u);
    }
    best
  }
}

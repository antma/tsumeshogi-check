use crate::shogi_rules;
use shogi_rules::Position;


pub struct Search {
  pv: Vec<shogi_rules::Move>,
  c: Vec<shogi_rules::Checks>,
}

impl Search {
  fn gote_search(&mut self, pos: &mut Position, cur_depth: usize) -> i32 {
    let moves = pos.enumerate_moves();
    for m in moves {

    }
    0
  }
  fn sente_search(&mut self, pos: &mut Position, cur_depth: usize) -> i32 {
    let moves = pos.enumerate_moves();
    for m in moves {
      let u = pos.do_move(&m);
      if pos.is_legal() {
        self.c[cur_depth] = pos.compute_checks();
        if self.c[cur_depth].is_check() {
          let ev = self.gote_search(pos, cur_depth + 1);
        }
      }
      pos.undo_move(&m, &u);
    }
    0
  }
}

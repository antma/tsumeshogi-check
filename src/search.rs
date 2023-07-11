pub mod it;

use super::shogi;
use shogi::moves::Move;
use shogi::Position;

struct Search {
  pos: Position,
}

type PV = Vec<Move>;

/*
impl Search {
  fn sente_search(&mut self, depth: usize) -> (SenteEval, PV) {

  }
}
*/

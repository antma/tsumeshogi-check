use super::result::{BestMove, SearchResult};
use crate::shogi::moves::Move;
use std::collections::HashMap;

pub struct SearchHashValue {
  nodes: u64,
  packed_move: u32,
  depth: u8,
  generation: u8,
}

impl SearchHashValue {
  pub fn to_sente_result(&self) -> SearchResult {
    let best_move = if (self.packed_move & 0x8000_0000) != 0 {
      BestMove::One(self.packed_move & 0x7fff_ffff)
    } else if self.packed_move == 0 {
      BestMove::None
    } else if self.packed_move == 1 {
      BestMove::Many
    } else {
      panic!(
        "can't unpack move from sente search hash value, packed_move = {}",
        self.packed_move
      );
    };
    SearchResult {
      nodes: self.nodes,
      best_move,
      depth: self.depth,
    }
  }
  pub fn new_sente(res: &SearchResult, generation: u8) -> Self {
    let packed_move = match &res.best_move {
      BestMove::None => 0,
      BestMove::One(v) => *v + 0x8000_0000,
      BestMove::Many => 1,
    };
    Self {
      nodes: res.nodes,
      packed_move,
      depth: res.depth,
      generation,
    }
  }
  pub fn to_gote_result(&self) -> (SearchResult, Option<Move>) {
    let (best_move, hash_move) = if self.packed_move == 0 {
      (BestMove::Many, None)
    } else if (self.packed_move & 0x8000_0000) != 0 {
      (BestMove::One(self.packed_move & 0x7fff_ffff), None)
    } else {
      (BestMove::None, Some(Move::from(self.packed_move)))
    };
    (
      SearchResult {
        nodes: self.nodes,
        best_move,
        depth: self.depth,
      },
      hash_move,
    )
  }
  pub fn new_gote(res: &SearchResult, cut_move: Option<Move>, generation: u8) -> Self {
    let packed_move = match &res.best_move {
      BestMove::None => u32::from(cut_move.unwrap()),
      BestMove::One(v) => {
        debug_assert!(cut_move.is_none());
        *v + 0x8000_0000
      }
      BestMove::Many => {
        debug_assert!(cut_move.is_none());
        0
      }
    };
    Self {
      nodes: res.nodes,
      packed_move,
      depth: res.depth,
      generation,
    }
  }
}

pub struct SearchHash(HashMap<u64, SearchHashValue>);

impl Default for SearchHash {
  fn default() -> Self {
    Self(HashMap::new())
  }
}

impl SearchHash {
  pub fn get_sente(&self, x: u64) -> Option<SearchResult> {
    self.0.get(&x).map(|p| p.to_sente_result())
  }
  pub fn get_gote(&self, x: u64) -> Option<(SearchResult, Option<Move>)> {
    self.0.get(&x).map(|p| p.to_gote_result())
  }
  pub fn insert_sente(&mut self, hash: u64, res: &SearchResult, generation: u8) {
    self
      .0
      .insert(hash, SearchHashValue::new_sente(res, generation));
  }
  pub fn insert_gote(
    &mut self,
    hash: u64,
    res: &SearchResult,
    cut_move: Option<Move>,
    generation: u8,
  ) {
    self
      .0
      .insert(hash, SearchHashValue::new_gote(res, cut_move, generation));
  }
  pub fn clear(&mut self) {
    self.0.clear();
  }
  pub fn retain(&mut self, generation: u8, margin: u8) -> usize {
    let l = self.0.len();
    self
      .0
      .retain(|_, v| generation.wrapping_sub(v.generation) < margin);
    l - self.0.len()
  }
}

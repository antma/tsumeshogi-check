use super::result::{BestMove, SearchResult};
use crate::shogi::moves::Move;
use std::collections::HashMap;

#[derive(Default)]
struct Entry {
  nodes: u64,
  packed_move: u32,
  depth: u8,
  used: bool,
}

impl Entry {
  fn to_sente_result(&mut self) -> SearchResult {
    self.used = true;
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
  fn new_sente(res: &SearchResult) -> Self {
    let packed_move = match &res.best_move {
      BestMove::None => 0,
      BestMove::One(v) => *v + 0x8000_0000,
      BestMove::Many => 1,
    };
    Self {
      nodes: res.nodes,
      packed_move,
      depth: res.depth,
      used: true,
    }
  }
  fn to_gote_result(&mut self) -> (SearchResult, Option<Move>) {
    self.used = true;
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
  fn new_gote(res: &SearchResult, cut_move: Option<Move>) -> Self {
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
      used: true,
    }
  }
}

#[test]
fn test_hash_entry() {
  for sr in vec![
    SearchResult {
      best_move: BestMove::One(123),
      nodes: 321,
      depth: 1,
    },
    SearchResult {
      best_move: BestMove::Many,
      nodes: 321,
      depth: 1,
    },
    SearchResult {
      best_move: BestMove::None,
      nodes: 321,
      depth: 1,
    },
  ] {
    let mut h = Entry::new_sente(&sr);
    assert_eq!(h.to_sente_result(), sr);
  }
  let king = crate::shogi::piece::KING;
  let m = Move {
    from: 0,
    to: 10,
    from_piece: king,
    to_piece: king,
  };
  for (sr, cm) in vec![
    (
      SearchResult {
        best_move: BestMove::One(u32::from(&m)),
        nodes: 321,
        depth: 2,
      },
      None,
    ),
    (
      SearchResult {
        best_move: BestMove::Many,
        nodes: 321,
        depth: 1,
      },
      None,
    ),
    (
      SearchResult {
        best_move: BestMove::None,
        nodes: 321,
        depth: 1,
      },
      Some(m),
    ),
  ] {
    let mut h = Entry::new_gote(&sr, cm.clone());
    assert_eq!(h.to_gote_result(), (sr, cm));
  }
}

#[derive(Default)]
struct HashTable(HashMap<u64, Entry>);
impl HashTable {
  fn clear(&mut self) {
    self.0.clear();
  }
  fn remove_unused(&mut self) -> usize {
    let old_len = self.0.len();
    self.0.retain(|_, e| {
      if e.used {
        e.used = false;
        true
      } else {
        false
      }
    });
    old_len - self.0.len()
  }
  #[cfg(feature = "stats")]
  fn len(&self) -> usize {
    self.0.len()
  }
  fn memory(&self) -> u64 {
    self.0.capacity() as u64 * (std::mem::size_of::<Entry>() + std::mem::size_of::<u64>()) as u64
      + std::mem::size_of::<Self>() as u64
  }
}

#[derive(Default)]
pub struct SenteHashTable(HashTable);
#[derive(Default)]
pub struct GoteHashTable(HashTable);
impl SenteHashTable {
  pub fn clear(&mut self) {
    self.0.clear();
  }
  pub fn remove_unused(&mut self) -> usize {
    self.0.remove_unused()
  }
  pub fn get(&mut self, x: u64) -> Option<SearchResult> {
    self.0 .0.get_mut(&x).map(|p| p.to_sente_result())
  }
  pub fn insert(&mut self, hash: u64, res: &SearchResult) {
    self.0 .0.insert(hash, Entry::new_sente(res));
  }
  #[cfg(feature = "stats")]
  pub fn len(&self) -> usize {
    self.0.len()
  }
  pub fn memory(&self) -> u64 {
    self.0.memory()
  }
}

impl GoteHashTable {
  pub fn clear(&mut self) {
    self.0.clear();
  }
  pub fn remove_unused(&mut self) -> usize {
    self.0.remove_unused()
  }
  pub fn get(&mut self, x: u64) -> Option<(SearchResult, Option<Move>)> {
    self.0 .0.get_mut(&x).map(|p| p.to_gote_result())
  }
  pub fn insert(&mut self, hash: u64, res: &SearchResult, cut_move: Option<Move>) {
    self.0 .0.insert(hash, Entry::new_gote(res, cut_move));
  }
  #[cfg(feature = "stats")]
  pub fn len(&self) -> usize {
    self.0.len()
  }
  pub fn memory(&self) -> u64 {
    self.0.memory()
  }
}

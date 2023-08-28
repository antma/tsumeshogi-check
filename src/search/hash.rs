use super::result::{BestMove, SearchResult};
use crate::shogi::moves::Move;
use std::collections::HashMap;

#[derive(Default)]
struct Entry {
  nodes: u64,
  packed_move: u32,
  depth: u8,
  generation: u8,
}

struct CacheSlot {
  key: u64,
  value: Entry,
}

impl Entry {
  fn to_sente_result(&self) -> SearchResult {
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
  fn new_sente(res: &SearchResult, generation: u8) -> Self {
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
  fn to_gote_result(&self) -> (SearchResult, Option<Move>) {
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
  fn new_gote(res: &SearchResult, cut_move: Option<Move>, generation: u8) -> Self {
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
    let h = Entry::new_sente(&sr, 1);
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
    let h = Entry::new_gote(&sr, cm.clone(), 1);
    assert_eq!(h.to_gote_result(), (sr, cm));
  }
}

struct HashTable {
  cache: Vec<CacheSlot>,
  hash: HashMap<u64, Entry>,
  mask: u64,
}

impl HashTable {
  fn clear(&mut self) {
    self.hash.clear();
  }
  fn new(memory: usize) -> Self {
    let k = memory / (std::mem::size_of::<CacheSlot>());
    let m = (1..).find(|i| (1 << i) > k).unwrap() - 1;
    let mask = (1u64 << m) - 1;
    let mut cache = Vec::with_capacity((mask + 1) as usize);
    for i in 0..=mask {
      cache.push(CacheSlot {
        key: i + 1,
        value: Entry::default(),
      });
    }
    Self {
      cache,
      hash: HashMap::default(),
      mask,
    }
  }
  fn get(&self, hash: u64) -> Option<&Entry> {
    let u = &self.cache[(hash & self.mask) as usize];
    if u.key == hash {
      return Some(&u.value);
    }
    self.hash.get(&hash)
  }
  fn insert(&mut self, hash: u64, mut entry: Entry) {
    let u = &mut self.cache[(hash & self.mask) as usize];
    let old_key = u.key;
    let move_to_hash = old_key != hash && u.value.generation == entry.generation;
    if move_to_hash {
      std::mem::swap(&mut entry, &mut u.value);
      u.key = hash;
      self.hash.insert(old_key, entry);
    } else {
      u.key = hash;
      u.value = entry;
    }
  }
}

pub struct SenteHashTable(HashTable);
pub struct GoteHashTable(HashTable);
impl SenteHashTable {
  pub fn clear(&mut self) {
    self.0.clear();
  }
  pub fn new(memory: usize) -> Self {
    Self(HashTable::new(memory))
  }
  pub fn get(&self, x: u64) -> Option<SearchResult> {
    self.0.get(x).map(|p| p.to_sente_result())
  }
  pub fn insert(&mut self, hash: u64, res: &SearchResult, generation: u8) {
    self.0.insert(hash, Entry::new_sente(res, generation));
  }
  #[cfg(feature = "stats")]
  pub fn len(&self) -> usize {
    self.0.hash.len()
  }
}

impl GoteHashTable {
  pub fn clear(&mut self) {
    self.0.clear();
  }
  pub fn new(memory: usize) -> Self {
    Self(HashTable::new(memory))
  }
  pub fn get(&self, x: u64) -> Option<(SearchResult, Option<Move>)> {
    self.0.get(x).map(|p| p.to_gote_result())
  }
  pub fn insert(&mut self, hash: u64, res: &SearchResult, cut_move: Option<Move>, generation: u8) {
    self
      .0
      .insert(hash, Entry::new_gote(res, cut_move, generation));
  }
  #[cfg(feature = "stats")]
  pub fn len(&self) -> usize {
    self.0.hash.len()
  }
}

/*
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
  pub fn capacity(&self) -> usize {
    self.0.capacity()
  }
  #[cfg(feature = "stats")]
  pub fn len(&self) -> usize {
    self.0.len()
  }
}
*/

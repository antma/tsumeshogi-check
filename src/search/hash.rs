use super::result::{BestMove, SearchResult};
use crate::shogi::moves::Move;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Default, Clone, Debug)]
struct Entry {
  nodes: u64,
  packed_move: u32,
  depth: u8,
  generation: u8,
}

impl Entry {
  fn better(&self, other: &Self) -> bool {
    match self.depth.cmp(&other.depth) {
      Ordering::Greater => true,
      Ordering::Equal => self.nodes > other.nodes,
      Ordering::Less => false,
    }
  }
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
    let h = Entry::new_sente(&sr, 0);
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
    let h = Entry::new_gote(&sr, cm.clone(), 0);
    assert_eq!(h.to_gote_result(), (sr, cm));
  }
}

#[derive(Clone, Debug)]
struct CacheSlot {
  key: u64,
  entry: Entry,
}

struct Cache {
  c: Vec<CacheSlot>,
  mask: u64,
}

impl Cache {
  fn new(memory: usize) -> Self {
    let k = memory / (2 * std::mem::size_of::<CacheSlot>());
    let m = (1..).find(|i| (1 << i) > k).unwrap() - 1;
    let mask = (1u64 << m) - 1;
    let mut c = Vec::with_capacity(((mask + 1) * 2) as usize);
    for i in 0..=mask {
      for j in 0..2 {
        c.push(CacheSlot {
          key: i + j + 1,
          entry: Entry {
            nodes: 0,
            packed_move: 0,
            depth: 0,
            generation: 0,
          },
        });
      }
    }
    Self { c, mask }
  }
  fn get_mut(&mut self, hash: u64) -> Option<&mut Entry> {
    let k = 2 * ((hash & self.mask) as usize);
    if self.c[k].key == hash {
      return Some(&mut self.c[k].entry);
    }
    if self.c[k + 1].key == hash {
      return Some(&mut self.c[k + 1].entry);
    }
    None
  }
  fn insert(&mut self, key: u64, entry: Entry) {
    let k = 2 * ((key & self.mask) as usize);
    let mut v = CacheSlot { key, entry };
    let u = &mut self.c[k..k + 2];
    if u[0].entry.generation != v.entry.generation || v.entry.better(&u[0].entry) {
      std::mem::swap(&mut v, &mut u[0]);
      if key != v.key {
        u[1] = v;
      }
    } else {
      assert_ne!(u[0].key, key, "u[0] = {:?}, v = {:?}", u[0], v);
      u[1] = v;
    }
  }
  pub fn remove(&mut self, key: u64) {
    let k = 2 * ((key & self.mask) as usize);
    let u = &mut self.c[k..k + 2];
    assert_ne!(u[0].key, u[1].key, "u = {:?}", u);
    if u[0].key == key {
      u[0] = u[1].clone();
      u[1].key = !key;
    } else if u[1].key == key {
      u[1].key = !key;
    }
  }
}

struct Table {
  //contains slots without moves
  cache: Cache,
  //contains slots with moves
  hash: HashMap<u64, Entry>,
  generation: u8,
}

impl Table {
  fn new(memory: usize) -> Self {
    Self {
      cache: Cache::new(memory),
      hash: HashMap::default(),
      generation: 0,
    }
  }
  fn next_generation(&mut self) {
    self.generation = self.generation.wrapping_add(1);
  }
  fn clear(&mut self) {
    self.hash.clear();
  }
  fn remove_unused(&mut self) -> usize {
    let old_len = self.hash.len();
    let generation = self.generation;
    self.hash.retain(|_, e| e.generation == generation);
    old_len - self.hash.len()
  }
  #[cfg(feature = "stats")]
  fn len(&self) -> usize {
    self.hash.len()
  }
  fn memory(&self) -> u64 {
    self.hash.capacity() as u64 * (std::mem::size_of::<Entry>() + std::mem::size_of::<u64>()) as u64
  }
  pub fn get_mut(&mut self, x: u64) -> Option<&mut Entry> {
    if let Some(p) = self.cache.get_mut(x) {
      p.generation = self.generation;
      return Some(p);
    }
    if let Some(p) = self.hash.get_mut(&x) {
      p.generation = self.generation;
      return Some(p);
    }
    None
  }
}

pub struct SenteHashTable(Table);
pub struct GoteHashTable(Table);
impl SenteHashTable {
  pub fn new(memory: usize) -> Self {
    Self(Table::new(memory))
  }
  pub fn next_generation(&mut self) {
    self.0.next_generation();
  }
  pub fn clear(&mut self) {
    self.0.clear();
  }
  pub fn remove_unused(&mut self) -> usize {
    self.0.remove_unused()
  }
  pub fn get(&mut self, x: u64) -> Option<SearchResult> {
    self.0.get_mut(x).map(|p| p.to_sente_result())
  }
  pub fn insert(&mut self, hash: u64, res: &SearchResult) {
    let entry = Entry::new_sente(res, self.0.generation);
    if res.best_move.with_move() {
      self.0.cache.remove(hash);
      self.0.hash.insert(hash, entry);
    } else {
      self.0.cache.insert(hash, entry);
    }
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
  pub fn new(memory: usize) -> Self {
    Self(Table::new(memory))
  }
  pub fn next_generation(&mut self) {
    self.0.next_generation();
  }
  pub fn clear(&mut self) {
    self.0.clear();
  }
  pub fn remove_unused(&mut self) -> usize {
    self.0.remove_unused()
  }
  pub fn get(&mut self, x: u64) -> Option<(SearchResult, Option<Move>)> {
    self.0.get_mut(x).map(|p| p.to_gote_result())
  }
  pub fn insert(&mut self, hash: u64, res: &SearchResult, cut_move: Option<Move>) {
    let entry = Entry::new_gote(res, cut_move, self.0.generation);
    if res.best_move.with_move() {
      self.0.cache.remove(hash);
      self.0.hash.insert(hash, entry);
    } else {
      self.0.cache.insert(hash, entry);
    }
  }
  #[cfg(feature = "stats")]
  pub fn len(&self) -> usize {
    self.0.len()
  }
  pub fn memory(&self) -> u64 {
    self.0.memory()
  }
}

use super::result::{BestMove, SearchResult};
use crate::shogi::moves::Move;

struct Slot {
  key: u64,
  nodes: u64,
  packed_move: u32,
  depth: u8,
  generation: u8,
}

impl Slot {
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
  fn new_sente(key: u64, res: &SearchResult, generation: u8) -> Self {
    let packed_move = match &res.best_move {
      BestMove::None => 0,
      BestMove::One(v) => *v + 0x8000_0000,
      BestMove::Many => 1,
    };
    Self {
      key,
      nodes: res.nodes,
      packed_move,
      depth: res.depth,
      generation,
    }
  }
  fn new_gote(key: u64, res: &SearchResult, cut_move: Option<Move>, generation: u8) -> Self {
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
      key,
      nodes: res.nodes,
      packed_move,
      depth: res.depth,
      generation,
    }
  }
}

struct Cache {
  c: Vec<Slot>,
  mask: u64,
}

impl Cache {
  fn new(memory: usize) -> Self {
    let k = memory / (2 * std::mem::size_of::<Slot>());
    let m = (1..).find(|i| (1 << i) > k).unwrap() - 1;
    let mask = (1u64 << m) - 1;
    let mut c = Vec::with_capacity(((mask + 1) * 2) as usize);
    for i in 0..=mask {
      for _ in 0..2 {
        c.push(Slot {
          key: i + 1,
          nodes: 0,
          packed_move: 0,
          depth: 0,
          generation: 0,
        });
      }
    }
    Self { c, mask }
  }
  fn get(&self, hash: u64) -> Option<&Slot> {
    let k = 2 * ((hash & self.mask) as usize);
    if self.c[k].key == hash {
      return Some(&self.c[k]);
    }
    if self.c[k + 1].key == hash {
      return Some(&self.c[k + 1]);
    }
    None
  }
  fn insert(&mut self, mut v: Slot) {
    let k = 2 * ((v.key & self.mask) as usize);
    let u = &mut self.c[k..k + 2];
    if v.nodes > u[0].nodes || v.generation != u[0].generation {
      std::mem::swap(&mut v, &mut u[0]);
      u[1] = v;
    } else {
      u[1] = v;
    }
  }
}

pub struct SenteCache(Cache);
pub struct GoteCache(Cache);
impl SenteCache {
  #[cfg(feature = "stats")]
  pub fn len(&self) -> usize {
    self.0.c.len()
  }
  pub fn new(memory: usize) -> Self {
    Self(Cache::new(memory))
  }
  pub fn get(&self, x: u64) -> Option<SearchResult> {
    self.0.get(x).map(|p| p.to_sente_result())
  }
  pub fn insert(&mut self, hash: u64, res: &SearchResult, generation: u8) {
    self.0.insert(Slot::new_sente(hash, res, generation));
  }
}

impl GoteCache {
  #[cfg(feature = "stats")]
  pub fn len(&self) -> usize {
    self.0.c.len()
  }
  pub fn new(memory: usize) -> Self {
    Self(Cache::new(memory))
  }
  pub fn get(&self, x: u64) -> Option<(SearchResult, Option<Move>)> {
    self.0.get(x).map(|p| p.to_gote_result())
  }
  pub fn insert(&mut self, hash: u64, res: &SearchResult, cut_move: Option<Move>, generation: u8) {
    self
      .0
      .insert(Slot::new_gote(hash, res, cut_move, generation));
  }
}

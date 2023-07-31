struct HistoryEntry {
  success: u64,
  total: u64,
}

impl HistoryEntry {
  fn get(&self) -> f64 {
    self.success as f64 / self.total as f64
  }
}

impl Default for HistoryEntry {
  fn default() -> Self {
    HistoryEntry {
      success: 1,
      total: 1,
    }
  }
}

#[derive(Default)]
struct HistoryTable(std::collections::HashMap<u32, HistoryEntry>);
impl HistoryTable {
  fn len(&self) -> usize {
    self.0.len()
  }
  fn get(&self, packed_move: u32) -> f64 {
    self
      .0
      .get(&packed_move)
      .map(HistoryEntry::get)
      .unwrap_or(1.0f64)
  }
  fn success(&mut self, packed_move: u32) {
    self
      .0
      .entry(packed_move)
      .and_modify(|e| {
        e.success += 1;
        e.total += 1;
      })
      .or_insert_with(HistoryEntry::default);
  }
  fn fail(&mut self, packed_move: u32) {
    self
      .0
      .entry(packed_move)
      .and_modify(|e| e.total += 1)
      .or_insert_with(HistoryEntry::default);
  }
  fn merge(&mut self, other: Self) {
    for (key, value) in other.0 {
      self
        .0
        .entry(key)
        .and_modify(|e| {
          e.success += value.success;
          e.total += value.total;
        })
        .or_insert(value);
    }
  }
}

#[derive(Default)]
pub struct History {
  local: HistoryTable,
  global: HistoryTable,
}

struct F64(f64);
impl PartialOrd for F64 {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    other.0.partial_cmp(&self.0)
  }
}
impl Ord for F64 {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    other.0.partial_cmp(&self.0).unwrap()
  }
}
impl PartialEq for F64 {
  fn eq(&self, other: &F64) -> bool {
    self.0 == other.0
  }
}
impl Eq for F64 {}

impl History {
  pub fn len(&self) -> usize {
    self.global.len() + self.local.len()
  }
  fn get(&self, packed_move: u32) -> f64 {
    self.local.get(packed_move) * self.global.get(packed_move)
  }
  pub fn success(&mut self, packed_move: u32) {
    self.local.success(packed_move)
  }
  pub fn fail(&mut self, packed_move: u32) {
    self.local.fail(packed_move)
  }
  pub fn merge(&mut self) {
    let local = std::mem::take(&mut self.local);
    self.global.merge(local);
  }
  pub fn sort(&self, m: &mut [crate::shogi::moves::Move]) {
    m.sort_by_cached_key(|p| F64(self.get(u32::from(p))));
  }
}

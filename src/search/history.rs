//use crate::shogi::moves::Move;

struct HistoryEntry {
  success: u64,
  total: u64,
}

impl HistoryEntry {
  fn get(&self) -> f64 {
    self.success as f64 / self.total as f64
  }
}

#[derive(Default)]
pub struct HistoryTable(std::collections::HashMap<u32, HistoryEntry>);
impl HistoryTable {
  pub fn get(&self, packed_move: u32) -> f64 {
    self
      .0
      .get(&packed_move)
      .map(HistoryEntry::get)
      .unwrap_or(1.0f64)
  }
  pub fn success(&mut self, packed_move: u32) {
    self
      .0
      .entry(packed_move)
      .and_modify(|e| {
        e.success += 1;
        e.total += 1;
      })
      .or_insert_with(|| HistoryEntry {
        success: 1,
        total: 1,
      });
  }
  pub fn fail(&mut self, packed_move: u32) {
    self
      .0
      .entry(packed_move)
      .and_modify(|e| e.total += 1)
      .or_insert_with(|| HistoryEntry {
        success: 1,
        total: 1,
      });
  }
}

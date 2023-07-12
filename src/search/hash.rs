use std::collections::HashMap;

pub struct SearchHash<T>(HashMap<u64, (T, u8)>);

impl<T> Default for SearchHash<T> {
  fn default() -> Self {
    Self(HashMap::new())
  }
}

impl<T> SearchHash<T> {
  pub fn get(&self, x: u64) -> Option<&T> {
    self.0.get(&x).map(|p| &p.0)
  }
  pub fn set(&mut self, x: u64, t: T, generation: u8) {
    self.0.insert(x, (t, generation));
  }
  pub fn clear(&mut self) {
    self.0.clear();
  }
  pub fn retain(&mut self, generation: u8, margin: u8) -> usize {
    let l = self.0.len();
    self.0.retain(|_, v| generation.wrapping_sub(v.1) < margin);
    l - self.0.len()
  }
}

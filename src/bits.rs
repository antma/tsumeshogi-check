pub struct BitIterator(u32);

impl Iterator for BitIterator {
  type Item = u32;
  fn next(&mut self) -> Option<Self::Item> {
    if self.0 == 0 {
      None
    } else {
      let i = self.0.trailing_zeros();
      self.0 ^= 1 << i;
      Some(i)
    }
  }
}

pub struct Bits(pub u32);
impl IntoIterator for Bits {
  type Item = u32;
  type IntoIter = BitIterator;
  fn into_iter(self) -> Self::IntoIter {
    BitIterator(self.0)
  }
}

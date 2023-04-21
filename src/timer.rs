pub struct Timer(std::time::SystemTime);

impl Timer {
  pub fn new() -> Self {
    Self(std::time::SystemTime::now())
  }
  pub fn elapsed(&self) -> f64 {
    self.0.elapsed().unwrap().as_secs_f64()
  }
}

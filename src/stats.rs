use std::fmt;

#[derive(Default)]
pub struct Average {
  sum: u64,
  total: u64,
}

impl std::ops::AddAssign<u64> for Average {
  fn add_assign(&mut self, rhs: u64) {
    self.sum += rhs;
    self.total += 1;
  }
}

impl Average {
  fn average(&self) -> f64 {
    if self.total == 0 {
      0.0
    } else {
      self.sum as f64 / self.total as f64
    }
  }
}

impl fmt::Debug for Average {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:.10}", self.average())
  }
}

#[cfg(feature = "stats")]
macro_rules! incr {
  ($e:expr) => {
    $e += 1;
  };
  ($x:expr, $y:expr) => {
    $x += $y;
  };
}

#[cfg(feature = "stats")]
macro_rules! max {
  ($e:expr, $v: expr) => {
    let t = $v;
    if $e < t {
      $e = t;
    }
  };
}

#[cfg(feature = "stats")]
macro_rules! percent {
  ($e: expr, $num: expr, $den: expr) => {
    let t = $den;
    $e = if t == 0 {
      0.0
    } else {
      100.0 * (($num) as f64) / (t as f64)
    };
  };
}

#[cfg(feature = "stats")]
macro_rules! average {
  ($e: expr, $num: expr, $den: expr) => {
    let t = $den;
    $e = if t == 0 {
      0.0
    } else {
      (($num) as f64) / (t as f64)
    };
  };
}

#[cfg(not(feature = "stats"))]
macro_rules! incr {
  ($e:expr) => {};
  ($x:expr, $y:expr) => {};
}
#[cfg(not(feature = "stats"))]
macro_rules! max {
  ($e:expr, $v: expr) => {};
}
#[cfg(not(feature = "stats"))]
macro_rules! percent {
  ($e: expr, $num: expr, $den: expr) => {};
}
#[cfg(not(feature = "stats"))]
macro_rules! average {
  ($e: expr, $num: expr, $den: expr) => {};
}

pub(crate) use {average, incr, max, percent};

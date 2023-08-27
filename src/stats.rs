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

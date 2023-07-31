#[cfg(feature = "stats")]
macro_rules! incr {
  ($e:expr) => {
    $e += 1;
  };
}

#[cfg(feature = "stats")]
macro_rules! max {
  ($e:expr, $v: expr) => {{
    let t = $v;
    if $e < t {
      $e = t;
    }
  }};
}

#[cfg(not(feature = "stats"))]
macro_rules! incr {
  ($e:expr) => {};
}
#[cfg(not(feature = "stats"))]
macro_rules! max {
  ($e:expr, $v: expr) => {};
}

pub(crate) use incr;
pub(crate) use max;

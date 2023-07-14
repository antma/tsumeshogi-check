use std::convert::TryFrom;
use tsumeshogi_check::search;
use tsumeshogi_check::shogi::Position;
use tsumeshogi_check::tsume_search::search_ext;

pub fn tsume_batch_test_ext(
  v: Vec<&str>,
  depth: usize,
  ans: Option<i32>,
  allow_futile_drops: bool,
) {
  let mut s = std::collections::BTreeSet::new();
  for (test, sfen) in v.into_iter().enumerate() {
    assert!(
      s.insert(sfen),
      "test #{}, duplicated sfen: {}",
      test + 1,
      sfen
    );
    let pos = Position::parse_sfen(&sfen).unwrap();
    assert_eq!(
      search_ext(pos, depth, allow_futile_drops),
      ans,
      "test #{}, sfen: {}",
      test + 1,
      sfen
    );
    let mut pos = Position::parse_sfen(&sfen).unwrap();
    if pos.side < 0 {
      pos.swap_sides();
    }
    let mut s = search::Search::default();
    assert_eq!(
      s.search(&mut pos, depth as u8).0,
      ans.map(|i| u8::try_from(i).unwrap()),
      "test #{}, sfen: {}",
      test + 1,
      sfen
    );
  }
}

pub fn tsume_batch_test(v: Vec<&str>, depth: usize) {
  tsume_batch_test_ext(v, depth, Some(depth as i32), false);
}

pub fn no_tsume_batch_test(v: Vec<&str>, depth: usize) {
  tsume_batch_test_ext(v, depth, None, false);
}

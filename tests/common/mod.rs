use tsumeshogi_check::shogi_rules::Position;
use tsumeshogi_check::tsume_search::search_ext;

fn tsume_batch_test_ext(v: Vec<&str>, depth: usize, ans: Option<i32>) {
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
      search_ext(pos, depth, false),
      ans,
      "test #{}, sfen: {}",
      test + 1,
      sfen
    );
  }
}

pub fn tsume_batch_test(v: Vec<&str>, depth: usize) {
  tsume_batch_test_ext(v, depth, Some(depth as i32));
}

pub fn no_tsume_batch_test(v: Vec<&str>, depth: usize) {
  tsume_batch_test_ext(v, depth, None);
}

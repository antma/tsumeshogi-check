use tsumeshogi_check::shogi_rules::Position;
use tsumeshogi_check::tsume_search::search;

#[test]
fn tsume3() {
  for sfen in vec![
      "3sks3/9/4S4/9/9/8B/9/9/9 b S 1",
      "9/4k4/9/4P4/9/9/9/9/9 b 2G2r2b4s4n4l17p 1",
      ] {
    let pos = Position::parse_sfen(&sfen).unwrap();
    assert_eq!(search(pos, 3), Some(3));
  }
}

#[test]
fn no_tsume3() {
  for sfen in vec![
      "9/4k4/9/4P4/9/9/9/9/9 b G2r2b4s4n4l17p 1"
      ] {
    let pos = Position::parse_sfen(&sfen).unwrap();
    assert_eq!(search(pos, 3), None);
  }
}

use tsumeshogi_check::shogi_rules::Position;
use tsumeshogi_check::tsume_search::search;

#[test]
fn tsume1() {
  for sfen in vec![
    "k8/9/PK7/9/9/9/9/9/9 b 2r2b4g4s4n4l17p 1",
    "3k5/9/3S5/9/9/9/9/9/9 b S2r2b4g2s4n4l18p 1",
  ] {
    let mut pos = Position::parse_sfen(&sfen).unwrap();
    assert_eq!(search(pos, 1), Some(1), "sfen: {}", sfen);
  }
}

use tsumeshogi_check::shogi_rules::Position;
use tsumeshogi_check::tsume_search::search;

#[test]
fn tsume1() {
  for sfen in vec![
    "k8/9/PK7/9/9/9/9/9/9 b 2r2b4g4s4n4l17p 1",
    "3k5/9/3S5/9/9/9/9/9/9 b S2r2b4g2s4n4l18p 1",
    "k8/9/1K7/9/9/9/8B/8B/9 b 2r4g4s4n4l18p 1",
    "kb7/p8/9/9/9/9/8B/9/8K b 2R4G4S4N4L17P 1",
  ] {
    let pos = Position::parse_sfen(&sfen).unwrap();
    assert_eq!(search(pos, 1), Some(1), "sfen: {}", sfen);
  }
}

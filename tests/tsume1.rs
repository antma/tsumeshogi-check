use tsumeshogi_check::shogi_rules::Position;
use tsumeshogi_check::tsume_search::search;

#[test]
fn tsume1() {
  for (test, sfen) in vec![
    "k8/9/PK7/9/9/9/9/9/9 b 2r2b4g4s4n4l17p 1",
    "3k5/9/3S5/9/9/9/9/9/9 b S2r2b4g2s4n4l18p 1",
    "k8/9/1K7/9/9/9/8B/8B/9 b 2r4g4s4n4l18p 1",
    "kb7/p8/9/9/9/9/8B/9/8K b 2R4G4S4N4L17P 1",
    "7pk/9/7K1/5+r3/9/9/9/1B7/R8 b b4g4s4n4l18p 1",
    "4k4/9/4P4/9/9/9/9/9/9 b G2r2b3g4s4n4l17p 1",
    "4k4/9/4S4/9/9/9/9/9/9 b S2r2b4g2s4n4l18p 1",
    "kn7/ps7/9/9/N8/9/9/9/8B b 2r2b4g2s4n4l18p 1",
    "9/2B6/3G2r2/3pkL1R1/3gsg3/3S2B2/3g5/4N4/4L4 b 2s3n2l17p 1",
  ]
  .into_iter()
  .enumerate()
  {
    let pos = Position::parse_sfen(&sfen).unwrap();
    assert_eq!(
      search(pos, 1),
      Some(1),
      "test #{}, sfen: {}",
      test + 1,
      sfen
    );
  }
}

#[test]
fn tsume1_futile_drops() {
  for (test, sfen) in vec!["6Snl/5+Rg1k/6ppp/9/9/9/9/9/9 b r2b3g3s3n3l15p 5"]
    .into_iter()
    .enumerate()
  {
    let pos = Position::parse_sfen(&sfen).unwrap();
    assert_eq!(
      search(pos, 1),
      Some(1),
      "test #{}, sfen: {}",
      test + 1,
      sfen
    );
  }
}

#[test]
fn pawn_drop_no_mate() {
  for (test, sfen) in vec!["kn7/1s7/9/1N7/9/9/9/9/9 b P2r2b4g3s2n4l17p 1"]
    .into_iter()
    .enumerate()
  {
    let pos = Position::parse_sfen(&sfen).unwrap();
    assert_eq!(search(pos, 1), None, "test #{}, sfen: {}", test + 1, sfen);
  }
}

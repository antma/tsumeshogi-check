use tsumeshogi_check::shogi_rules::Position;
use tsumeshogi_check::tsume_search::search;

#[test]
fn tsume3() {
  for (test, sfen) in vec![
    "3sks3/9/4S4/9/9/8B/9/9/9 b S 1",
    "9/4k4/9/4P4/9/9/9/9/9 b 2G2r2b4s4n4l17p 1",
    "9/6k2/9/5G3/4G4/4P4/9/9/9 b 2G2r2b4s4n4l17p 1",
    "9/4k4/9/5G3/4G4/4P4/9/9/9 b 2G2r2b4s4n4l17p 1",
    "9/5k3/9/5G3/4G4/4P4/9/9/9 b 2G2r2b4s4n4l17p 1",
    "7nl/7k1/6Ppp/9/9/9/9/9/9 b 2G2r2b2g4s3n3l15p 1",
    "7nl/7k1/5+Pppp/9/9/9/9/9/9 b GS2r2b2g4s3n3l14p 1",
    "5g1nl/4g1sk1/4ppppp/9/4B4/9/9/9/9 b GN2rbg3s2n3l13p 1",
    "5k3/7gR/4S4/7L1/9/9/9/9/9 b Sr2b3g2s4n3l18p 1",
    "8l/6nkg/5PpBp/5+r3/7R1/9/9/9/9 b b3g4s3n3l15p 1",
    "9/9/9/9/9/7np/8k/8B/7LR b rb4g4s3n3l17p 1",
  ]
  .into_iter()
  .enumerate()
  {
    let pos = Position::parse_sfen(&sfen).unwrap();
    assert_eq!(
      search(pos, 3),
      Some(3),
      "test #{}, sfen: {}",
      test + 1,
      sfen
    );
  }
}

#[test]
fn no_tsume3() {
  for sfen in vec!["9/4k4/9/4P4/9/9/9/9/9 b G2r2b4s4n4l17p 1"] {
    let pos = Position::parse_sfen(&sfen).unwrap();
    assert_eq!(search(pos, 3), None);
  }
}

//futile drops
//9/2B+Pg4/2gp5/5p3/R2LkN3/2NP5/2G1PP3/9/9 b rbg4s2n3l14p 1

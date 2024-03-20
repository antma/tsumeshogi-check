use tsumeshogi_check::shogi::Position;

#[test]
fn pgn_moves() {
  let mut pos = Position::default();
  for s in "d1d2 b7b6 c3c4 g7g6 f1f2 b6b5 b2c3 c9d8 e3e4 c7c6 c1c2 d8c7 h3h4 a7a6 f2e3 c7d6 h4h5 f9g8 e1f1 b9c7 d3d4 b8b6 c2d3 c6c5 g1f2 g9f8 h5h6 h7h6 h2h6 f8g7 h6h2 P@h7 c4c5 d6c5 d2e2 b5b4 b3b4 c5b4 c3e1 P@b2 P@c6 b6c6  P@h6 g7h6 P@h5 h6g5 e1b4 c6c1+ e2e1 d9e8 f2g1 b2b1+ h2c2 P@b5 c2c1 b1c1 R@c9 e9f8 b4c5 R@b1 S@f9 c1d1 f9e8+ f8e8 e3e2 d1e1 e2e1 G@d2 G@e2 d2e1 e2e1 S@d2 G@e2 d2e1+ e2e1 G@d2 S@f2 d2d3 f1g2 g5h4 g1h2 N@g5 c9c8 G@d8 c8d8+ e8d8 G@h3 g5h3+ h2h3 h4h3+ g2f1 G@g2".split_whitespace() {
    let m = pos.parse_pgn_move(s).unwrap();
    assert_eq!(s, m.to_pgn().as_str());
    assert!(pos.validate_move(&m));
    pos.do_move(&m);
    assert!(pos.is_legal());
  }
}

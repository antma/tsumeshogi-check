use tsumeshogi_check::shogi_rules::Position;

#[test]
fn legal_position() {
  for sfen in vec!["k8/+P8/1K7/9/9/9/9/9/9 w 2r2b4g4s4n4l17p 1"] {
    let pos = Position::parse_sfen(&sfen).unwrap();
    assert!(pos.is_legal());
  }
  let mut pos = Position::parse_sfen("k8/9/PK7/9/9/9/9/9/9 b 2r2b4g4s4n4l17p 1").unwrap();
  assert!(pos.do_san_move("P9b+"));
  assert!(pos.is_legal());
}

#[test]
fn moves_generation() {
  for (sfen, ans) in vec![
    (
      "k8/9/PK7/9/9/9/9/9/9 b 2r2b4g4s4n4l17p 1",
      vec!["K7b", "K7c", "K7d", "K8d", "K9d", "P9b+", "P9b="],
    ),
    ("k8/+P8/1K7/9/9/9/9/9/9 w 2r2b4g4s4n4l17p 1", vec![]),
  ] {
    let mut pos = Position::parse_sfen(&sfen).unwrap();
    let checks = pos.compute_checks();
    let moves = pos.compute_moves(&checks);
    let drops = pos.compute_drops(&checks);
    println!("moves = {:?}", moves);
    println!("drops = {:?}", drops);
    let mut res = Vec::new();
    for m in moves.iter().chain(drops.iter()) {
      let u = pos.do_move(&m);
      let legal = pos.is_legal();
      pos.undo_move(&m, &u);
      let san = pos.move_to_string(&m, &moves);
      println!("move {}, {:#?}", san, m);
      if legal {
        res.push(san);
      }
    }
    res.sort();
    assert_eq!(ans, res);
  }
}

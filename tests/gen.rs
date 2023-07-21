use shogi::{moves::Move, Position};
use tsumeshogi_check::shogi;

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
    (
      "l5Gll/6SGk/2n1+N3p/p1p3ppL/7G1/P1P2P3/4SS3/2+p4K1/+rP2P1R2 w 2BGS2N5P2p 2",
      vec!["Kx2b", "Lx2b"],
    ),
  ] {
    let mut pos = Position::parse_sfen(&sfen).unwrap();
    let checks = pos.compute_checks();
    let moves = pos.compute_moves(&checks);
    let drops = pos.compute_drops(&checks);
    let mut res = Vec::new();
    for m in moves.iter().chain(drops.iter()) {
      let u = pos.do_move(&m);
      let legal = pos.is_legal();
      pos.undo_move(&m, &u);
      let san = pos.move_to_string(&m, &moves);
      let packed_move: u32 = u32::from(m.clone());
      assert_eq!(*m, Move::from(packed_move));
      if legal {
        res.push(san);
      }
    }
    res.sort();
    assert_eq!(ans, res);
  }
}

#[test]
fn reorder_takes_to_front() {
  let pos = Position::parse_sfen("k8/9/9/4p4/9/3N5/9/B8/K3R4 b - 1").unwrap();
  let checks = pos.compute_checks();
  let mut moves = pos.compute_moves(&checks);
  assert_eq!(pos.reorder_takes_to_front(&mut moves), 3);
  assert!(moves.iter().take(3).all(|m| pos.is_take(&m)));
  assert!(moves.iter().skip(3).all(|m| !pos.is_take(&m)));
}

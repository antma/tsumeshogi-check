use tsumeshogi_check::shogi_rules::Position;

#[test]
fn double_check() {
  let pos = Position::parse_sfen("9/9/9/4k4/9/4R4/9/B8/9 w - 1").unwrap();
  assert_eq!(pos.is_double_check(), true);
}

#[test]
fn knight_check() {
  let pos = Position::parse_sfen("2n6/9/1K7/9/9/9/9/9/9 b - 1").unwrap();
  let c = pos.compute_checks();
  assert_eq!(c.attacking_pieces, vec![6]);
  let pos = Position::parse_sfen("n8/9/1K7/9/9/9/9/9/9 b - 1").unwrap();
  let c = pos.compute_checks();
  assert_eq!(c.attacking_pieces, vec![8]);
}

#[test]
fn no_check() {
  let pos = Position::parse_sfen("1l1b5/nbr6/pKs6/+nl+p6/9/9/9/9/9 b - 1").unwrap();
  assert_eq!(pos.is_check(), false);
}

#[test]
fn lance_check() {
  let pos = Position::parse_sfen("k8/9/9/9/9/9/9/9/L8 w - 1").unwrap();
  let c = pos.compute_checks();
  assert_eq!(c.attacking_pieces, vec![80]);
  let pos = Position::parse_sfen("k8/p8/9/9/9/9/9/9/L8 w - 1").unwrap();
  assert_eq!(pos.is_check(), false);
}

#[test]
fn tokin_check() {
  let pos = Position::parse_sfen("9/9/1K7/1+p7/9/9/9/9/9 b - 1").unwrap();
  let c = pos.compute_checks();
  assert_eq!(c.attacking_pieces, vec![3 * 9 + 7]);
  let pos = Position::parse_sfen("9/9/1K7/2+p6/9/9/9/9/9 b - 1").unwrap();
  assert_eq!(pos.is_check(), false);
}

#[test]
fn sfen_nifu() {
  let pos = Position::parse_sfen("k8/p8/p8/9/9/9/9/9/L8 w - 1");
  assert_eq!(pos.is_ok(), false);
}

#[test]
fn test_position_is_unblockable_check_false() {
  for (test, sfen) in vec![
    "8k/9/9/9/9/9/9/9/8L w p 1",
    "8k/r8/7K1/9/9/9/9/9/8L w g 1",
    "8k/9/7K1/9/9/9/9/+r8/8L w - 1",
  ].into_iter().enumerate() {
    let pos = Position::parse_sfen(&sfen).unwrap();
    let c = pos.compute_checks();
    assert_eq!(pos.is_unblockable_check(&c), false, "test #{}, sfen: {}", test + 1, sfen);
  }
}

#[test]
fn test_position_is_unblockable_check_true() {
  for sfen in vec![
    "8k/9/7K1/9/9/9/9/9/8L w - 1",
    "8k/9/7N1/9/9/9/9/+r8/8L w - 1",
    "k8/+P8/9/9/9/9/9/9/9 w 2r2b4g4s4n4l17p 1",
    "k8/R8/9/9/9/9/9/9/9 w 2r2b4g4s4n4l17p 1",
  ] {
    let pos = Position::parse_sfen(&sfen).unwrap();
    let c = pos.compute_checks();
    assert_eq!(pos.is_unblockable_check(&c), true);
  }
}

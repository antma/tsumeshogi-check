use tsumeshogi_check::shogi_rules::Position;

#[test]
fn double_check() {
  let pos = Position::parse_sfen("9/9/9/4k4/9/4R4/9/B8/9 w - 1").unwrap();
  let c = pos.find_checks();
  assert_eq!(c.attacking_pieces.len(), 2);
  assert_eq!(c.blocking_cells, 0);
}

#[test]
fn knight_check() {
  let pos = Position::parse_sfen("2n6/9/1K7/9/9/9/9/9/9 b - 1").unwrap();
  let c = pos.find_checks();
  assert_eq!(c.attacking_pieces, vec![6]);
  let pos = Position::parse_sfen("n8/9/1K7/9/9/9/9/9/9 b - 1").unwrap();
  let c = pos.find_checks();
  assert_eq!(c.attacking_pieces, vec![8]);
}


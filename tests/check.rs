use shogi::{AttackingPiecesVec, Position};
use tsumeshogi_check::shogi;

#[test]
fn double_check() {
  let pos = Position::parse_sfen("9/9/9/4k4/9/4R4/9/B8/9 w - 1").unwrap();
  assert_eq!(pos.is_double_check(), true);
}

#[test]
fn knight_check() {
  let pos = Position::parse_sfen("2n6/9/1K7/9/9/9/9/9/9 b - 1").unwrap();
  let c = pos.compute_checks();
  assert_eq!(c.attacking_pieces, AttackingPiecesVec::once(6));
  let pos = Position::parse_sfen("n8/9/1K7/9/9/9/9/9/9 b - 1").unwrap();
  let c = pos.compute_checks();
  assert_eq!(c.attacking_pieces, AttackingPiecesVec::once(8));
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
  assert_eq!(c.attacking_pieces, AttackingPiecesVec::once(80));
  let pos = Position::parse_sfen("k8/p8/9/9/9/9/9/9/L8 w - 1").unwrap();
  assert_eq!(pos.is_check(), false);
}

#[test]
fn tokin_check() {
  let pos = Position::parse_sfen("9/9/1K7/1+p7/9/9/9/9/9 b - 1").unwrap();
  let c = pos.compute_checks();
  assert_eq!(c.attacking_pieces, AttackingPiecesVec::once(34));
  let pos = Position::parse_sfen("9/9/1K7/2+p6/9/9/9/9/9 b - 1").unwrap();
  assert_eq!(pos.is_check(), false);
}

#[test]
fn dragon_check() {
  let pos = Position::parse_sfen("6Snl/6+R1k/6ppp/9/9/9/9/9/9 w Gr2b3g3s3n3l15p 6").unwrap();
  assert!(pos.is_check());
}

#[test]
fn sfen_nifu() {
  let pos = Position::parse_sfen("k8/p8/p8/9/9/9/9/9/L8 w - 1");
  assert_eq!(pos.is_ok(), false);
}

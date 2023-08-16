use tsumeshogi_check::shogi::Position;

#[test]
fn unpromoted_pieces_in_parse_sfen() {
  let p =
    Position::parse_sfen("lnsN1ksnl/1r2g2b1/pp1p3pp/2p2pp2/9/6P2/PPPPPP1PP/1B5R1/LNSGKGS1L b - 1");
  assert!(p.is_err(), "unpromoted black knight on rank 1");
  let p = Position::parse_sfen("lnsgkgsn1/1r5b1/pppppppp1/9/9/7L1/PPPPPPPP1/1B5R1/LNSGKGSNl b - 1");
  assert!(p.is_err(), "unpromoted white lance on rank 9");
  let p = Position::parse_sfen(
    "lnsgkgs1l/1r5b1/pppppp1pp/6p2/5n3/P5P2/1PPPPP1PP/1B2n2R1/LNSGKGS1L b - 1",
  );
  assert!(p.is_err(), "unpromoted white knight on rank 8");
  let p = Position::parse_sfen("lnsgkgsnl/1r5b1/Pppppppp1/9/9/9/1PPPPPPP1/1B5R1/LNSGKGSNp b - 1");
  assert!(p.is_err(), "unpromoted white pawn on rank 9");
}

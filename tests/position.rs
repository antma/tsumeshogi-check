use tsumeshogi_check::shogi::Position;

#[test]
fn unpromoted_pieces_in_parse_sfen() {
  for (sfen, err) in vec![
    (
      "lnsN1ksnl/1r2g2b1/pp1p3pp/2p2pp2/9/6P2/PPPPPP1PP/1B5R1/LNSGKGS1L b - 1",
      "unpromoted black knight on rank 1",
    ),
    (
      "lnsgkgsn1/1r5b1/pppppppp1/9/9/7L1/PPPPPPPP1/1B5R1/LNSGKGSNl b - 1",
      "unpromoted white lance on rank 9",
    ),
    (
      "lnsgkgs1l/1r5b1/pppppp1pp/6p2/5n3/P5P2/1PPPPP1PP/1B2n2R1/LNSGKGS1L b - 1",
      "unpromoted white knight on rank 8",
    ),
    (
      "lnsgkgsnl/1r5b1/Pppppppp1/9/9/9/1PPPPPPP1/1B5R1/LNSGKGSNp b - 1",
      "unpromoted white pawn on rank 9",
    ),
  ] {
    let p = Position::parse_sfen(sfen);
    let e = p.err().unwrap();
    assert!(
      e.message.starts_with("unpromoted"),
      "test {}, error: {:?}",
      err,
      e
    );
  }
}

use moves::{Move, PSNMove};
use shogi::{moves, Position};
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
  let i = pos.reorder_takes_to_front(&mut moves);
  assert_eq!(i, 3, "{:?}", printable_moves(&pos, &moves));
  assert!(moves.iter().take(3).all(|m| pos.is_take(&m)));
  assert!(moves.iter().skip(3).all(|m| !pos.is_take(&m)));
}

fn printable_moves(pos: &Position, a: &Vec<Move>) -> Vec<String> {
  let mut b = Vec::new();
  for m in a {
    b.push(PSNMove::new(&pos, m).to_string());
  }
  b
}

fn filter_checks(pos: &mut Position, a: Vec<Move>) -> Vec<String> {
  let mut b = Vec::new();
  for m in a {
    let u = pos.do_move(&m);
    if pos.is_legal() && pos.is_check() {
      let psn = PSNMove::from_undo(&m, &u);
      b.push(psn.to_string());
    }
    pos.undo_move(&m, &u);
  }
  b.sort();
  b
}

#[test]
fn potentional_checks() {
  for (test, fen) in vec![
    "l3k1snl/1+R2gg1b1/p1Npppppp/9/1pp6/PP7/3P+bsP1P/9/LNSGK3L b PNS3pgr 1",
    "l+R1R3nl/2p1ksgb1/4pp3/p1Ps2ppp/9/P8/2+s+pPPP1P/1P3K3/L1G2GSNL b P2p2ngb 1",
    "l2pk2nl/6Sb1/n1+R2Gsg1/p4pppp/P2S2nb1/1PPs1P1KP/3gP1P2/1+p+r3G2/LN6L b 2P2p 1",
    "l1g2SRnl/4S1g1k/p2+Ppp1Pp/3ps1pp1/+r7P/4PBPS1/3P1P3/3G5/2P1K2NL b PLB2p2ng 1",
    "lB1+R4l/7s1/p1s4s1/2p1pn2p/5k1G1/P3P1n1P/2PP1PGR1/3G3bK/g6LL b 5PNS3pn 1",
    "l2+R4+P/1+B3g3/3Bks2n/p1pppppSp/3n3p1/2s2PP1P/P2PP4/1+n2KG1R1/L5sNL b 2PLGpg 1",
    "l2+R3ll/2p+R5/p4g2+P/2kpp4/2Bn1p1n1/1Ps1P1PS1/P4P2b/3S4L/1NG2G1NK b 6PSGp 1",
    "l8/9/p1+Rp1+B3/6S2/S2Pp1sNp/4PskpP/P1+B2g+p2/2K6/L1G2N1LL b 5P2NGR3pg 1",
    "l1Sg5/1+N4B2/7+Rl/p1P1p2p1/5Ns1k/4P1S+bp/PP1Ps2G1/4+p1G1K/L8 b 5PL2NG3pr 1",
    "lnB5l/k1g+R2np1/pl1p1B1+Pn/2ss2p2/N1p2pP2/K1P1P2GP/PS1P2G2/1P4sg1/+r7L b 2P2p 1",
    "l3k1snl/1+R2gg1b1/p1Npppppp/9/1pp6/PP7/3P+bsP1P/9/LNSGK3L b PNS3pgr 1",
    "l+R1R3nl/2p1ksgb1/4pp3/p1Ps2ppp/9/P8/2+s+pPPP1P/1P3K3/L1G2GSNL b P2p2ngb 1",
    "l2pk2nl/6Sb1/n1+R2Gsg1/p4pppp/P2S2nb1/1PPs1P1KP/3gP1P2/1+p+r3G2/LN6L b 2P2p 1",
    "l1g2SRnl/4S1g1k/p2+Ppp1Pp/3ps1pp1/+r7P/4PBPS1/3P1P3/3G5/2P1K2NL b PLB2p2ng 1",
    "lB1+R4l/7s1/p1s4s1/2p1pn2p/5k1G1/P3P1n1P/2PP1PGR1/3G3bK/g6LL b 5PNS3pn 1",
    "l2+R4+P/1+B3g3/3Bks2n/p1pppppSp/3n3p1/2s2PP1P/P2PP4/1+n2KG1R1/L5sNL b 2PLGpg 1",
    "l2+R3ll/2p+R5/p4g2+P/2kpp4/2Bn1p1n1/1Ps1P1PS1/P4P2b/3S4L/1NG2G1NK b 6PSGp 1",
    "l8/9/p1+Rp1+B3/6S2/S2Pp1sNp/4PskpP/P1+B2g+p2/2K6/L1G2N1LL b 5P2NGR3pg 1",
    "l1Sg5/1+N4B2/7+Rl/p1P1p2p1/5Ns1k/4P1S+bp/PP1Ps2G1/4+p1G1K/L8 b 5PL2NG3pr 1",
    "lnB5l/k1g+R2np1/pl1p1B1+Pn/2ss2p2/N1p2pP2/K1P1P2GP/PS1P2G2/1P4sg1/+r7L b 2P2p 1",
    "l2p2k1l/2+R1+Sp1g1/2B1p1n+Bp/p1p1gs3/1p1n5/P3P2p1/K7P/2G2S3/LN2G2NL b 2PS5pr 1",
    "9/2+S6/4k1n+R1/3sppS2/1+B4p2/2PpPPPP1/PP1P1+n3/3G1+r3/KNL+nb4 b 6PS3l3g 1",
    "lnS1+P3l/k1r+R5/pp2p2pp/2p2pP2/1N5n1/1PPP5/PG6P/1K5+s1/L4G2L b 3P2S2G2Bpn 1",
    "l5+R2/4g1gS1/p2+Lp2p1/2pp1pkg1/3PP2np/1PP2Pp2/P1N1G2+sP/2KS3+r1/L3N4 b PLN2B2ps 1",
    "l3+BS2k/6S2/5pnG1/P1ppp1ppl/8p/5bg2/1PPP4P/2G2S1R1/LN1K1s1NL b 6pngr 1",
    "l1p2S1nl/2G1+R4/2S3spk/p1P1ppp2/3K3Gp/3B1P1P1/P1+p1P1P2/5S1R1/L6NL b 2PB2p2n2g 1",
    "ln1B+R4/1k5p1/p1ppS1+L2/1ps1ppp2/9/5P2p/PPNPP1P2/3S5/L2GKGSNL b 2PNBp2gr 1",
    "lnG3+P+Ll/2B1g2+R1/p1pp1s3/4pp3/5k3/2P2NP2/P+p1PPP3/2S1K4/+rNSG1G1N1 b 4Pplsb 1",
    "1+Rs1k2nl/4gsg2/1p1Lp2pp/1Ppp1pP2/p3B2P1/2g1P3P/1+s1P1P3/2s2K3/1NN4rL b 3PLNGb 1",
    "3+R2s1l/1pPs+B2b1/k1s1pp3/7gP/1SN2Gpn1/3P5/1P1KPP1+p1/1R+l6/2gg3+lL b 2P2N5p 1",
    "+N7r/5+Rg1k/p4LB2/5p1Gp/2P2PL2/P1p1P1SpP/1P1P1+nP2/2+b1+ls2K/7NL b 4PGpn2sg 1",
    "1k1g1+R2l/9/P2p3g1/S1Nn1B1pp/2s1G1P2/SnpPp4/NK5+lP/3S5/4B3L b 5PR4plg 1",
    "l7l/1S1g3S1/3p1G+Rp1/2pbpP2p/3n2p1k/p1PBP1PP1/1+p1P1G2P/2S6/L1K3R1L b P2N2pnsg 1",
    "l6Sl/9/6+Rsk/p1png1pp1/1p1N1p2b/P1SS4R/1P1P1G3/5K3/Lb6L b P2NG8pg 1",
    "ln1gk1b1+R/5p1P1/3p+SbN2/p5pNp/4p4/1PpPP1P2/P1NGS1g1P/3K5/L7L b 3PSplsgr 1",
    "ln4+R2/5+S3/p1p1p+B1pk/1G4p2/2N1bl1Gp/1PP2+s2L/P2P1+n1SP/2+p2pPPK/1r3GG1L b PNS2p 1",
    "l4g2l/4k1s2/p+R+B1pp1p1/4r2bp/8P/PPP1PSpP1/1G1PKP1s1/9/LN4GNL b 2P2NG2ps 1",
    "6Snl/5+Rg1k/4p2np/p2P1p+Bp1/4P2PP/PP1K1SP2/1+nN2P3/6g2/LR5+s1 b 4P2LSGpgb 1",
    "5Gn2/k1r1+R4/1LB6/lG1ppP2p/1PPn1n3/b1G1P4/NS1P4P/2KS1S3/L3G3L b 9Ps 1",
    "ln1B3n1/4+R4/3p5/1kpPp3l/p2G2pp1/1g+s5p/P3K1N2/9/+bN6L b 3P2G6pl3sr 1",
    "1bk2+SR1l/2g4+R1/G1p1pp3/2nL2p1p/L1G1P4/p4P2P/1P+s+pS1Pb1/6S2/1+p2GK2L b 3P3N2p 1",
    "l7l/1+S3+Rgb1/2n2S2k/p1p1p1PR1/3p1p1pK/PpPPPs3/1bNS2p2/4G4/L2g3NL b 3PN2pg 1",
    "ls5nl/1p1G5/2B6/ps1p2p1+P/k8/2GP2P2/3SPPK1P/G1+l6/L4G2+p b 3P3NS4pb2r 1",
    "l4kg1l/3gR4/2n2BnSp/2BPpP1n1/P1p3K2/2P1Pp3/1P1S1g2P/1G7/LN5+RL b 6P2Sp 1",
    "4g1s+B1/1p1s+N1g2/3pp+L1pp/1+L4p2/1Pp1N2Sk/2P2nPL1/3+B1P2P/+r2G3g1/2+p1K1RNL b 5PS 1",
    "4g1s+B1/1p1s+N1g2/3pp+L1pp/1+L4p2/1Pp1N2Sk/2P2nPL1/5P2P/+r2G3g1/2+p1K1RNL b S5P 1",
  ]
  .into_iter()
  .enumerate()
  {
    let mut pos = Position::parse_sfen(fen).unwrap();
    let checks = pos.compute_checks();
    let l1 = pos.compute_check_candidates(&checks);
    let l2 = pos.compute_moves(&checks);
    let r1 = filter_checks(&mut pos, l1);
    let r2 = filter_checks(&mut pos, l2);
    assert_eq!(r1, r2, "test {}, fen {}", test + 1, fen);
    pos.swap_sides();
    let checks = pos.compute_checks();
    let l1 = pos.compute_check_candidates(&checks);
    let l2 = pos.compute_moves(&checks);
    let r1 = filter_checks(&mut pos, l1);
    let r2 = filter_checks(&mut pos, l2);
    assert_eq!(r1, r2, "test(swapped sides) {}, fen {}", test + 1, pos);
  }
}

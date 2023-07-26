use tsumeshogi_check::search;
use tsumeshogi_check::shogi::Position;
mod common;

#[test]
fn tsume3() {
  common::tsume_batch_test(
    vec![
      "k2G5/9/K8/9/N8/9/9/9/9 b 2r2b3g4s3n4l18p 1",
      "3sks3/9/4S4/9/9/8B/9/9/9 b S 1",
      "9/4k4/9/4P4/9/9/9/9/9 b 2G2r2b4s4n4l17p 1",
      "9/6k2/9/5G3/4G4/4P4/9/9/9 b 2G2r2b4s4n4l17p 1",
      "9/4k4/9/5G3/4G4/4P4/9/9/9 b 2G2r2b4s4n4l17p 1",
      "9/5k3/9/5G3/4G4/4P4/9/9/9 b 2G2r2b4s4n4l17p 1",
      "7nl/7k1/6Ppp/9/9/9/9/9/9 b 2G2r2b2g4s3n3l15p 1",
      "7nl/7k1/5+Pppp/9/9/9/9/9/9 b GS2r2b2g3s3n3l14p 1",
      "5g1nl/4g1sk1/4ppppp/9/4B4/9/9/9/9 b GN2rbg3s2n3l13p 1",
      "5k3/7gR/4S4/7L1/9/9/9/9/9 b Sr2b3g2s4n3l18p 1",
      "8l/6nkg/5PpBp/5+r3/7R1/9/9/9/9 b b3g4s3n3l15p 1",
      "9/9/9/9/9/7np/8k/8B/7LR b rb4g4s3n3l17p 1",
      "ln1gkg1nl/6+P2/2sppps1p/2p3p2/p8/P1P1P3P/2NP1PP2/3s1KSR1/L1+b2G1NL w R2Pbgp 42",
      "4kp+R2/1+B4n2/4p1Sp1/2pps1l1g/1p3PR2/2P6/1GNPP+pN2/3S5/1+lBKG4 b GNL4Psl3p 105",
      "6+R2/1+B1k2n2/4p1Sp1/2pps1l1g/1p3PR2/2P6/1GNPP+pN2/3S5/1+lBKG4 b GNL4Psl4p 105",
      "6+R2/1+B2k1n2/4p1Sp1/2pps1l1g/1p3PR2/2P6/1GNPP+pN2/3S5/1+lBKG4 b GNL4Psl4p 105",
      "4kl+R2/1+B4n2/4p1Sp1/2pps1l1g/1p3PR2/2P6/1GNPP+pN2/3S5/1+lBKG4 b GNL4Ps4p 105",
      "4k4/l1+P2pG2/7p1/p2p1lp2/1ps2P1P1/PP1PP1P2/2pGBSN2/1s1G1K2s/L6R1 b 2NL3Prbgn 3",
      "4+R4/l1+P2pG2/5k1p1/p2p1lp2/1ps2P1P1/PP1PP1P2/2pGBSN2/1s1G1K2s/L6R1 b 2NL3Pbgn 3",
      "lg3+B1nl/3s5/p1n1sSk2/7pp/9/PKPpP4/1P2gPP1P/1s1r5/LN5NL b G6Prbgp 3",
      "lg3+B1nl/3s2p2/p1n1sS1k1/7pp/9/PKPpP4/1P2gPP1P/1s1r5/LN5NL b G6Prbg 3",
    ],
    3,
  );
}

#[test]
fn no_tsume3() {
  common::no_tsume_batch_test(
    vec![
      "9/4k4/9/4P4/9/9/9/9/9 b G2r2b4s4n4l17p 1",
      "k8/9/9/1+R7/9/9/9/9/P8 b LPr2b4g4s4n3l16p 1",
    ],
    3,
  );
}

#[test]
fn swap_sides() {
  let mut pos = Position::parse_sfen(
    "ln1gkg1nl/6+P2/2sppps1p/2p3p2/p8/P1P1P3P/2NP1PP2/3s1KSR1/L1+b2G1NL w R2Pbgp 42",
  )
  .unwrap();
  pos.swap_sides();
  let mut a = Vec::new();
  let s = pos.to_string();
  a.push(s.as_str());
  common::tsume_batch_test(a, 3);
}

#[test]
fn unique_mate() {
  let mut pos =
    Position::parse_sfen("+N7l/9/2GSppS2/p5p+Rp/7l1/1PP1k4/P1SP2NP1/2GK2S2/LN1B1G3 b RN3Pbgl5p 1")
      .unwrap();
  let mut s = search::Search::default();
  let ans = s.search(&mut pos, 3);
  assert_eq!(ans.0, Some(3));
  assert!(ans.1.is_some());
}

//futile drops
//9/2B+Pg4/2gp5/5p3/R2LkN3/2NP5/2G1PP3/9/9 b rbg4s2n3l14p 1

mod common;

#[test]
fn tsume7() {
  common::tsume_batch_test(
    vec![
      "9/9/9/4k4/9/4P4/9/9/9 b 4G2r2b4s4n4l17p 1",
      "+R4G1nl/6k2/4ppppp/9/9/9/9/9/9 b r2b3g4s3n3l13p 1",
      "3nknB2/2G2g1G1/3p1ps2/7N1/9/9/9/9/9 b L2rbg3sn3l16p 1",
      "ln7/1k7/1pp1S4/p8/9/9/9/9/9 b RBGNrb3g3s2n3l15p 1",
      "l8/1ks+P5/1ppp5/p8/9/2P6/9/9/9 b G2SN2r2b3gs3n3l12p 1",
      "7+R1/l1+P1kpG2/7p1/p2p1lp2/1ps2P1P1/PP1PP1P2/2pGBSN2/1s1G1K2s/L6R1 b 2NL3Pbgn 1",
      "ln1r4l/3k1gp2/pS1p1+P1+R1/6L1p/1pPP2S2/2S3P2/PP2p1+b1P/3K5/+b7L w 2GSN3Pg2n2p 118",
      "+B7l/3k5/pp2P1+B1p/2s3p2/1Ppp2s2/6l1P/Ps1P1+p1+r1/3K1GP2/LN1R4L b 2P2NG3pns2g 1",
    ],
    7,
  );
}

#[test]
fn with_futil_drop() {
  common::tsume_batch_test_ext(vec!["4r2nk/6+Pbl/9/6L2/9/9/9/9/9 b B 1"], 7, Some(7), true);
}

mod common;

#[test]
fn tsume5() {
  common::tsume_batch_test(
    vec![
      "9/9/5k3/9/4G4/4P4/9/9/9 b 3G2r2b4s4n4l17p 1",
      "6knl/6s2/5Pppp/9/9/9/9/9/9 b G2S2r2b3gs3n3l14p 1",
      "7nl/7k1/6ppp/9/9/9/9/9/5R3 b Sr2b4g3s3n3l15p 1",
      "7nl/5+R1gk/6Ppp/9/9/9/9/9/9 b Gr2b2g4s3n3l15p 1",
      "4R2nl/6sk1/6pp1/8p/9/9/9/9/9 b BGLrb3g3s3n2l15p 1",
      "l6nl/3+R1pk2/3p3gp/p1p3p2/1p1nsN3/2P3P1P/PP1P1S3/1SG2GK2/LN6L b BGPrbs4p 1",
      "3sr4/4p4/2Sks1n2/3p1P3/2L3N2/4P1P2/4N4/9/9 b BNb3gs3l13p 1",
      "l2k5/9/3S2S2/9/9/9/9/9/9 b B2N2rb4g2s2n3l18p 1",
      "9/9/3S5/3p1p3/4kb3/3P2s2/4P4/4N4/9 b RNrb3g2s2n4l14p 1",
      "7nl/7k1/9/9/7BL/5+r1R1/9/9/9 b 2Gb2g4s3n2l18p 1",
      "7r1/7k1/4r4/5N+Bp1/8p/9/9/9/9 b G2Nb3g4sn4l16p 1",
    ],
    5,
  );
}

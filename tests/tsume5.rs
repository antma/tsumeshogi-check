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
    ],
    5,
  );
}
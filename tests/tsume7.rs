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
    ],
    7,
  );
}

#[test]
fn with_futil_drop() {
  common::tsume_batch_test_ext(
    vec!["4r2nk/6+Pbl/9/6L2/9/9/9/9/9 b B 1"],
    7,
    Some(7),
    true);
}

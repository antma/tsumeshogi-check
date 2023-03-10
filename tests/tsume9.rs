mod common;

#[test]
fn tsume9() {
  common::tsume_batch_test(
    vec![
      "ln7/1k1+P5/1pp1S4/p8/9/2P6/9/9/9 b BGN2rbg2s2n3l13p 1",
      "ln1p5/2k1p4/1pgP+P4/p8/1P7/P8/9/9/9 b RG2Sr2b2g2s3n3l10p 1",
      "ln7/1k2s4/1pp6/p8/9/3L5/9/9/9 b RBGNrb3g3s2n2l15p 1",
    ],
    9,
  );
}

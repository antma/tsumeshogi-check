mod common;

#[test]
fn tsume3() {
  common::tsume_batch_test(
    vec![
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

//futile drops
//9/2B+Pg4/2gp5/5p3/R2LkN3/2NP5/2G1PP3/9/9 b rbg4s2n3l14p 1

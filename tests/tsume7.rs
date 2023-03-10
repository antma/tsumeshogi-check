mod common;

#[test]
fn tsume7() {
  common::tsume_batch_test(
    vec![
      "9/9/9/4k4/9/4P4/9/9/9 b 4G2r2b4s4n4l17p 1",
      "+R4G1nl/6k2/4ppppp/9/9/9/9/9/9 b r2b3g4s3n3l13p 1",
    ],
    7,
  );
}

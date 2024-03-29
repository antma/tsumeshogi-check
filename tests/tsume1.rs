use tsumeshogi_check::search;
use tsumeshogi_check::shogi::Position;
mod common;

#[test]
fn tsume1() {
  common::tsume_batch_test(
    vec![
      "k8/9/PK7/9/9/9/9/9/9 b 2r2b4g4s4n4l17p 1",
      "3k5/9/3S5/9/9/9/9/9/9 b S2r2b4g2s4n4l18p 1",
      "k8/9/1K7/9/9/9/8B/8B/9 b 2r4g4s4n4l18p 1",
      "kb7/p8/9/9/9/9/8B/9/8K b 2R4G4S4N4L17P 1",
      "7pk/9/7K1/5+r3/9/9/9/1B7/R8 b b4g4s4n4l17p 1",
      "4k4/9/4P4/9/9/9/9/9/9 b G2r2b3g4s4n4l17p 1",
      "4k4/9/4S4/9/9/9/9/9/9 b S2r2b4g2s4n4l18p 1",
      "kn7/ps7/9/9/N8/9/9/9/8B b 2r1b4g2s2n4l17p 1",
      "9/2B6/3G2r2/3pkL1R1/3gsg3/3S2B2/3g5/4N4/4L4 b 2s3n2l17p 1",
    ],
    1,
  );
}

#[test]
fn tsume1_nifu() {
  common::tsume_batch_test(vec!["k8/2K6/9/7BB/7LL/7LL/8R/p2NNSSGG/P2NNSSGG b 16p 1"], 1);
}

#[test]
fn tsume1_drop_white_knight_on_first_rank_is_illegal() {
  common::tsume_batch_test(
    vec!["l1+L6/9/p1pL3Gp/2s+Bp1pp1/2Np3P1/P1P6/1PKP1SB1P/2Gg+nr3/L4k3 b RG2SN5Pn 179"],
    1,
  );
}

#[test]
fn tsume1_futile_drops() {
  common::tsume_batch_test(
    vec![
      "6Snl/5+Rg1k/6ppp/9/9/9/9/9/9 b r2b3g3s3n3l15p 1",
      "6Snl/5+Rg1k/6pps/9/9/9/9/9/8L b r2b3g2s3n2l16p 1",
      "6Snl/5+Rg1k/6pgs/9/9/9/9/9/1B6L b rb2g2s3n2l17p 1",
      "lS5nl/3s2+R2/p6p1/kS2Gb2p/3ppNpS1/Np3P2P/PPG1P1K2/4G4/LB5RL b GN4P2p 1",
    ],
    1,
  );
  common::no_tsume_batch_test(
    vec!["k3l3R/p4s3/9/N8/4p4/4KBB2/9/1pp+l+l+l+n+n+n/+s+s+sggggr1 b 14p 1"],
    1,
  );
}

#[test]
fn no_tsume1() {
  common::no_tsume_batch_test(vec!["9/6sG1/8k/5B2p/7L1/9/9/9/9 b 2rb3g3s4n3l17p 1"], 1);
}

#[test]
fn pawn_drop_no_mate() {
  common::no_tsume_batch_test(vec!["kn7/1s7/9/1N7/9/9/9/9/9 b P2r2b4g3s2n4l17p 1"], 1);
}

#[test]
fn not_unique_mate() {
  let mut pos = Position::parse_sfen("k8/9/K8/9/9/9/9/9/9 b G2r2b3g4s4n4l18p 1").unwrap();
  let mut s = search::Search::new(1 << 20);
  let ans = s.search(&mut pos, 1);
  assert_eq!(ans.0, Some(1));
  assert!(ans.1.is_none());
}

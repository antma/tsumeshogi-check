use tsumeshogi_check::shogi::Position;

fn rec(pos: &mut Position, depth: usize) -> u32 {
  if depth == 0 {
    return 1;
  }
  let mut r = 0;
  let checks = pos.compute_checks();
  let moves = pos.compute_moves(&checks);
  let drops = pos.compute_drops(&checks);
  for m in moves.into_iter().chain(drops.into_iter()) {
    let u = pos.do_move(&m);
    if pos.is_legal() {
      r += rec(pos, depth - 1);
    }
    pos.undo_move(&m, &u);
  }
  r
}

#[test]
fn perft() {
  let mut pos = Position::default();
  assert_eq!(rec(&mut pos, 4), 719731);
}

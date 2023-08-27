use tsumeshogi_check::shogi::{alloc::PositionMovesAllocator, Position};

fn rec(pos: &mut Position, allocator: &mut PositionMovesAllocator, depth: usize) -> u32 {
  if depth == 0 {
    return 1;
  }
  let mut r = 0;
  let checks = pos.compute_checks();
  let moves = pos.compute_moves(&checks);
  let drops = pos.compute_drops(allocator, &checks);
  for m in moves.into_iter().chain(drops.into_iter()) {
    let u = pos.do_move(&m);
    if pos.is_legal() {
      r += rec(pos, allocator, depth - 1);
    }
    pos.undo_move(&m, &u);
  }
  r
}

#[test]
fn perft() {
  let mut pos = Position::default();
  let mut allocator = PositionMovesAllocator::default();
  assert_eq!(rec(&mut pos, &mut allocator, 4), 719731);
}

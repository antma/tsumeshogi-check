use super::moves::Move;

#[derive(Clone, Debug)]
pub struct MovesAllocator {
  pub total_moves: u64,
  pub total_calls: u64,
}

impl Default for MovesAllocator {
  fn default() -> Self {
    Self {
      total_moves: 0,
      total_calls: 0,
    }
  }
}

impl std::ops::AddAssign for MovesAllocator {
  fn add_assign(&mut self, rhs: Self) {
    self.total_moves += rhs.total_moves;
    self.total_calls += rhs.total_calls;
  }
}

impl MovesAllocator {
  pub(super) fn alloc_vec(&self) -> Vec<Move> {
    if self.total_calls > 0 {
      let l = self.total_moves / self.total_calls;
      Vec::with_capacity(l as usize + 1)
    } else {
      Vec::new()
    }
  }
  pub(super) fn update(&mut self, v: &Vec<Move>) {
    self.total_moves += v.len() as u64;
    self.total_calls += 1;
  }
}

#[derive(Default, Debug)]
pub struct PositionMovesAllocator {
  pub compute_check_candidates_allocator: MovesAllocator,
  pub compute_moves_after_non_blocking_check_allocator: MovesAllocator,
  pub compute_moves_after_sliding_piece_check_allocator: MovesAllocator,
  pub compute_drops_with_checks_allocator: MovesAllocator,
  pub compute_drops_no_pawns_with_checks_allocator: MovesAllocator,
  pub compute_drops_after_sliding_piece_check_allocator: MovesAllocator,
  pub compute_legal_king_moves_allocator: MovesAllocator,
}

impl std::ops::AddAssign for PositionMovesAllocator {
  fn add_assign(&mut self, rhs: Self) {
    self.compute_check_candidates_allocator += rhs.compute_check_candidates_allocator;
    self.compute_moves_after_non_blocking_check_allocator +=
      rhs.compute_moves_after_non_blocking_check_allocator;
    self.compute_moves_after_sliding_piece_check_allocator +=
      rhs.compute_moves_after_sliding_piece_check_allocator;
    self.compute_drops_with_checks_allocator += rhs.compute_drops_with_checks_allocator;
    self.compute_drops_no_pawns_with_checks_allocator +=
      rhs.compute_drops_no_pawns_with_checks_allocator;
    self.compute_drops_after_sliding_piece_check_allocator +=
      rhs.compute_drops_after_sliding_piece_check_allocator;
    self.compute_legal_king_moves_allocator += rhs.compute_legal_king_moves_allocator;
  }
}

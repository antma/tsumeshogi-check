pub fn find_king_position(board: &[i8], s: i8) -> Option<usize> {
  let king = super::piece::KING * s;
  board
    .iter()
    .enumerate()
    .find_map(|(i, v)| if *v == king { Some(i) } else { None })
}

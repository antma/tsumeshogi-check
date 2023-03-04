pub fn does_column_contain_piece(board: &[i8], col: usize, piece: i8) -> bool {
  board.iter().skip(col).step_by(9).any(|q| *q == piece)
}

pub fn does_column_contain_at_least_two_pieces(board: &[i8], col: usize, piece: i8) -> bool {
  board
    .iter()
    .skip(col)
    .step_by(9)
    .filter(|&&q| q == piece)
    .count()
    >= 2
}

pub const JP_COLS: [char; 9] = [
  '１', '２', '３', '４', '５', '６', '７', '８', '９',
];

pub const JP_ROWS: [char; 9] = [
  '一', '二', '三', '四', '五', '六', '七', '八', '九',
];

pub fn push_cell_as_jp_str(s: &mut String, cell: usize) {
  let (row, col) = super::cell::unpack(cell);
  s.push(JP_COLS[col]);
  s.push(JP_ROWS[row]);
}

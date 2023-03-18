use crate::shogi::game::Game;
use crate::shogi::moves::Move;
use crate::shogi::piece;

const JP_COLS: [char; 9] = [
  '１', '２', '３', '４', '５', '６', '７', '８', '９',
];
const JP_ROWS: [char; 9] = [
  '一', '二', '三', '四', '五', '六', '七', '八', '九',
];

pub fn move_to_kif_format(m: &Move, prev_move: &Option<Move>) -> String {
  let mut s = String::with_capacity(8);
  if m.is_drop() {
    s.push(JP_COLS[m.to % 9]);
    s.push(JP_ROWS[m.to / 9]);
    s.push_str(&piece::to_jp_string(m.to_piece.abs()));
  } else {
    let cell = prev_move.as_ref().map(|q| q.to).unwrap_or(0xff);
    if cell == m.to {
      s.push_str("同　");
    } else {
      s.push(JP_COLS[m.to % 9]);
      s.push(JP_ROWS[m.to / 9]);
    }
    if m.to_piece != m.from_piece {
      s.push_str(&piece::to_jp_string(m.from_piece.abs()));
      s.push('成');
    } else {
      s.push_str(&piece::to_jp_string(m.to_piece.abs()));
    }
    s.push('(');
    s.push((49 + (m.from % 9) as u8) as char);
    s.push((49 + (m.from / 9) as u8) as char);
    s.push(')');
  }
  s
}

pub fn game_to_lines(game: &Game) -> Vec<String> {
  let mut a = Vec::with_capacity(game.moves.len() + 10);
  for (jp, en) in vec![
    ("開始日時", "date"),
    ("先手", "black"),
    ("後手", "white"),
  ] {
    if let Some(t) = game.header.get(en) {
      a.push(format!("{}：{}", jp, t));
    }
  }
  a.push(String::from("手数----指手---------消費時間--"));
  a
}

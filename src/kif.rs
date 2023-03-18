use crate::shogi::game::Game;

const JP_COLS: [char; 9] = [
  '１', '２', '３', '４', '５', '６', '７', '８', '９',
];
const JP_ROWS: [char; 9] = [
  '一', '二', '三', '四', '五', '六', '七', '八', '九',
];

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

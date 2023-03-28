use crate::shogi::game::Game;

pub fn game_to_lines(game: &Game) -> Vec<String> {
  let mut a = Vec::with_capacity(game.moves.len() + 10);
  for (jp, en) in vec![
    ("開始日時", "date"),
    ("棋戦", "event"),
    ("先手", "black"),
    ("後手", "white"),
  ] {
    if let Some(t) = game.header.get(en) {
      a.push(format!("{}：{}", jp, t));
    }
  }
  a.push(String::from("手数----指手---------消費時間--"));
  let mut last_move = None;
  for (i, m) in game.moves.iter().enumerate() {
    a.push(format!("{0:>4} {1}", i + 1, m.to_kif(&last_move)));
    last_move = Some(m.clone());
  }
  if let Some(_) = game.header.get("checkmate") {
    a.push(format!("{0:>4} {1}", game.moves.len() + 1, "詰み"));
  }
  if let Some(_) = game.header.get("resignation") {
    a.push(format!("{0:>4} {1}", game.moves.len() + 1, "投了"));
  }
  a
}

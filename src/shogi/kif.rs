use super::game::Game;
use super::piece;
use super::Position;

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

//TODO: empty hand
fn push_pockets_as_jp_str(s: &mut String, pockets: &[u8], side: i8) {
  s.push(if side > 0 { '先' } else { '後' });
  s.push_str("手の持駒：");
  for (f, (i, p)) in pockets
    .iter()
    .enumerate()
    .skip(1)
    .rev()
    .filter(|&(_, &c)| c > 0)
    .enumerate()
  {
    if f > 0 {
      s.push(' ');
    }
    s.push(piece::to_jp_char(i as i8));
    if *p > 1 {
      if *p >= 10 {
        assert!(*p < 20);
        s.push('十');
        s.push(JP_ROWS[(*p % 10) as usize - 1]);
      } else {
        s.push(JP_ROWS[*p as usize - 1]);
      }
    }
  }
  s.push('\n');
}

const BOARD_DELIMETER: &'static str = "+---------------------------+\n";

pub fn position_to_kif(s: &mut String, pos: &Position) {
  push_pockets_as_jp_str(s, &pos.white_pockets, -1);
  s.push_str("  ９ ８ ７ ６ ５ ４ ３ ２ １\n");
  s.push_str(BOARD_DELIMETER);
  for (row, d) in JP_ROWS.iter().enumerate() {
    s.push('|');
    for c in pos.board.iter().skip(9 * row).take(9).rev() {
      if *c == 0 {
        s.push('・');
        continue;
      }
      let abs_piece = if *c < 0 {
        s.push('v');
        -(*c)
      } else {
        s.push(' ');
        *c
      };
      s.push(piece::to_jp_char(abs_piece));
    }
    s.push('|');
    s.push(*d);
    s.push('\n');
  }
  s.push_str(BOARD_DELIMETER);
  push_pockets_as_jp_str(s, &pos.black_pockets, 1);
  if pos.side < 0 {
    s.push_str("後手番\n");
  }
}

pub fn game_to_kif(game: &Game, start_pos: Option<&Position>) -> String {
  let mut s = String::new();
  for (jp, en) in vec![
    ("開始日時", "date"),
    ("棋戦", "event"),
    ("", "sfen"),
    ("先手", "black"),
    ("後手", "white"),
  ] {
    if en == "sfen" {
      if let Some(pos) = start_pos {
        position_to_kif(&mut s, pos);
      }
      continue;
    }
    if let Some(t) = game.header.get(en) {
      s.push_str(&format!("{}：{}\n", jp, t));
    }
  }
  s.push_str("手数----指手---------消費時間--\n");
  let mut last_move = None;
  for (i, m) in game.moves.iter().enumerate() {
    s.push_str(&format!("{0:>4} {1}\n", i + 1, m.to_kif(&last_move)));
    last_move = Some(m.clone());
  }
  if let Some(_) = game.header.get("checkmate") {
    s.push_str(&format!("{0:>4} {1}\n", game.moves.len() + 1, "詰み"));
  }
  if let Some(_) = game.header.get("resignation") {
    s.push_str(&format!("{0:>4} {1}\n", game.moves.len() + 1, "投了"));
  }
  s
}

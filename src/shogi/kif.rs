use super::alloc::PositionMovesAllocator;
use super::game::Game;
use super::piece;
use super::Position;

use super::super::io::FileIterator;
use std::collections::HashMap;

pub const JP_COLS: [char; 9] = ['１', '２', '３', '４', '５', '６', '７', '８', '９'];

pub const JP_ROWS: [char; 9] = ['一', '二', '三', '四', '五', '六', '七', '八', '九'];

pub struct KIFBuilder {
  jp: HashMap<&'static str, &'static str>,
  en: HashMap<&'static str, &'static str>,
  allocator: PositionMovesAllocator,
}

impl Default for KIFBuilder {
  fn default() -> Self {
    let mut jp = HashMap::new();
    let mut en = HashMap::new();
    for (s_jp, s_en) in vec![
      ("開始日時", "date"),
      ("棋戦", "event"),
      ("場所", "location"),
      ("先手", "sente"),
      ("後手", "gote"),
      ("持ち時間", "control"),
      ("手合割", "handicap"),
    ] {
      jp.insert(s_jp, s_en);
      en.insert(s_en, s_jp);
    }
    Self {
      jp,
      en,
      allocator: PositionMovesAllocator::default(),
    }
  }
}

impl KIFBuilder {
  fn jp_to_en(&self, word: &str) -> Option<&'static str> {
    self.jp.get(&word).map(|p| *p)
  }
  fn en_to_jp(&self, word: &str) -> Option<&'static str> {
    self.en.get(&word).map(|p| *p)
  }
}

pub fn push_cell_as_jp_str(s: &mut String, cell: usize) {
  let (row, col) = super::cell::unpack(cell);
  s.push(JP_COLS[col]);
  s.push(JP_ROWS[row]);
}

fn push_pockets_as_jp_str(s: &mut String, pockets: &[u8], side: i8) {
  s.push(if side > 0 { '先' } else { '後' });
  s.push_str("手の持駒：");
  let mut empty_hand = true;
  for (f, (i, p)) in pockets
    .iter()
    .enumerate()
    .skip(1)
    .rev()
    .filter(|&(_, &c)| c > 0)
    .enumerate()
  {
    empty_hand = false;
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
  if empty_hand {
    s.push_str("なし");
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

impl KIFBuilder {
  pub fn game_to_kif(&self, game: &Game, start_pos: Option<&Position>) -> String {
    let mut s = KIF_HEADER_LINE.to_owned();
    s.push('\n');
    for en in vec![
      "date", "event", "location", "sfen", "control", "handicap", "sente", "gote",
    ] {
      if en == "sfen" {
        if let Some(pos) = start_pos {
          position_to_kif(&mut s, pos);
        }
      } else {
        if let Some(t) = game.header.get(en) {
          if let Some(jp) = self.en_to_jp(en) {
            s.push_str(&format!("{}：{}\n", jp, t));
          }
        }
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
}

const KIF_HEADER_LINE: &str = "#KIF version=2.0 encoding=UTF-8";

pub fn kif_file_iterator(filename: &str) -> std::io::Result<FileIterator> {
  FileIterator::new(filename, KIF_HEADER_LINE)
}

fn parse_header<'a>(s: &'a str) -> Option<(&'a str, &'a str)> {
  s.split_once('：')
}

fn parse_move<'a>(s: &'a str, move_no: u32) -> Option<&'a str> {
  let mut it = s.split_ascii_whitespace();
  if let Some(s) = it.next() {
    if s != move_no.to_string() {
      None
    } else {
      it.next()
    }
  } else {
    None
  }
}

#[derive(Debug)]
pub struct ParseKIFGameError {
  pub msg: String,
  pub line: String,
}

impl ParseKIFGameError {
  fn new(line: String, msg: String) -> Self {
    Self { msg, line }
  }
}

impl KIFBuilder {
  pub fn parse_kif_game(
    &mut self,
    a: &Vec<String>,
  ) -> std::result::Result<Game, ParseKIFGameError> {
    let mut g = Game::default();
    let mut st = 0;
    let mut pos = Position::default();
    let mut last_move = None;
    for s in a {
      log::debug!("st = {}, process line {}", st, s);
      if st == 0 {
        if s == "手数----指手---------消費時間--" {
          st += 1;
          continue;
        }
        if let Some((key, value)) = parse_header(&s) {
          if let Some(en) = self.jp_to_en(key) {
            g.set_header(en.to_owned(), value.to_owned());
          }
        } else {
          return Err(ParseKIFGameError::new(
            s.to_owned(),
            "fail to parse game header (no colon delimiter?)".to_owned(),
          ));
        }
      }
      if st == 1 {
        if s == "*時間切れにて終局" {
          g.out_of_time(pos.move_no);
          break;
        }
        if let Some(kif) = parse_move(s, pos.move_no) {
          if kif == "投了" {
            g.resign(pos.move_no);
            break;
          }
          if let Some(m) = pos.parse_kif_move(&mut self.allocator, kif, last_move) {
            pos.do_move(&m);
            last_move = Some(m.clone());
            if pos.is_legal() {
              g.moves.push(m);
            } else {
              st = 2;
            }
          } else {
            last_move = None;
            st = 2;
          }
        }
        //for checking illegal move comment
        continue;
      }
      //after illegal move
      if st == 2 {
        if s != "*反則手にて終局" {
          return Err(ParseKIFGameError::new(
            s.to_owned(),
            "expected illegal move message".to_owned(),
          ));
        }
        g.illegal_move(pos.move_no - 1);
        break;
      }
    }
    Ok(g)
  }
}

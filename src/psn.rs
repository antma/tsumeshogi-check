use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

use crate::shogi::game::{Game, GameResult};
use crate::shogi::moves;
use crate::shogi::Position;
use moves::Move;

#[test]
fn test_parse_header() {
  assert_eq!(
    parse_header("[Date \"24/02/2023\"]"),
    Some((String::from("date"), String::from("24/02/2023")))
  );
}

fn last_line<'a>(s: &'a str) -> Option<&'a str> {
  if let Some(t) = s.strip_prefix("--") {
    t.strip_suffix("--")
  } else {
    None
  }
}

fn parse_header(s: &str) -> Option<(String, String)> {
  let mut key = String::new();
  let mut value = String::new();
  let mut st = 0;
  for c in s.trim().chars() {
    match st {
      0 => {
        if c == '[' {
          st += 1;
        } else {
          return None;
        }
      }
      1 => {
        if c.is_ascii_whitespace() {
          st += 1;
        } else if c.is_ascii_alphabetic() {
          key.push(c.to_ascii_lowercase());
        } else {
          return None;
        }
      }
      2 => {
        if !c.is_ascii_whitespace() {
          if c == '"' {
            st += 1;
          } else {
            return None;
          }
        }
      }
      3 => {
        if c == '"' {
          st += 1;
        } else {
          value.push(c);
        }
      }
      4 => {
        if c == ']' {
          st += 1;
        } else {
          return None;
        }
      }
      _ => return None,
    }
  }
  if st < 5 {
    return None;
  }
  Some((key, value))
}

pub struct PSNFileIterator {
  reader: BufReader<File>,
}

impl Iterator for PSNFileIterator {
  type Item = std::io::Result<Vec<String>>;
  fn next(&mut self) -> Option<Self::Item> {
    let mut r = Vec::new();
    loop {
      let mut s = String::new();
      match self.reader.read_line(&mut s) {
        Ok(sz) => {
          if sz == 0 {
            //EOF reached
            break None;
          }
          let s = s.trim();
          if r.is_empty() && s.is_empty() {
            //skip first blank lines
            continue;
          }
          let last = last_line(s).is_some();
          r.push(String::from(s));
          if last {
            return Some(Ok(r));
          }
        }
        Err(err) => break Some(Err(err)),
      }
    }
  }
}

impl PSNFileIterator {
  pub fn new(filename: &str) -> std::io::Result<Self> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    Ok(Self { reader })
  }
}

#[derive(Debug)]
pub struct ParsePSNGameError {
  pub msg: String,
  pub line: String,
}

impl ParsePSNGameError {
  fn new(line: String, msg: String) -> Self {
    Self { msg, line }
  }
}

pub fn parse_psn_game(a: &Vec<String>) -> std::result::Result<Game, ParsePSNGameError> {
  let mut g = Game::default();
  let mut st = 0;
  let mut pos = Position::default();
  for s in a {
    log::debug!("st = {}, process line {}", st, s);
    if st == 0 {
      //parse headers
      match parse_header(s) {
        Some((key, value)) => {
          g.set_header(key, value);
          continue;
        }
        None => st += 1,
      }
    }
    if st == 1 {
      let prefix = format!("{}.", pos.move_no);
      log::debug!("prefix = {}", prefix);
      if let Some(t) = s.strip_prefix(&prefix) {
        match Move::from_str(t) {
          Ok(mut m) => {
            log::debug!("move = {:?}", m);
            if pos.side < 0 {
              m.swap_piece_side();
            }
            if !pos.validate_move(&m) {
              return Err(ParsePSNGameError::new(
                s.clone(),
                format!("illegal move in position \"{}\"", pos),
              ));
            }
            let _ = pos.do_move(&m);
            if !pos.is_legal() {
              return Err(ParsePSNGameError::new(
                s.clone(),
                String::from("king under check"),
              ));
            }
            g.moves.push(m);
          }
          Err(e) => {
            return Err(ParsePSNGameError::new(s.clone(), format!("{:?}", e)));
          }
        }
      } else if let Some(t) = last_line(s) {
        g.set_header(String::from("text_result"), String::from(t));
        st += 1;
      } else {
        return Err(ParsePSNGameError::new(
          s.clone(),
          String::from("expected move or game result"),
        ));
      }
    } else {
      return Err(ParsePSNGameError::new(
        s.clone(),
        String::from("extra data after result"),
      ));
    }
  }
  match g.result() {
    GameResult::BlackWon => {
      if pos.side < 0 {
        assert_eq!(g.moves.len() % 2, 1);
        if !pos.has_legal_move() {
          g.set_header(String::from("checkmate"), String::from("true"));
        } else {
          g.set_header(String::from("resignation"), String::from("true"));
        }
      } else {
        return Err(ParsePSNGameError::new(
          String::default(),
          format!("unexpected black to move in {}", g.to_short_string()),
        ));
      }
    }
    GameResult::WhiteWon => {
      if pos.side > 0 {
        assert_eq!(g.moves.len() % 2, 0);
        if !pos.has_legal_move() {
          g.set_header(String::from("checkmate"), String::from("true"));
        } else {
          g.set_header(String::from("resignation"), String::from("true"));
        }
      } else {
        return Err(ParsePSNGameError::new(
          String::default(),
          format!("unexpected white to move in {}", g.to_short_string()),
        ));
      }
    }
    GameResult::Unknown => {
      return Err(ParsePSNGameError::new(
        String::default(),
        format!("unknown game result in {}", g.to_short_string()),
      ));
    }
  }
  Ok(g)
}

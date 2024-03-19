use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
//use std::str::FromStr;

use super::{
  alloc::PositionMovesAllocator,
  game::Game,
  //moves::Move,
  Position,
};

pub struct PGNFileIterator {
  reader: BufReader<File>,
}

impl Iterator for PGNFileIterator {
  type Item = std::io::Result<Vec<String>>;
  fn next(&mut self) -> Option<Self::Item> {
    let mut r = Vec::new();
    let mut blanks = 0;
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
          if s.is_empty() {
            blanks += 1;
          }
          if blanks > 1 {
            break Some(Ok(r));
          }
          r.push(s.to_owned());
        }
        Err(err) => break Some(Err(err)),
      }
    }
  }
}

impl PGNFileIterator {
  pub fn new(filename: &str) -> std::io::Result<Self> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    Ok(Self { reader })
  }
}

#[derive(Debug)]
pub struct ParsePGNGameError {
  pub msg: String,
  pub line: String,
}

impl ParsePGNGameError {
  fn new(line: String, msg: String) -> Self {
    Self { msg, line }
  }
}

pub fn parse_pgn_game(a: &Vec<String>) -> std::result::Result<Game, ParsePGNGameError> {
  let mut g = Game::default();
  let mut st = 0;
  let mut pos = Position::default();
  let mut allocator = PositionMovesAllocator::default();
  let mut game_result = String::new();
  for s in a {
    log::debug!("st = {}, process line {}", st, s);
    if st == 0 {
      //parse headers
      if s.is_empty() {
        game_result = g.result().to_string();
        st += 1;
      } else {
        match super::psn::parse_header(s) {
          Some((key, value)) => {
            if key == "result" {
              if value == "1-0" {
                g.set_header(key, "2".to_owned());
              } else if value == "0-1" {
                g.set_header(key, "3".to_owned());
              }
            } else if key == "P1" {
              g.set_header("sente".to_owned(), value);
            } else if key == "P2" {
              g.set_header("gote".to_owned(), value);
            } else {
              g.set_header(key, value);
            }
            continue;
          }
          None => {
            return Err(ParsePGNGameError::new(
              s.clone(),
              String::from("parsing header"),
            ))
          }
        }
      }
    }
    if st == 1 {
      //TODO: s.split_whitespace()
      for t in s.split_whitespace() {
        if t.ends_with('.') {
          //skip move numbers
          continue;
        }
        if t == game_result.as_str() {
          //skip result
          continue;
        }
        let m = pos.parse_pgn_move(&t);
        if m.is_none() {
          return Err(ParsePGNGameError::new(
            t.to_owned(),
            format!("fail to parse pgn move in position \"{}\"", pos),
          ));
        }
        let m = m.unwrap();
        if !pos.validate_move(&m) {
          return Err(ParsePGNGameError::new(
            t.to_owned(),
            format!("illegal move in position \"{}\"", pos),
          ));
        }
        let _ = pos.do_move(&m);
        if !pos.is_legal() {
          return Err(ParsePGNGameError::new(
            t.to_owned(),
            format!("king under check in position \"{}\"", pos),
          ));
        }
        g.moves.push(m);
      }
    }
  }
  let res = g.adjourn(&mut pos, &mut allocator);
  res
    .map(|_| g)
    .map_err(|s| ParsePGNGameError::new(String::default(), s))
}

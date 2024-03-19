use super::{alloc::PositionMovesAllocator, Move, Position};

pub enum GameResult {
  Unknown,
  BlackWon,
  WhiteWon,
}

impl std::fmt::Display for GameResult {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      GameResult::Unknown => "???",
      GameResult::BlackWon => "1-0",
      GameResult::WhiteWon => "0-1",
    };
    write!(f, "{}", s)
  }
}

#[derive(Default)]
pub struct Game {
  pub header: std::collections::BTreeMap<String, String>,
  pub moves: Vec<Move>,
}

const UNDEFINED_STRING: &str = "???";

impl Game {
  pub fn to_short_string(&self) -> String {
    format!(
      "{} vs {} ({})",
      self
        .header
        .get("sente")
        .cloned()
        .unwrap_or_else(|| UNDEFINED_STRING.to_string()),
      self
        .header
        .get("gote")
        .cloned()
        .unwrap_or_else(|| UNDEFINED_STRING.to_string()),
      self.result().to_string()
    )
  }
  pub fn copy_header(&mut self, src: &Game, key: &str) {
    let key = key.to_owned();
    if let Some(value) = src.header.get(&key) {
      self.set_header(key, value.clone());
    }
  }
  pub fn get_header(&self, key: &String) -> Option<&String> {
    self.header.get(key)
  }
  pub fn set_header(&mut self, key: String, value: String) {
    self.header.insert(key, value);
  }
  pub fn loss(&mut self, move_no: u32) {
    self.set_header("result".to_owned(), (2 + (move_no % 2)).to_string());
  }
  pub fn resign(&mut self, move_no: u32) {
    self.set_header("resignation".to_owned(), "true".to_owned());
    self.loss(move_no);
  }
  pub fn illegal_move(&mut self, move_no: u32) {
    self.loss(move_no);
  }
  pub fn out_of_time(&mut self, move_no: u32) {
    self.loss(move_no);
  }
  pub fn result(&self) -> GameResult {
    if let Some(t) = self.header.get("result") {
      match t.as_str() {
        "2" => GameResult::BlackWon,
        "3" => GameResult::WhiteWon,
        _ => GameResult::Unknown,
      }
    } else {
      GameResult::Unknown
    }
  }
  pub fn adjourn(
    &mut self,
    pos: &mut Position,
    allocator: &mut PositionMovesAllocator,
  ) -> std::result::Result<(), String> {
    match self.result() {
      GameResult::BlackWon => {
        if pos.side < 0 {
          assert_eq!(self.moves.len() % 2, 1);
          if !pos.has_legal_move(allocator) {
            self.set_header(String::from("checkmate"), String::from("true"));
          } else {
            self.set_header(String::from("resignation"), String::from("true"));
          }
        } else {
          return Err(format!(
            "unexpected black to move in {}",
            self.to_short_string()
          ));
        }
      }
      GameResult::WhiteWon => {
        if pos.side > 0 {
          assert_eq!(self.moves.len() % 2, 0);
          if !pos.has_legal_move(allocator) {
            self.set_header(String::from("checkmate"), String::from("true"));
          } else {
            self.set_header(String::from("resignation"), String::from("true"));
          }
        } else {
          return Err(format!(
            "unexpected white to move in {}",
            self.to_short_string()
          ));
        }
      }
      GameResult::Unknown => {
        return Err(format!("unknown game result in {}", self.to_short_string()));
      }
    }
    Ok(())
  }
}

use super::piece;
use std::str::FromStr;

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct Move {
  pub from: usize,
  pub to: usize,
  pub from_piece: i8,
  pub to_piece: i8,
}

impl Move {
  pub fn is_pawn_drop(&self) -> bool {
    self.from_piece == piece::NONE && self.to_piece.abs() == piece::PAWN
  }
  pub fn is_drop(&self) -> bool {
    self.from_piece == piece::NONE
  }
  pub fn swap_side(&mut self) {
    self.from_piece *= -1;
    self.to_piece *= -1;
  }
}

#[derive(Debug)]
pub struct MoveFromStrError {
  s: String,
  msg: String,
}

impl MoveFromStrError {
  fn new(s: &str, msg: String) -> Self {
    Self {
      s: String::from(s),
      msg,
    }
  }
}

impl FromStr for Move {
  type Err = MoveFromStrError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut st = 0;
    let mut from_promoted = false;
    let mut to_promoted = false;
    let mut from_piece = 0;
    let mut drop = false;
    let mut _take = false;
    let mut from_col = 0;
    let mut from_row = 0;
    let mut to_col = 0;
    let mut to_row = 0;
    for c in s.chars() {
      match st {
        0 => {
          if c == '+' {
            from_promoted = true;
            st += 1;
          } else {
            from_piece = piece::from_char(c);
            if from_piece <= 0 {
              return Err(MoveFromStrError::new(
                s,
                String::from("can't parse from piece"),
              ));
            }
            st += 2;
          }
        }
        //after +
        1 => {
          from_piece = piece::from_char(c);
          if from_piece <= 0 {
            return Err(MoveFromStrError::new(
              s,
              String::from("can't parse from piece"),
            ));
          }
          st += 1;
        }
        //after piece
        2 => {
          if c == '*' {
            drop = true;
            st = 5;
          } else if c.is_ascii_digit() {
            from_col = ((c as u8) - 48) as i8;
            st += 1;
          } else {
            return Err(MoveFromStrError::new(
              s,
              String::from("expected from column"),
            ));
          }
        }
        //from row
        3 => {
          if 'a' <= c && c <= 'i' {
            from_row = (c as u8 - 96) as i8;
            st += 1;
          } else {
            return Err(MoveFromStrError::new(s, String::from("expected from row")));
          }
        }
        4 => {
          if c == 'x' {
            _take = true;
            st += 1;
          } else if c == '-' {
            st += 1;
          } else {
            return Err(MoveFromStrError::new(
              s,
              String::from("expected move or take"),
            ));
          }
        }
        //to col
        5 => {
          if c.is_ascii_digit() {
            to_col = ((c as u8) - 48) as i8;
            st += 1;
          } else {
            return Err(MoveFromStrError::new(s, String::from("expected to column")));
          }
        }
        6 => {
          if 'a' <= c && c <= 'i' {
            to_row = (c as u8 - 96) as i8;
            st += 1;
          } else {
            return Err(MoveFromStrError::new(s, String::from("expected to row")));
          }
        }
        7 => {
          if !drop {
            if c == '=' {
              st += 1;
            } else if c == '+' {
              to_promoted = true;
              st += 1;
            } else {
              return Err(MoveFromStrError::new(
                s,
                String::from("expected to promotion"),
              ));
            }
          }
        }
        8 => {
          return Err(MoveFromStrError::new(s, String::from("eoln expected")));
        }
        _ => {
          panic!("unhandled state {}", st);
        }
      }
    }
    if st < 7 {
      return Err(MoveFromStrError::new(s, String::from("incomplete move")));
    }
    log::debug!("to_row = {}, to_col = {}", to_row, to_col);
    let to_cell = (9 * (to_row - 1) + (to_col - 1)) as usize;
    if drop {
      if from_promoted {
        return Err(MoveFromStrError::new(
          s,
          String::from("promoted piece can't be dropped"),
        ));
      }
      Ok(Move {
        from_piece: 0,
        to_piece: from_piece,
        from: 0xff,
        to: to_cell,
      })
    } else {
      if from_promoted && to_promoted {
        return Err(MoveFromStrError::new(
          s,
          String::from("already promoted piece can't be promoted"),
        ));
      }
      let p = from_piece + (if from_promoted { piece::PROMOTED } else { 0 });
      Ok(Move {
        from: (9 * (from_row - 1) + (from_col - 1)) as usize,
        to: to_cell,
        from_piece: p,
        to_piece: p + (if to_promoted { piece::PROMOTED } else { 0 }),
      })
    }
  }
}

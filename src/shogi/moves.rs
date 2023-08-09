use super::kif::push_cell_as_jp_str;
use super::Position;
use super::{cell, piece};
use std::str::FromStr;

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct Move {
  pub from: usize,
  pub to: usize,
  pub from_piece: i8,
  pub to_piece: i8,
}

pub struct UndoMove {
  pub all_pieces: u128,
  pub all_pieces2: u128,
  pub all_pieces3: u128,
  pub all_pieces4: u128,
  pub black_pieces: u128,
  pub white_pieces: u128,
  pub sliding_pieces: u128,
  pub hash: u64,
  pub drop_masks: u32,
  pub nifu_masks: u32,
  pub taken_piece: i8,
}

impl UndoMove {
  pub fn is_take(&self) -> bool {
    self.taken_piece != piece::NONE
  }
}

impl std::convert::From<Move> for u32 {
  fn from(m: Move) -> u32 {
    //7, 7, 6, 6
    ((m.from as u32) << 19)
      + ((m.to as u32) << 12)
      + (((m.from_piece + 32) as u32) << 6)
      + ((m.to_piece + 32) as u32)
  }
}

impl std::convert::From<&Move> for u32 {
  fn from(m: &Move) -> u32 {
    //7, 7, 6, 6
    ((m.from as u32) << 19)
      + ((m.to as u32) << 12)
      + (((m.from_piece + 32) as u32) << 6)
      + ((m.to_piece + 32) as u32)
  }
}

impl std::convert::From<u32> for Move {
  fn from(x: u32) -> Move {
    let mut x = x;
    let to_piece = (x & 63) as i8 - 32;
    x >>= 6;
    let from_piece = (x & 63) as i8 - 32;
    x >>= 6;
    let to = (x & 127) as usize;
    Move {
      from: (x >> 7) as usize,
      to,
      from_piece,
      to_piece,
    }
  }
}

impl Move {
  pub fn is_pawn_drop(&self) -> bool {
    self.from_piece == piece::NONE && self.to_piece.abs() == piece::PAWN
  }
  pub fn is_drop(&self) -> bool {
    self.from_piece == piece::NONE
  }
  pub fn is_promotion(&self) -> bool {
    self.from_piece != piece::NONE && self.from_piece != self.to_piece
  }
  pub fn is_king_move(&self) -> bool {
    self.from_piece.abs() == piece::KING
  }
  pub fn swap_piece_side(&mut self) {
    self.from_piece *= -1;
    self.to_piece *= -1;
  }
  fn mirror(&mut self) {
    if !self.is_drop() {
      self.from = cell::mirror(self.from);
    }
    self.to = cell::mirror(self.to);
  }
  pub fn swap_side(&mut self) {
    self.mirror();
    self.swap_piece_side();
  }
  pub fn to_kif(&self, prev_move: &Option<Move>) -> String {
    let mut s = String::with_capacity(8);
    if self.is_drop() {
      push_cell_as_jp_str(&mut s, self.to);
      s.push_str(&piece::to_jp_string(self.to_piece.abs()));
      s.push('打');
    } else {
      let cell = prev_move.as_ref().map(|q| q.to).unwrap_or(0xff);
      if cell == self.to {
        s.push_str("同　");
      } else {
        push_cell_as_jp_str(&mut s, self.to);
      }
      if self.to_piece != self.from_piece {
        s.push_str(&piece::to_jp_string(self.from_piece.abs()));
        s.push('成');
      } else {
        s.push_str(&piece::to_jp_string(self.to_piece.abs()));
      }
      s.push('(');
      cell::push_cell_as_en_str(&mut s, self.from, true);
      s.push(')');
    }
    s
  }
  pub fn to_psn(&self, is_take: bool) -> String {
    if self.is_drop() {
      let mut s = piece::to_string(self.to_piece, true);
      s.push('\'');
      cell::push_cell_as_en_str(&mut s, self.to, false);
      s
    } else {
      let mut s = piece::to_string(self.from_piece, true);
      cell::push_cell_as_en_str(&mut s, self.from, false);
      s.push(if is_take { 'x' } else { '-' });
      cell::push_cell_as_en_str(&mut s, self.to, false);
      if !piece::is_promoted(self.from_piece)
        && piece::could_promoted(self.from_piece)
        && (cell::promotion_zone(self.from, self.to_piece)
          || cell::promotion_zone(self.to, self.to_piece))
      {
        s.push(if piece::is_promoted(self.to_piece) {
          '+'
        } else {
          '='
        });
      }
      s
    }
  }
}

pub struct PSNMove(Move, bool);
impl std::fmt::Display for PSNMove {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0.to_psn(self.1))
  }
}
impl PSNMove {
  pub fn new(m: &Move, u: &UndoMove) -> Self {
    PSNMove(m.clone(), u.taken_piece != piece::NONE)
  }
}

pub fn moves_to_psn(moves: &Vec<PSNMove>) -> String {
  let mut s = String::new();
  for (i, m) in moves.iter().enumerate() {
    if i > 0 {
      s.push(' ');
    }
    s.push_str(&(i + 1).to_string());
    s.push('.');
    s.push_str(&m.to_string());
  }
  s
}

#[derive(Debug)]
pub struct MoveFromStrError {
  pub s: String,
  pub msg: String,
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

pub struct Moves {
  moves: Vec<Move>,
  undos: Vec<UndoMove>,
}

pub fn moves_to_kif(moves: &Vec<Move>, mut side: i8) -> String {
  let mut s = String::new();
  let mut prev: Option<Move> = None;
  for m in moves {
    if prev.is_some() {
      s.push(' ');
    }
    s.push(if side > 0 { '☗' } else { '☖' });
    s.push_str(m.to_kif(&prev).as_str());
    prev = Some(m.clone());
    side *= -1;
  }
  s
}

impl Moves {
  pub fn with_capacity(capacity: usize) -> Self {
    Moves {
      moves: Vec::with_capacity(capacity),
      undos: Vec::with_capacity(capacity),
    }
  }
  pub fn len(&self) -> usize {
    self.moves.len()
  }
  pub fn push(&mut self, pos: &mut Position, m: Move) {
    self.undos.push(pos.do_move(&m));
    self.moves.push(m);
  }
  pub fn pop(&mut self, pos: &mut Position) -> Option<Move> {
    let o = self.moves.pop();
    if let Some(m) = o.as_ref() {
      let u = self.undos.pop().unwrap();
      pos.undo_move(m, &u);
    }
    o
  }
  pub fn undo(&self, pos: &mut Position) {
    for (m, u) in self.moves.iter().zip(self.undos.iter()).rev() {
      pos.undo_move(m, u);
    }
  }
  pub fn only_moves(self) -> Vec<Move> {
    self.moves
  }
  pub fn to_kif(&self, side: i8) -> String {
    moves_to_kif(&self.moves, side)
  }
}

#[derive(Default)]
pub struct HistoryTable(std::collections::HashMap<u32, u64>);
impl HistoryTable {
  fn get(&self, m: &Move) -> u64 {
    *self.0.get(&u32::from(m)).unwrap_or(&0)
  }
  pub fn increment(&mut self, m: Move) {
    self
      .0
      .entry(u32::from(m))
      .and_modify(|e| *e += 1)
      .or_insert(1);
  }
  pub fn sort(&self, m: &mut Vec<Move>) {
    m.sort_by_cached_key(|m| u64::MAX - self.get(m));
  }
}

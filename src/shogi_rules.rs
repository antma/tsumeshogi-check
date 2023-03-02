use std::str::FromStr;

pub mod piece {
  type Dir = (isize, isize);
  pub const NONE: i8 = 0;
  pub const PAWN: i8 = 1;
  pub const LANCE: i8 = 2;
  pub const KNIGHT: i8 = 3;
  pub const SILVER: i8 = 4;
  pub const GOLD: i8 = 5;
  pub const BISHOP: i8 = 6;
  pub const ROOK: i8 = 7;
  pub const KING: i8 = 8;
  pub const PROMOTED: i8 = 16;
  pub const PROMOTED_PAWN: i8 = PAWN + PROMOTED;
  pub const PROMOTED_LANCE: i8 = LANCE + PROMOTED;
  pub const PROMOTED_KNIGHT: i8 = KNIGHT + PROMOTED;
  pub const PROMOTED_SILVER: i8 = SILVER + PROMOTED;
  pub const PROMOTED_BISHOP: i8 = BISHOP + PROMOTED;
  pub const PROMOTED_ROOK: i8 = ROOK + PROMOTED;
  pub const KNIGHT_MOVES: [(isize, isize); 2] = [(-2, -1), (-2, 1)];
  pub const SILVER_MOVES: [(isize, isize); 5] = [(-1, -1), (-1, 0), (-1, 1), (1, -1), (1, 1)];
  pub const GOLD_MOVES: [(isize, isize); 6] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, 0)];
  pub const ROOK_MOVES: [(isize, isize); 4] = [(-1, 0), (0, -1), (0, 1), (1, 0)];
  pub const BISHOP_MOVES: [(isize, isize); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
  pub const KING_MOVES: [(isize, isize); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
  //flags: +1 - bishop
  //flags: +2 - rook
  //flags: +4 - general (forward)
  pub const BLACK_DIRECTIONS: [(isize, isize, u8); 8] = [
    (-1, -1, 1 + 4), (-1, 0, 2 + 4), (-1, 1, 1 + 4),
    (0, -1, 2), (0, 1, 2),
    (1, -1, 1), (1, 0, 2), (1, 1, 1)
  ];
  pub const WHITE_DIRECTIONS: [(isize, isize, u8); 8] = [
    (1, -1, 1 + 4), (1, 0, 2 + 4), (1, 1, 1 + 4),
    (0, -1, 2), (0, 1, 2),
    (-1, -1, 1), (-1, 0, 2), (-1, 1, 1)
  ];
  pub fn unpromote(v: i8) -> i8 {
    if v >= PROMOTED { v - PROMOTED }
    else if v <= -PROMOTED { v + PROMOTED }
    else { v }
  }
  pub fn could_unpromoted(piece: i8, cell: usize) -> bool {
    if piece.abs() >= PROMOTED { return false; }
    if piece == PAWN || piece == LANCE { cell < 9 }
    else if piece == -PAWN || piece == -LANCE { cell >= 72 }
    else if piece == KNIGHT { cell < 18 }
    else if piece == -KNIGHT { cell >= 63 }
    else { false }
  }
  pub fn could_promoted(piece: i8) -> bool {
    let p = piece.abs();
    p < KING && p != GOLD
  }
  pub fn is_sliding_dir(abs_piece: i8, flags: u8) -> bool {
    assert!(abs_piece > 0);
    match abs_piece {
      LANCE => (flags & 6) == 6,
      ROOK | PROMOTED_ROOK => (flags & 2) != 0,
      BISHOP | PROMOTED_BISHOP => (flags & 1) != 0,
      _ => false
    }
  }
  pub fn is_near_dir(abs_piece: i8, flags: u8) -> bool {
    assert!(abs_piece > 0);
    match abs_piece {
      PROMOTED_ROOK | PROMOTED_BISHOP | KING => true,
      PAWN | LANCE => (flags & 6) == 6,
      BISHOP => (flags & 1) != 0,
      ROOK => (flags & 2) != 0,
      SILVER => (flags & 5) != 0,
      PROMOTED_PAWN | PROMOTED_LANCE | PROMOTED_KNIGHT | PROMOTED_SILVER | GOLD => (flags & 6) != 0,
      KNIGHT => false,
      _ => {
        panic!("piece::is_near_dir() unhandled piece {}", abs_piece);
        false
      }
    }
  }
   //k8/p1K6/1N7/9/9/9/9/9/9 w P2r2b4g4s3n4l16p 1
  pub fn from_char(c: char) -> i8 {
    let s =
      if c.is_uppercase() { 1 }
      else if c.is_lowercase() { -1 }
      else { 0 };
    if s == 0 { return NONE; }
    match c.to_ascii_lowercase() {
      'p' => s * PAWN,
      'l' => s * LANCE,
      'n' => s * KNIGHT,
      's' => s * SILVER,
      'g' => s * GOLD,
      'b' => s * BISHOP,
      'r' => s * ROOK,
      'k' => s * KING,
      _ => NONE,
    }
  }
}

pub struct Position {
  board: [i8; 81],
  black_pockets: [u8; 8],
  white_pockets: [u8; 8],
  side: i8,
  move_no: u32,
  //checks: Option<Box<Checks>>,
}

pub mod cell {
  pub fn to_string(row: usize, col: usize) -> String {
    format!("{}{}", col + 1, row + 1)
  }
}

fn cell(row: usize, col: usize) -> usize {
  row * 9 + col
}

fn promotion_zone(cell: usize, side: i8) -> bool {
  if side == 1 {
    cell < 27
  } else {
    assert_eq!(side, -1);
    cell >= 54
  }
}

struct Move {
  from: usize,
  to: usize,
  from_piece: i8,
  to_piece: i8,
}

#[derive(Default)]
pub struct Checks {
  pub attacking_pieces: Vec<usize>,
  pub blocking_cells: u128,
}

#[derive(Debug)]
pub struct ParseSFENError {
  sfen: String,
  message: String,
}

impl ParseSFENError {
  fn new(sfen: &str, message: String) -> Self {
    ParseSFENError {
      sfen: String::from(sfen),
      message,
    }
  }
}


impl Position {
  pub fn parse_sfen(sfen: &str) -> Result<Self, ParseSFENError> {
    //k8/p1K6/1N7/9/9/9/9/9/9 w P2r2b4g4s3n4l16p 1
    let a: Vec<_> = sfen.split(' ').collect();
    if a.len() != 4 {
      return Err(ParseSFENError::new(sfen, format!("invalid number of tokens ({}), expected <position> <color> <pocket> <move>", a.len())));
    }
    let b: Vec<_> = a[0].split('/').collect();
    if b.len() != 9 {
      return Err(ParseSFENError::new(sfen, format!("invalid number of rows ({})", b.len())));
    }
    let mut board: [i8; 81] = [piece::NONE; 81];
    for (row, s) in b.iter().enumerate() {
      let mut col = 0;
      let mut promoted = 0;
      for c in s.chars() {
        if c.is_digit(10) {
          col += c.to_digit(10).unwrap() as usize;
          if col > 9 {
            return Err(ParseSFENError::new(sfen, format!("invalid number of columns in row {}", row + 1)));
          }
        } else if c == '+' {
          if promoted != 0 {
            return Err(ParseSFENError::new(sfen, format!("double promotion in cell {}", cell::to_string(row, 8-col))));
          }
          promoted = piece::PROMOTED;
        } else {
          if col > 9 {
            return Err(ParseSFENError::new(sfen, format!("invalid number of columns in row {}", row + 1)));
          }
          let p = piece::from_char(c);
          if p == piece::NONE {
            return Err(ParseSFENError::new(sfen, format!("invalid piece in cell {}", cell::to_string(row, 8-col))));
          }
          if promoted != 0 {
            if p == piece::KING {
              return Err(ParseSFENError::new(sfen, format!("promoted king in cell {}", cell::to_string(row, 8-col))));
            }
            if p == piece::GOLD {
              return Err(ParseSFENError::new(sfen, format!("promoted gold general in cell {}", cell::to_string(row, 8-col))));
            }
          }
          board[9 * row + (8 - col)] = p + promoted * p.signum();
          promoted = 0;
          col += 1;
        }
      }
    }
    let side =
      if a[1] == "w" { -1 }
      else if a[1] == "b" { 1 }
      else { return Err(ParseSFENError::new(sfen, String::from("invalid color"))); };
    let mut black_pockets: [u8; 8] = [0; 8];
    let mut white_pockets: [u8; 8] = [0; 8];
    if a[2] != "-" {
      let mut cnt = 0;
      for c in a[2].chars() {
        if c.is_digit(10) {
          cnt = 10 * cnt + c.to_digit(10).unwrap() as u8;
        } else {
          let p = piece::from_char(c);
          if p == piece::NONE {
            return Err(ParseSFENError::new(sfen, String::from("invalid dropping piece")));
          }
          if p.abs() == piece::KING {
            return Err(ParseSFENError::new(sfen, String::from("king in dropping piece")));
          }
          if cnt == 0 { cnt = 1; }
          if p > 0 {
            black_pockets[p as usize] += cnt;
          } else {
            white_pockets[(-p) as usize] += cnt;
          }
          cnt = 0;
        }
      }
    }
    let move_no = u32::from_str(&a[3]);
    if move_no.is_err() {
      return Err(ParseSFENError::new(sfen, String::from("invalid move number")));
    }
    let move_no = move_no.unwrap();
    Ok(Position {
      board,
      black_pockets,
      white_pockets,
      side,
      move_no,
    })
  }
  //true -> stop, false -> continue
  fn enumerate_piece_move<F: Fn(&Position, Move) -> bool>
    (&self, f: &F, pos: usize, piece: i8, delta_row: isize, delta_col: isize, sliding: bool) -> bool {
    let mut row = pos / 9;
    let mut col = pos % 9;
    let p = promotion_zone(pos, self.side);
    loop {
      let r = (row as isize) + delta_row;
      if r < 0 || r >= 9 { break; }
      let c = (col as isize) + delta_col;
      if c < 0 || c >= 9 { break; }
      row = r as usize;
      col = c as usize;
      let k = row * 9 + col;
      let t = piece * self.board[k].signum();
      if t > 0 { break; }
      if piece::could_unpromoted(piece, k) {
        let m = Move {
          from: pos,
          to: k,
          from_piece: piece,
          to_piece: piece,
        };
        if f(&self, m) { return true; }
      }
      if piece::could_promoted(piece) && (p || promotion_zone(k, self.side)) {
        let m = Move {
          from: pos,
          to: k,
          from_piece: piece,
          to_piece: piece + piece.signum() * piece::PROMOTED,
        };
        if f(&self, m) { return true; }
      }
      if t != 0 || !sliding { break; }
    }
    false
  }
  fn enumerate_simple_moves<F: Fn(&Position, Move) -> bool>(&self, f: &F) -> bool {
    for (pos, &v) in self.board.iter().enumerate() {
      if self.side * v <= 0 { continue; }
      let w = piece::unpromote(v);
      if v == piece::PAWN {
        if self.enumerate_piece_move(f, pos, v, -1, 0, false) { return true; }
      } else if v == -piece::PAWN {
        if self.enumerate_piece_move(f, pos, v, 1, 0, false) { return true; }
      } else if v == piece::LANCE {
        if self.enumerate_piece_move(f, pos, v, -1, 0, true) { return true; }
      } else if v == -piece::LANCE {
        if self.enumerate_piece_move(f, pos, v, 1, 0, true) { return true; }
      } else if v == piece::KNIGHT {
        if self.enumerate_piece_move(f, pos, v, -2, -1, false) { return true; }
        if self.enumerate_piece_move(f, pos, v, -2, 1, false) { return true; }
      } else if v == -piece::KNIGHT {
        if self.enumerate_piece_move(f, pos, v, 2, -1, false) { return true; }
        if self.enumerate_piece_move(f, pos, v, 2, 1, false) { return true; }
      } else if v == piece::SILVER {
        for t in piece::SILVER_MOVES.iter() {
          if self.enumerate_piece_move(f, pos, v, t.0, t.1, false) { return true; }
        }
      } else if v == -piece::SILVER {
        for t in piece::SILVER_MOVES.iter() {
          if self.enumerate_piece_move(f, pos, v, -t.0, -t.1, false) { return true; }
        }
      } else if v == piece::GOLD || v == piece::PROMOTED_PAWN || v == piece::PROMOTED_LANCE ||
                v == piece::PROMOTED_KNIGHT || v == piece::PROMOTED_SILVER {
        for t in piece::GOLD_MOVES.iter() {
          if self.enumerate_piece_move(f, pos, v, t.0, t.1, false) { return true; }
        }
      } else if v == -piece::GOLD || v == -piece::PROMOTED_PAWN || v == -piece::PROMOTED_LANCE ||
                v == -piece::PROMOTED_KNIGHT || v == -piece::PROMOTED_SILVER {
        for t in piece::GOLD_MOVES.iter() {
          if self.enumerate_piece_move(f, pos, v, -t.0, -t.1, false) { return true; }
        }
      } else if w == piece::BISHOP || w == -piece::BISHOP {
        for t in piece::BISHOP_MOVES.iter() {
          if self.enumerate_piece_move(f, pos, v, t.0, t.1, true) { return true; }
        }
      } else if w == piece::ROOK || w == -piece::ROOK {
        for t in piece::ROOK_MOVES.iter() {
          if self.enumerate_piece_move(f, pos, v, t.0, t.1, true) { return true; }
        }
      } else if v == piece::KING || v == -piece::KING {
        for t in piece::KING_MOVES.iter() {
          if self.enumerate_piece_move(f, pos, v, t.0, t.1, false) { return true; }
        }
      }
      //promoted
      if v != w {
        if w == piece::BISHOP || w == -piece::BISHOP {
          for t in piece::ROOK_MOVES.iter() {
            if self.enumerate_piece_move(f, pos, v, t.0, t.1, false) { return true; }
          }
        } else if w == piece::ROOK || w == -piece::ROOK {
          for t in piece::BISHOP_MOVES.iter() {
            if self.enumerate_piece_move(f, pos, v, t.0, t.1, false) { return true; }
          }
        }
      }
    }
    false
  }
  fn enumerate_drops<F: Fn(&Position, Move) -> bool, G: Fn(usize) -> bool>(&self, f: &F, g: &G) -> bool {
    let pawn = piece::PAWN * self.side;
    let q = if self.side > 0 { &self.black_pockets } else { &self.white_pockets };
    let v: Vec<i8> = (piece::PAWN ..= piece::ROOK).filter(|&i| q[i as usize] > 0).collect();
    for col in 0 .. 9 {
      let allow_pawn_drop = q[piece::PAWN as usize] > 0 && !(0..9).any(|r| self.board[9*r+col] == pawn);
      for row in 0 .. 9 {
        let k = row * 9 + col;
        if self.board[k] == piece::NONE && g(k) {
          for &p in &v {
            if p == piece::PAWN && !allow_pawn_drop { continue; }
            let m = Move {
              from: 0xff,
              to: k,
              from_piece: piece::NONE,
              to_piece: p * self.side,
            };
            if f(&self, m) { return true; }
          }
        }
      }
    }
    false
  }
  fn checks(&self, king_pos: usize, s: i8) -> Checks {
    let king_row = king_pos / 9;
    let king_col = king_pos % 9;
    let mut attacking_pieces = Vec::new();
    let mut blocking_cells = 0u128;
    for t in if s > 0 { piece::BLACK_DIRECTIONS.iter() } else { piece::WHITE_DIRECTIONS.iter() } {
      let mut row = king_row;
      let mut col = king_col;
      let mut cells = 0;
      let q = loop {
        let r = (row as isize) + t.0;
        if r < 0 || r >= 9 { break None; }
        let c = (col as isize) + t.1;
        if c < 0 || c >= 9 { break None; }
        row = r as usize;
        col = c as usize;
        let k = row * 9 + col;
        let piece = self.board[k];
        let t = s * piece;
        if t < 0 {
          break Some((k, piece));
        } else if t == 0 {
          cells |= 1u128 << k;
        } else {
          break None;
        }
      };
      match q {
        Some( (k, piece)) => {
          let p = piece.abs();
          let b =
          if cells == 0 {
            piece::is_near_dir(p, t.2)
          } else {
            piece::is_sliding_dir(p, t.2)
          };
          if b {
            attacking_pieces.push(k);
            blocking_cells |= cells;
          }
        },
        _ => (),
      }
    }
    //knight checks
    for t in piece::KNIGHT_MOVES.iter() {
      let r = (king_row as isize) + t.0 * (s as isize);
      if r < 0 || r >= 9 { continue; }
      let c = (king_col as isize) + t.1 * (s as isize);
      if c < 0 || c >= 9 { continue; }
      let k = r as usize * 9 + c as usize;
      let piece = self.board[k];
      if s * piece >= 0 { continue; }
      if piece.abs() != piece::KNIGHT { continue; }
      attacking_pieces.push(k);
    }
    //double checks can't be blocked
    if attacking_pieces.len() > 1 { blocking_cells = 0; }
    Checks {
      attacking_pieces, blocking_cells,
    }
  }
  fn find_king(&self, s: i8) -> Option<usize> {
    let king = piece::KING * s;
    self.board.iter().enumerate().find_map(|(i, v)| if *v == king { Some(i) } else { None })
  }
  pub fn find_checks(&self) -> Checks {
    let king_pos = self.find_king(self.side);
    match king_pos {
      Some(king_pos) => self.checks(king_pos, self.side),
      None => Checks::default(),
    }
  }
  pub fn is_check(&self) -> bool { !self.find_checks().attacking_pieces.is_empty() }
  fn enumerate_moves(&self) -> Vec<Move> {
    Vec::new()
  }
}

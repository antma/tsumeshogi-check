use std::str::FromStr;

mod cell;
mod piece;

pub struct Position {
  board: [i8; 81],
  black_pockets: [u8; 8],
  white_pockets: [u8; 8],
  side: i8,
  move_no: u32,
}

fn cell(row: usize, col: usize) -> usize {
  row * 9 + col
}

pub struct Move {
  from: usize,
  to: usize,
  from_piece: i8,
  to_piece: i8,
}

#[derive(Default)]
pub struct Checks {
  pub blocking_cells: u128,
  pub attacking_pieces: Vec<usize>,
  king_pos: usize,
}

impl Checks {
  pub fn is_check(&self) -> bool {
    !self.attacking_pieces.is_empty()
  }
  pub fn is_double_check(&self) -> bool {
    if self.attacking_pieces.len() >= 2 {
      assert_eq!(self.blocking_cells, 0);
      true
    } else {
      false
    }
  }
  fn blocking_cell(&self, cell: usize) -> bool {
    (self.blocking_cells & (1u128 << cell)) != 0
  }
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

pub struct UndoMove {
  taken_piece: i8,
}

impl Position {
  pub fn parse_sfen(sfen: &str) -> Result<Self, ParseSFENError> {
    let a: Vec<_> = sfen.split(' ').collect();
    if a.len() != 4 {
      return Err(ParseSFENError::new(
        sfen,
        format!(
          "invalid number of tokens ({}), expected <position> <color> <pocket> <move>",
          a.len()
        ),
      ));
    }
    let b: Vec<_> = a[0].split('/').collect();
    if b.len() != 9 {
      return Err(ParseSFENError::new(
        sfen,
        format!("invalid number of rows ({})", b.len()),
      ));
    }
    let mut board: [i8; 81] = [piece::NONE; 81];
    for (row, s) in b.iter().enumerate() {
      let mut col = 0;
      let mut promoted = 0;
      for c in s.chars() {
        if c.is_digit(10) {
          col += c.to_digit(10).unwrap() as usize;
          if col > 9 {
            return Err(ParseSFENError::new(
              sfen,
              format!("invalid number of columns in row {}", row + 1),
            ));
          }
        } else if c == '+' {
          if promoted != 0 {
            return Err(ParseSFENError::new(
              sfen,
              format!("double promotion in cell {}", cell::to_string(row, 8 - col)),
            ));
          }
          promoted = piece::PROMOTED;
        } else {
          if col > 9 {
            return Err(ParseSFENError::new(
              sfen,
              format!("invalid number of columns in row {}", row + 1),
            ));
          }
          let p = piece::from_char(c);
          if p == piece::NONE {
            return Err(ParseSFENError::new(
              sfen,
              format!("invalid piece in cell {}", cell::to_string(row, 8 - col)),
            ));
          }
          if promoted != 0 {
            if p == piece::KING {
              return Err(ParseSFENError::new(
                sfen,
                format!("promoted king in cell {}", cell::to_string(row, 8 - col)),
              ));
            }
            if p == piece::GOLD {
              return Err(ParseSFENError::new(
                sfen,
                format!(
                  "promoted gold general in cell {}",
                  cell::to_string(row, 8 - col)
                ),
              ));
            }
          }
          board[9 * row + (8 - col)] = p + promoted * p.signum();
          promoted = 0;
          col += 1;
        }
      }
    }
    //checking nifu
    for col in 0..9 {
      if board
        .iter()
        .skip(col)
        .step_by(9)
        .filter(|&&q| q == piece::PAWN)
        .count()
        >= 2
      {
        return Err(ParseSFENError::new(
          sfen,
          format!("more than one black pawn in column {}", col),
        ));
      }
      if board
        .iter()
        .skip(col)
        .step_by(9)
        .filter(|&&q| q == -piece::PAWN)
        .count()
        >= 2
      {
        return Err(ParseSFENError::new(
          sfen,
          format!("more than one white pawn in column {}", col),
        ));
      }
    }
    //TODO: check pawns and knights in promotion zone
    let side = if a[1] == "w" {
      -1
    } else if a[1] == "b" {
      1
    } else {
      return Err(ParseSFENError::new(sfen, String::from("invalid color")));
    };
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
            return Err(ParseSFENError::new(
              sfen,
              String::from("invalid dropping piece"),
            ));
          }
          if p.abs() == piece::KING {
            return Err(ParseSFENError::new(
              sfen,
              String::from("king in dropping piece"),
            ));
          }
          if cnt == 0 {
            cnt = 1;
          }
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
      return Err(ParseSFENError::new(
        sfen,
        String::from("invalid move number"),
      ));
    }
    let move_no = move_no.unwrap();
    let pos = Position {
      board,
      black_pockets,
      white_pockets,
      side,
      move_no,
    };
    if pos.is_legal() {
      Ok(pos)
    } else {
      Err(ParseSFENError::new(sfen, String::from("king under check")))
    }
  }
  //true -> stop, false -> continue
  fn enumerate_piece_move<F: FnMut(Move) -> bool>(
    &self,
    f: &mut F,
    pos: usize,
    piece: i8,
    delta_row: isize,
    delta_col: isize,
    sliding: bool,
  ) -> bool {
    let (mut row, mut col) = cell::unpack(pos);
    let p = cell::promotion_zone(pos, self.side);
    loop {
      let r = (row as isize) + delta_row;
      if r < 0 || r >= 9 {
        break;
      }
      let c = (col as isize) + delta_col;
      if c < 0 || c >= 9 {
        break;
      }
      row = r as usize;
      col = c as usize;
      let k = row * 9 + col;
      let t = piece * self.board[k].signum();
      if t > 0 {
        break;
      }
      if piece::could_unpromoted(piece, k) {
        let m = Move {
          from: pos,
          to: k,
          from_piece: piece,
          to_piece: piece,
        };
        if f(m) {
          return true;
        }
      }
      if piece::could_promoted(piece) && (p || cell::promotion_zone(k, self.side)) {
        let m = Move {
          from: pos,
          to: k,
          from_piece: piece,
          to_piece: piece + piece.signum() * piece::PROMOTED,
        };
        if f(m) {
          return true;
        }
      }
      if t != 0 || !sliding {
        break;
      }
    }
    false
  }
  fn enumerate_simple_moves<F: FnMut(Move) -> bool>(&self, mut f: F) -> bool {
    for (pos, &v) in self.board.iter().enumerate() {
      if self.side * v <= 0 {
        continue;
      }
      let w = piece::unpromote(v);
      if v == piece::PAWN {
        if self.enumerate_piece_move(&mut f, pos, v, -1, 0, false) {
          return true;
        }
      } else if v == -piece::PAWN {
        if self.enumerate_piece_move(&mut f, pos, v, 1, 0, false) {
          return true;
        }
      } else if v == piece::LANCE {
        if self.enumerate_piece_move(&mut f, pos, v, -1, 0, true) {
          return true;
        }
      } else if v == -piece::LANCE {
        if self.enumerate_piece_move(&mut f, pos, v, 1, 0, true) {
          return true;
        }
      } else if v == piece::KNIGHT {
        if self.enumerate_piece_move(&mut f, pos, v, -2, -1, false) {
          return true;
        }
        if self.enumerate_piece_move(&mut f, pos, v, -2, 1, false) {
          return true;
        }
      } else if v == -piece::KNIGHT {
        if self.enumerate_piece_move(&mut f, pos, v, 2, -1, false) {
          return true;
        }
        if self.enumerate_piece_move(&mut f, pos, v, 2, 1, false) {
          return true;
        }
      } else if v == piece::SILVER {
        for t in piece::SILVER_MOVES.iter() {
          if self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false) {
            return true;
          }
        }
      } else if v == -piece::SILVER {
        for t in piece::SILVER_MOVES.iter() {
          if self.enumerate_piece_move(&mut f, pos, v, -t.0, -t.1, false) {
            return true;
          }
        }
      } else if v == piece::GOLD
        || v == piece::PROMOTED_PAWN
        || v == piece::PROMOTED_LANCE
        || v == piece::PROMOTED_KNIGHT
        || v == piece::PROMOTED_SILVER
      {
        for t in piece::GOLD_MOVES.iter() {
          if self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false) {
            return true;
          }
        }
      } else if v == -piece::GOLD
        || v == -piece::PROMOTED_PAWN
        || v == -piece::PROMOTED_LANCE
        || v == -piece::PROMOTED_KNIGHT
        || v == -piece::PROMOTED_SILVER
      {
        for t in piece::GOLD_MOVES.iter() {
          if self.enumerate_piece_move(&mut f, pos, v, -t.0, -t.1, false) {
            return true;
          }
        }
      } else if w == piece::BISHOP || w == -piece::BISHOP {
        for t in piece::BISHOP_MOVES.iter() {
          if self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, true) {
            return true;
          }
        }
      } else if w == piece::ROOK || w == -piece::ROOK {
        for t in piece::ROOK_MOVES.iter() {
          if self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, true) {
            return true;
          }
        }
      } else if v == piece::KING || v == -piece::KING {
        for t in piece::KING_MOVES.iter() {
          if self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false) {
            return true;
          }
        }
      }
      //promoted
      if v != w {
        if w == piece::BISHOP || w == -piece::BISHOP {
          for t in piece::ROOK_MOVES.iter() {
            if self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false) {
              return true;
            }
          }
        } else if w == piece::ROOK || w == -piece::ROOK {
          for t in piece::BISHOP_MOVES.iter() {
            if self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false) {
              return true;
            }
          }
        }
      }
    }
    false
  }
  fn enumerate_drops<F: Fn(&Position, Move) -> bool, G: Fn(usize) -> bool>(
    &self,
    f: &F,
    g: &G,
  ) -> bool {
    let pawn = piece::PAWN * self.side;
    let q = if self.side > 0 {
      &self.black_pockets
    } else {
      &self.white_pockets
    };
    let v: Vec<i8> = (piece::PAWN..=piece::ROOK)
      .filter(|&i| q[i as usize] > 0)
      .collect();
    for col in 0..9 {
      let allow_pawn_drop =
        q[piece::PAWN as usize] > 0 && !(0..9).any(|r| self.board[9 * r + col] == pawn);
      for row in 0..9 {
        let k = row * 9 + col;
        if self.board[k] == piece::NONE && g(k) {
          for &p in &v {
            if p == piece::PAWN && !allow_pawn_drop {
              continue;
            }
            let m = Move {
              from: 0xff,
              to: k,
              from_piece: piece::NONE,
              to_piece: p * self.side,
            };
            if f(&self, m) {
              return true;
            }
          }
        }
      }
    }
    false
  }
  fn checks(&self, king_pos: usize, s: i8) -> Checks {
    let (king_row, king_col) = cell::unpack(king_pos);
    let mut attacking_pieces = Vec::new();
    let mut blocking_cells = 0u128;
    for t in if s > 0 {
      piece::BLACK_DIRECTIONS.iter()
    } else {
      piece::WHITE_DIRECTIONS.iter()
    } {
      let mut row = king_row;
      let mut col = king_col;
      let mut cells = 0;
      let q = loop {
        let r = (row as isize) + t.0;
        if r < 0 || r >= 9 {
          break None;
        }
        let c = (col as isize) + t.1;
        if c < 0 || c >= 9 {
          break None;
        }
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
        Some((k, piece)) => {
          let p = piece.abs();
          let b = if cells == 0 {
            piece::is_near_dir(p, t.2)
          } else {
            piece::is_sliding_dir(p, t.2)
          };
          if b {
            attacking_pieces.push(k);
            blocking_cells |= cells;
          }
        }
        _ => (),
      }
    }
    //knight checks
    for t in piece::KNIGHT_MOVES.iter() {
      let r = (king_row as isize) + t.0 * (s as isize);
      if r < 0 || r >= 9 {
        continue;
      }
      let c = (king_col as isize) + t.1 * (s as isize);
      if c < 0 || c >= 9 {
        continue;
      }
      let k = r as usize * 9 + c as usize;
      let piece = self.board[k];
      if s * piece >= 0 {
        continue;
      }
      if piece.abs() != piece::KNIGHT {
        continue;
      }
      attacking_pieces.push(k);
    }
    //double checks can't be blocked
    if attacking_pieces.len() > 1 {
      blocking_cells = 0;
    }
    Checks {
      attacking_pieces,
      blocking_cells,
      king_pos,
    }
  }
  fn find_king(&self, s: i8) -> Option<usize> {
    let king = piece::KING * s;
    self
      .board
      .iter()
      .enumerate()
      .find_map(|(i, v)| if *v == king { Some(i) } else { None })
  }
  fn find_checks(&self, s: i8) -> Checks {
    let king_pos = self.find_king(s);
    match king_pos {
      Some(king_pos) => self.checks(king_pos, s),
      None => Checks::default(),
    }
  }
  pub fn compute_checks(&self) -> Checks {
    self.find_checks(self.side)
  }
  pub fn is_legal(&self) -> bool {
    self.find_checks(-self.side).attacking_pieces.is_empty()
  }
  pub fn is_check(&self) -> bool {
    self.compute_checks().is_check()
  }
  pub fn is_double_check(&self) -> bool {
    self.compute_checks().is_double_check()
  }
  pub fn enumerate_moves(&self) -> Vec<Move> {
    let mut r = Vec::new();
    let c = self.find_checks(self.side);
    let l = c.attacking_pieces.len();
    //TODO: enumerate drops
    if l == 0 {
      //no check
      self.enumerate_simple_moves(|m| {
        r.push(m);
        false
      });
    } else if l == 1 {
      let p = c.attacking_pieces[0];
      self.enumerate_simple_moves(|m| {
        let b = c.blocking_cell(m.to);
        if (m.from_piece.abs() == piece::KING && !b) || b || m.to == p {
          r.push(m);
        }
        false
      });
    } else {
      assert_eq!(l, 2);
      self.enumerate_simple_moves(|m| {
        if m.from_piece.abs() == piece::KING {
          r.push(m);
        }
        false
      });
    }
    r
  }
  pub fn do_move(&mut self, m: &Move) -> UndoMove {
    if m.from != 0xff {
      self.board[m.from] = piece::NONE;
    }
    let taken_piece = self.board[m.to];
    self.board[m.to] = m.to_piece;
    self.move_no += 1;
    UndoMove { taken_piece }
  }
  pub fn undo_move(&mut self, m: &Move, u: &UndoMove) {
    self.board[m.to] = u.taken_piece;
    if m.from != 0xff {
      self.board[m.from] = m.from_piece;
    }
    self.move_no -= 1;
  }
}

#[test]
fn test_position_is_unblockable_check_false() {
  for sfen in vec![
    "8k/9/9/9/9/9/9/9/8L w - 1",
    "8k/r8/7K1/9/9/9/9/9/8L w - 1",
    "8k/9/7K1/9/9/9/9/+r8/8L w - 1",
  ] {
    let pos = Position::parse_sfen(&sfen).unwrap();
    let c = pos.compute_checks();
    assert_eq!(pos.is_unblockable_check(&c), false);
  }
}

#[test]
fn test_position_is_unblockable_check_true() {
  for sfen in vec![
    "8k/9/7K1/9/9/9/9/9/8L w - 1",
    "8k/9/7N1/9/9/9/9/+r8/8L w - 1",
  ] {
    let pos = Position::parse_sfen(&sfen).unwrap();
    let c = pos.compute_checks();
    assert_eq!(pos.is_unblockable_check(&c), true);
  }
}

impl Position {
  //helper method for unavoidable mate detection
  fn is_unblockable_check(&self, checks: &Checks) -> bool {
    let l = checks.attacking_pieces.len();
    if l != 1 {
      l == 2
    } else {
      let a = checks.attacking_pieces[0];
      let p = self.board[a];
      if !piece::sliding(p) {
        false
      } else {
        let (delta_row, delta_col) = cell::delta_direction(a, checks.king_pos);
        let delta = 9 * delta_row + delta_col;
        let mut cell = checks.king_pos;
        for k in 0.. {
          cell = ((cell as isize) + delta) as usize;
          if cell == a {
            break;
          }
          assert!(checks.blocking_cell(cell));
          let d = if k == 0 { 2 } else { 1 };
          if self.checks(cell, -self.side).attacking_pieces.len() >= d {
            return false;
          }
          if k == 0 && self.checks(cell, self.side).attacking_pieces.len() < 2 {
            return false;
          }
        }
        true
      }
    }
  }
}

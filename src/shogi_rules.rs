use std::fmt;
use std::str::FromStr;

use crate::bits;

mod cell;
mod piece;

pub struct Position {
  board: [i8; 81],
  black_pockets: [u8; 8],
  white_pockets: [u8; 8],
  nifu_masks: u32,
  move_no: u32,
  side: i8,
}

#[derive(Clone, Default, Debug)]
pub struct Move {
  from: usize,
  to: usize,
  from_piece: i8,
  to_piece: i8,
}

impl Move {
  pub fn is_pawn_drop(&self) -> bool {
    self.from_piece == piece::NONE && self.to_piece.abs() == piece::PAWN
  }
}

struct SlidingIterator {
  delta: isize,
  last: usize,
  end: usize,
  drops_mask: u32,
}

impl Iterator for SlidingIterator {
  type Item = (usize, u32);
  fn next(&mut self) -> Option<Self::Item> {
    let a = (self.last as isize + self.delta) as usize;
    if a == self.end {
      None
    } else {
      self.last = a;
      Some((a, self.drops_mask))
    }
  }
}

impl SlidingIterator {
  fn new(attacking_piece: usize, king_pos: usize, drops_mask: u32) -> Self {
    let (delta_row, delta_col) = cell::delta_direction(attacking_piece, king_pos);
    SlidingIterator {
      delta: 9 * delta_row + delta_col,
      last: king_pos,
      end: attacking_piece,
      drops_mask,
    }
  }
}

#[derive(Clone)]
pub struct Checks {
  pub blocking_cells: u128,
  pub attacking_pieces: Vec<usize>,
  king_pos: Option<usize>,
}

impl Default for Checks {
  fn default() -> Self {
    Checks {
      blocking_cells: 0,
      attacking_pieces: Vec::new(),
      king_pos: None,
    }
  }
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

struct PotentialDropsMap(Vec<(usize, u32)>);

impl Default for PotentialDropsMap {
  fn default() -> Self {
    Self(Vec::new())
  }
}

impl PotentialDropsMap {
  fn insert(&mut self, cell: usize, mask: u32) {
    self.0.push((cell, mask));
  }
}

impl IntoIterator for PotentialDropsMap {
  type Item = (usize, u32);
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

#[derive(Debug)]
pub struct ParseSFENError {
  sfen: String,
  message: String,
}

impl fmt::Display for ParseSFENError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}, SFEN: \"{}\"", self.message, self.sfen)
  }
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
  pub fn move_to_string(&self, m: &Move, moves: &Vec<Move>) -> String {
    let mut s = String::new();
    let p = m.from_piece;
    if p != piece::NONE {
      s.push_str(&piece::to_string(p, true));
      let f = moves
        .iter()
        .find(|q| m.from_piece == q.from_piece && m.to == q.to && m.from != q.from);
      match f {
        Some(n) => {
          let (row1, col1) = cell::unpack(m.from);
          let (row2, col2) = cell::unpack(n.from);
          if col1 != col2 {
            s.push(('0' as u8 + col2 as u8) as char);
          } else {
            assert_ne!(row1, row2);
            s.push(('a' as u8 + row2 as u8) as char);
          }
        }
        _ => (),
      }
      if self.board[m.to] != piece::NONE {
        s.push('x');
      }
    } else {
      s.push_str(&piece::to_string(m.to_piece, true));
      s.push('\'');
    }
    s.push_str(&cell::to_string(m.to));
    if p != piece::NONE {
      if p != m.to_piece {
        s.push('+');
      } else {
        let a = p.abs();
        if a != piece::KING && a != piece::GOLD {
          let side = p.signum();
          if cell::promotion_zone(m.from, side) || cell::promotion_zone(m.to, side) {
            s.push('=');
          }
        }
      }
    }
    s
  }
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
    let mut nifu_masks = 0u32;
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
              format!(
                "double promotion in cell {}",
                cell::to_string(9 * row + 8 - col)
              ),
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
              format!(
                "invalid piece in cell {}",
                cell::to_string(9 * row + 8 - col)
              ),
            ));
          }
          if promoted != 0 {
            if p == piece::KING {
              return Err(ParseSFENError::new(
                sfen,
                format!(
                  "promoted king in cell {}",
                  cell::to_string(9 * row + 8 - col)
                ),
              ));
            }
            if p == piece::GOLD {
              return Err(ParseSFENError::new(
                sfen,
                format!(
                  "promoted gold general in cell {}",
                  cell::to_string(9 * row + 8 - col)
                ),
              ));
            }
          } else if p.abs() == piece::PAWN {
            let bit = 1u32 << (((p.signum() as i32 + 1) << 3) + (8 - col as i32));
            if (nifu_masks & bit) != 0 {
              return Err(ParseSFENError::new(
                sfen,
                format!(
                  "more than one {} pawn in column {}",
                  piece::color(p),
                  9 - col
                ),
              ));
            }
            nifu_masks |= bit;
          }
          board[9 * row + (8 - col)] = p + promoted * p.signum();
          promoted = 0;
          col += 1;
        }
      }
    }
    //check pawns and knights in promotion zone
    for row in (0..3).chain(6..9) {
      for (c, p) in board.iter().enumerate().skip(9 * row).take(9) {
        if !piece::is_promoted(*p) && !piece::could_unpromoted(*p, c) {
          return Err(ParseSFENError::new(
            sfen,
            format!(
              "unpromoted {} on the {} row at cell {}",
              if p.abs() == piece::PAWN {
                "pawn"
              } else {
                "knight"
              },
              row + 1,
              cell::to_string(c)
            ),
          ));
        }
      }
    }
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
      nifu_masks,
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
      let k = 9 * row + col;
      let t = piece * self.board[k].signum();
      if t > 0 {
        break;
      }
      if piece::is_promoted(piece) || piece::could_unpromoted(piece, k) {
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
  fn empty_cells_with_drop_mask(&self, drop_mask: u32) -> Vec<(usize, u32)> {
    self
      .board
      .iter()
      .enumerate()
      .filter_map(|(i, p)| {
        if *p == piece::NONE {
          Some((i, drop_mask))
        } else {
          None
        }
      })
      .collect()
  }
  fn compute_drops_mask(&self) -> u32 {
    let q = if self.side > 0 {
      &self.black_pockets
    } else {
      &self.white_pockets
    };
    q.iter()
      .enumerate()
      .skip(1)
      .filter(|&(_, c)| *c > 0)
      .fold(0, |acc, (i, _)| acc + (1 << i))
  }
  pub fn compute_drops_with_check(&self) -> Vec<Move> {
    let drops_mask = self.compute_drops_mask();
    let mut r = Vec::new();
    self.enumerate_drops(
      |m| {
        r.push(m);
        false
      },
      self.compute_potential_drops_map(drops_mask).into_iter(),
    );
    r
  }
  fn enumerate_drops<F: FnMut(Move) -> bool, I: Iterator<Item = (usize, u32)>>(
    &self,
    mut f: F,
    drop_masks_iterator: I,
  ) -> bool {
    let drops_mask = self.compute_drops_mask();
    for (k, mask) in drop_masks_iterator {
      let mask = mask & drops_mask;
      if mask == 0 {
        continue;
      }
      for p in bits::Bits(mask) {
        let p = p as i8;
        if p == piece::PAWN {
          let col = k % 9;
          let bit = 1u32 << (((1 + self.side as i32) << 3) + col as i32);
          if (self.nifu_masks & bit) != 0 {
            continue;
          }
        }
        let to_piece = p * self.side;
        if !piece::could_unpromoted(to_piece, k) {
          continue;
        }
        let m = Move {
          from: 0xff,
          to: k,
          from_piece: piece::NONE,
          to_piece,
        };
        if f(m) {
          return true;
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
        let k = 9 * row + col;
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
      let k = 9 * r as usize + c as usize;
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
      king_pos: Some(king_pos),
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
  fn compute_potential_drops_map(&self, drops_mask: u32) -> PotentialDropsMap {
    let mut m = PotentialDropsMap::default();
    let king_pos = self.find_king(-self.side);
    if king_pos.is_none() {
      return m;
    }
    let king_pos = king_pos.unwrap();
    let (king_row, king_col) = cell::unpack(king_pos);
    for t in if self.side < 0 {
      piece::BLACK_DIRECTIONS.iter()
    } else {
      piece::WHITE_DIRECTIONS.iter()
    } {
      let mut row = king_row;
      let mut col = king_col;
      let mut mask = piece::near_dir_to_mask(t.2) & drops_mask;
      if mask == 0 {
        continue;
      }
      for steps in 0.. {
        let r = (row as isize) + t.0;
        if r < 0 || r >= 9 {
          break;
        }
        let c = (col as isize) + t.1;
        if c < 0 || c >= 9 {
          break;
        }
        if steps == 1 {
          mask = piece::sliding_dir_to_mask(t.2) & drops_mask;
          if mask == 0 {
            break;
          }
        }
        row = r as usize;
        col = c as usize;
        let k = 9 * row + col;
        if self.board[k] != piece::NONE {
          break;
        }
        m.insert(k, mask);
      }
    }
    //knight checks
    let knight_bit = 1u32 << piece::KNIGHT;
    if (drops_mask & knight_bit) == 0 {
      return m;
    }
    for t in piece::KNIGHT_MOVES.iter() {
      let r = (king_row as isize) - t.0 * (self.side as isize);
      if r < 0 || r >= 9 {
        continue;
      }
      let c = (king_col as isize) - t.1 * (self.side as isize);
      if c < 0 || c >= 9 {
        continue;
      }
      let k = 9 * r as usize + c as usize;
      if self.board[k] != piece::NONE {
        continue;
      }
      m.insert(k, knight_bit);
    }
    m
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
  pub fn is_take(&self, m: &Move) -> bool {
    self.board[m.to] != piece::NONE
  }
  pub fn compute_drops(&self, checks: &Checks) -> Vec<Move> {
    let mut r = Vec::new();
    match checks.attacking_pieces.len() {
      0 => {
        self.enumerate_drops(
          |m| {
            r.push(m);
            false
          },
          self.empty_cells_with_drop_mask(0x7fff_ffff).into_iter(),
        );
      }
      1 => {
        if checks.blocking_cells != 0 {
          self.enumerate_drops(
            |m| {
              r.push(m);
              false
            },
            SlidingIterator::new(
              checks.attacking_pieces[0],
              checks.king_pos.unwrap(),
              0x7fff_ffff,
            ),
          );
        }
      }
      2 => {
        //drops are impossible
      }
      _ => panic!("too many attacking pieces"),
    }
    r
  }
  fn validate_checks(&self, checks: &Checks) -> bool {
    match checks.king_pos {
      Some(king_pos) => self.board[king_pos] == piece::KING * self.side,
      None => true,
    }
  }
  pub fn compute_moves(&self, checks: &Checks) -> Vec<Move> {
    let mut r = Vec::new();
    assert!(self.validate_checks(checks));
    match checks.attacking_pieces.len() {
      0 => {
        //no check
        self.enumerate_simple_moves(|m| {
          r.push(m);
          false
        });
      }
      1 => {
        let p = checks.attacking_pieces[0];
        self.enumerate_simple_moves(|m| {
          let b = checks.blocking_cell(m.to);
          if (m.from_piece.abs() == piece::KING && !b) || b || m.to == p {
            r.push(m);
          }
          false
        });
      }
      2 => {
        self.enumerate_simple_moves(|m| {
          if m.from_piece.abs() == piece::KING {
            r.push(m);
          }
          false
        });
      }
      _ => panic!("too many attacking pieces"),
    }
    r
  }
  //TODO: incremental drop_masks
  pub fn do_move(&mut self, m: &Move) -> UndoMove {
    if m.from != 0xff {
      self.board[m.from] = piece::NONE;
    } else {
      if m.to_piece > 0 {
        self.black_pockets[m.to_piece as usize] -= 1;
      } else {
        self.white_pockets[(-m.to_piece) as usize] -= 1;
      }
    }
    if m.from_piece != m.to_piece {
      if m.from_piece.abs() == piece::PAWN || m.to_piece.abs() == piece::PAWN {
        self.nifu_masks ^= 1u32 << (((1 + self.side as i32) << 3) + (m.to % 9) as i32);
      }
    }
    let taken_piece = self.board[m.to];
    if taken_piece != piece::NONE {
      if taken_piece.abs() == piece::PAWN {
        self.nifu_masks ^= 1u32 << (((1 - self.side as i32) << 3) + (m.to % 9) as i32);
      }
      let p = piece::unpromote(taken_piece);
      if p > 0 {
        self.white_pockets[p as usize] += 1;
      } else {
        self.black_pockets[(-p) as usize] += 1;
      }
    }
    self.board[m.to] = m.to_piece;
    self.move_no += 1;
    self.side *= -1;
    UndoMove { taken_piece }
  }
  pub fn undo_move(&mut self, m: &Move, u: &UndoMove) {
    self.board[m.to] = u.taken_piece;
    if m.from != 0xff {
      self.board[m.from] = m.from_piece;
    } else {
      if m.to_piece > 0 {
        self.black_pockets[m.to_piece as usize] += 1;
      } else {
        self.white_pockets[(-m.to_piece) as usize] += 1;
      }
    }
    if m.from_piece != m.to_piece {
      if m.from_piece.abs() == piece::PAWN || m.to_piece.abs() == piece::PAWN {
        self.nifu_masks ^= 1u32 << (((1 - self.side as i32) << 3) + (m.to % 9) as i32);
      }
    }
    if u.taken_piece != piece::NONE {
      if u.taken_piece.abs() == piece::PAWN {
        self.nifu_masks ^= 1u32 << (((1 + self.side as i32) << 3) + (m.to % 9) as i32);
      }
      let p = piece::unpromote(u.taken_piece);
      if p > 0 {
        self.white_pockets[p as usize] -= 1;
      } else {
        self.black_pockets[(-p) as usize] -= 1;
      }
    }
    self.move_no -= 1;
    self.side *= -1;
  }
  pub fn do_san_move(&mut self, san: &str) -> bool {
    let checks = self.compute_checks();
    let moves = self.compute_moves(&checks);
    for m in &moves {
      if san == self.move_to_string(&m, &moves) {
        self.do_move(m);
        return true;
      }
    }
    let drops = self.compute_drops(&checks);
    for m in drops {
      if san == self.move_to_string(&m, &moves) {
        self.do_move(&m);
        return true;
      }
    }
    false
  }
  pub fn is_futile_drop(&mut self, checks: &Checks, drop: &Move) -> bool {
    let attacking_piece = checks.attacking_pieces[0];
    let mut res = true;
    //let u1 = self.do_move(&drop);
    let p = self.board[attacking_piece];
    let take_move = Move {
      from: attacking_piece,
      to: drop.to,
      from_piece: p,
      to_piece: p,
    };
    let u2 = self.do_move(&take_move);
    assert!(self.is_legal(), "SFEN: {}", self);
    let moves = self.compute_moves(&self.compute_checks());
    for m in moves {
      let u3 = self.do_move(&m);
      if self.is_legal() {
        //no mate
        res = false;
        self.undo_move(&m, &u3);
        break;
      }
      self.undo_move(&m, &u3);
    }
    self.undo_move(&take_move, &u2);
    //self.undo_move(&drop, &u1);
    res
  }
}

impl fmt::Display for Position {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for row in 0..9 {
      if row > 0 {
        write!(f, "/")?;
      }
      let mut cnt = 0;
      for c in self.board.iter().skip(9 * row).take(9).rev() {
        if *c == piece::NONE {
          cnt += 1;
        } else {
          if cnt > 0 {
            write!(f, "{}", cnt)?;
            cnt = 0;
          }
          write!(f, "{}", piece::to_string(*c, true))?;
        }
      }
      if cnt > 0 {
        write!(f, "{}", cnt)?;
      }
    }
    write!(f, " {} ", if self.side > 0 { 'b' } else { 'w' })?;
    let mut t = 0u32;
    for (&k, c) in self.black_pockets[piece::PAWN as usize..piece::KING as usize]
      .iter()
      .zip(piece::PIECE_TO_CHAR.chars())
    {
      if k > 0 {
        write!(f, "{}", c.to_ascii_uppercase())?;
        t += k as u32;
      }
    }
    for (&k, c) in self.white_pockets[piece::PAWN as usize..piece::KING as usize]
      .iter()
      .zip(piece::PIECE_TO_CHAR.chars())
    {
      if k > 0 {
        write!(f, "{}", c)?;
        t += k as u32;
      }
    }
    if t == 0 {
      write!(f, "-")?;
    }
    write!(f, " {}", self.move_no)
  }
}

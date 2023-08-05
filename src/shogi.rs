use crate::bits;
use std::fmt;
use std::str::FromStr;

pub mod attacking_pieces;
mod bitboards;
mod board;
mod cell;
mod consts;
mod direction;
pub mod game;
mod hash;
pub mod kif;
pub mod moves;
pub mod piece;

use moves::Move;

#[derive(Clone)]
pub struct Position {
  board: [i8; 81],
  black_pockets: [u8; 8],
  white_pockets: [u8; 8],
  black_king_position: Option<usize>,
  white_king_position: Option<usize>,
  all_pieces: u128,
  all_pieces2: u128,
  all_pieces3: u128,
  all_pieces4: u128,
  pub hash: u64,
  pub move_no: u32,
  drop_masks: u32,
  nifu_masks: u32,
  pub side: i8,
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
    let (delta_row, delta_col) = direction::delta_direction(king_pos, attacking_piece);
    SlidingIterator {
      delta: 9 * delta_row + delta_col,
      last: king_pos,
      end: attacking_piece,
      drops_mask,
    }
  }
}

pub struct Checks {
  pub blocking_cells: u128,
  pub attacking_pieces: attacking_pieces::AttackingPieces,
  king_pos: Option<usize>,
  hash: u64,
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

fn compute_drops_mask(q: &[u8]) -> u32 {
  q.iter()
    .enumerate()
    .skip(1)
    .filter(|&(_, c)| *c > 0)
    .fold(0, |acc, (i, _)| acc + (1 << i))
}

fn decrement_pocket(q: &mut u8) -> bool {
  *q -= 1;
  *q == 0
}

fn increment_pocket(q: &mut u8) -> bool {
  *q += 1;
  *q == 1
}

fn compute_hash(board: &[i8], black_pockets: &[u8], white_pockets: &[u8], side: i8) -> u64 {
  let mut res = board::compute_hash(&board)
    ^ hash::compute_black_pockets_hash(&black_pockets)
    ^ hash::compute_white_pockets_hash(&white_pockets);
  if side < 0 {
    res = !res;
  }
  res
}

fn swap_words(x: u32) -> u32 {
  let lo = x & 0xffff;
  let hi = x >> 16;
  (lo << 16) + hi
}

fn nify_mask_reverse(x: u32) -> u32 {
  bits::Bits(x).into_iter().fold(0, |acc, i| acc + (256 >> i))
}

impl Position {
  fn compute_hash(&self) -> u64 {
    compute_hash(
      &self.board,
      &self.black_pockets,
      &self.white_pockets,
      self.side,
    )
  }
  pub fn swap_sides(&mut self) {
    for i in 0..9 * 4 + 4 {
      let j = cell::mirror(i);
      let t = -self.board[i];
      self.board[i] = -self.board[j];
      self.board[j] = t;
    }
    self.board[9 * 4 + 4] *= -1;
    let t = self.black_pockets;
    self.black_pockets = self.white_pockets;
    self.white_pockets = t;
    let t = self.black_king_position.map(cell::mirror);
    self.black_king_position = self.white_king_position.map(cell::mirror);
    self.white_king_position = t;
    self.drop_masks = swap_words(self.drop_masks);
    let lo = self.nifu_masks & 0xffff;
    let hi = self.nifu_masks >> 16;
    self.nifu_masks = (nify_mask_reverse(lo) << 16) + nify_mask_reverse(hi);
    self.side *= -1;
    self.hash = self.compute_hash();
    let (m1, m2, m3, m4) = board::compute_all_pieces(&self.board);
    self.all_pieces = m1;
    self.all_pieces2 = m2;
    self.all_pieces3 = m3;
    self.all_pieces4 = m4;
    debug_assert_eq!(
      self.black_king_position,
      board::find_king_position(&self.board, 1)
    );
    debug_assert_eq!(
      self.white_king_position,
      board::find_king_position(&self.board, -1)
    );
  }
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
    let (black_pieces, white_pieces) = board::count_pieces(&board);
    if black_pieces[piece::KING as usize] > 1 {
      return Err(ParseSFENError::new(
        sfen,
        String::from("too many black kings"),
      ));
    }
    if white_pieces[piece::KING as usize] > 1 {
      return Err(ParseSFENError::new(
        sfen,
        String::from("too many white kings"),
      ));
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
    for p in piece::PAWN..piece::KING {
      let e = piece::expected_number_of_pieces(p);
      let p = p as usize;
      let t = black_pieces[p] + white_pieces[p] + black_pockets[p] as u32 + white_pockets[p] as u32;
      if t > e {
        return Err(ParseSFENError::new(
          sfen,
          format!(
            "{} {}, expected number of theese pieces are {}",
            t,
            piece::to_human_string(p as i8),
            e
          ),
        ));
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
    let hash = compute_hash(&board, &black_pockets, &white_pockets, side);
    let (all_pieces, all_pieces2, all_pieces3, all_pieces4) = board::compute_all_pieces(&board);
    let pos = Position {
      board,
      black_pockets,
      white_pockets,
      black_king_position: board::find_king_position(&board, 1),
      white_king_position: board::find_king_position(&board, -1),
      all_pieces,
      all_pieces2,
      all_pieces3,
      all_pieces4,
      hash,
      drop_masks: compute_drops_mask(&black_pockets) | (compute_drops_mask(&white_pockets) << 16),
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
        if f(Move {
          from: pos,
          to: k,
          from_piece: piece,
          to_piece: piece,
        }) {
          return true;
        }
      }
      if piece::could_promoted(piece) && (p || cell::promotion_zone(k, self.side)) {
        if f(Move {
          from: pos,
          to: k,
          from_piece: piece,
          to_piece: piece + piece.signum() * piece::PROMOTED,
        }) {
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
      match v {
        piece::PAWN => {
          if self.enumerate_piece_move(&mut f, pos, v, -1, 0, false) {
            return true;
          }
        }
        piece::WHITE_PAWN => {
          if self.enumerate_piece_move(&mut f, pos, v, 1, 0, false) {
            return true;
          }
        }
        piece::LANCE => {
          if self.enumerate_piece_move(&mut f, pos, v, -1, 0, true) {
            return true;
          }
        }
        piece::WHITE_LANCE => {
          if self.enumerate_piece_move(&mut f, pos, v, 1, 0, true) {
            return true;
          }
        }
        piece::KNIGHT => {
          if self.enumerate_piece_move(&mut f, pos, v, -2, -1, false) {
            return true;
          }
          if self.enumerate_piece_move(&mut f, pos, v, -2, 1, false) {
            return true;
          }
        }
        piece::WHITE_KNIGHT => {
          if self.enumerate_piece_move(&mut f, pos, v, 2, -1, false) {
            return true;
          }
          if self.enumerate_piece_move(&mut f, pos, v, 2, 1, false) {
            return true;
          }
        }
        piece::SILVER => {
          for t in piece::SILVER_MOVES.iter() {
            if self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false) {
              return true;
            }
          }
        }
        piece::WHITE_SILVER => {
          for t in piece::SILVER_MOVES.iter() {
            if self.enumerate_piece_move(&mut f, pos, v, -t.0, t.1, false) {
              return true;
            }
          }
        }
        piece::GOLD
        | piece::PROMOTED_PAWN
        | piece::PROMOTED_LANCE
        | piece::PROMOTED_KNIGHT
        | piece::PROMOTED_SILVER => {
          for t in piece::GOLD_MOVES.iter() {
            if self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false) {
              return true;
            }
          }
        }
        piece::WHITE_GOLD
        | piece::WHITE_PROMOTED_PAWN
        | piece::WHITE_PROMOTED_LANCE
        | piece::WHITE_PROMOTED_KNIGHT
        | piece::WHITE_PROMOTED_SILVER => {
          for t in piece::GOLD_MOVES.iter() {
            if self.enumerate_piece_move(&mut f, pos, v, -t.0, t.1, false) {
              return true;
            }
          }
        }
        piece::KING | piece::WHITE_KING => {
          for t in piece::KING_MOVES.iter() {
            if self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false) {
              return true;
            }
          }
        }
        _ => {
          if w == piece::BISHOP || w == -piece::BISHOP {
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
  fn is_discover_check_piece(&self, from: usize, opponent_king_pos: usize) -> bool {
    if let Some(i) = direction::try_to_find_delta_direction_no(opponent_king_pos, from) {
      let b = consts::SLIDING_MASKS[8 * opponent_king_pos + i] & self.all_pieces;
      debug_assert_ne!(b, 0);
      let k = bitboards::scan(b, i);
      if k != from {
        return false;
      }
      let b = b ^ (1u128 << from);
      if b == 0 {
        return false;
      }
      let k = bitboards::scan(b, i);
      let t = self.board[k];
      if t * self.side < 0 {
        return false;
      }
      let flags = if self.side < 0 {
        piece::BLACK_DIRECTIONS_FLAGS[i]
      } else {
        piece::WHITE_DIRECTIONS_FLAGS[i]
      };
      if piece::is_sliding_dir(t.abs(), flags) {
        return true;
      }
    }
    false
  }
  fn promoted_sliding_piece_checks<F: FnMut(Move) -> bool>(
    &self,
    a: u128,
    from: usize,
    v: i8,
    f: &mut F,
  ) {
    for k in bitboards::Bits128(a) {
      let t = self.board[k];
      if t * self.side > 0 {
        continue;
      }
      f(Move {
        from,
        to: k,
        from_piece: v,
        to_piece: v,
      });
    }
  }
  fn sliding_piece_checks<F: FnMut(Move) -> bool>(
    &self,
    a: u128,
    b: u128,
    from: usize,
    opponent_king_pos: usize,
    v: i8,
    f: &mut F,
  ) {
    let (promoted_moves, not_promoted_moves) = if cell::promotion_zone(from, self.side) {
      (a, 0)
    } else {
      let t = a & bitboards::promotion_zone(self.side);
      (t, a ^ t)
    };
    if promoted_moves != 0 {
      let c = consts::KING_MASKS[opponent_king_pos];
      for k in bitboards::Bits128(promoted_moves & (b | c)) {
        let t = self.board[k];
        if t * self.side > 0 {
          continue;
        }
        f(Move {
          from,
          to: k,
          from_piece: v,
          to_piece: v + self.side * piece::PROMOTED,
        });
        let bit = 1u128 << k;
        if (b & bit) != 0 {
          f(Move {
            from,
            to: k,
            from_piece: v,
            to_piece: v,
          });
        }
      }
    }
    for k in bitboards::Bits128(not_promoted_moves & b) {
      let t = self.board[k];
      if t * self.side > 0 {
        continue;
      }
      f(Move {
        from,
        to: k,
        from_piece: v,
        to_piece: v,
      });
    }
  }

  pub fn compute_check_candidates(&self, checks: &Checks) -> Vec<Move> {
    if checks.is_check() {
      return self.compute_moves(checks);
    }
    let mut moves = Vec::new();
    let mut f = |mv| {
      moves.push(mv);
      false
    };
    let opponent_king_pos = self.find_king_position(-self.side);
    if opponent_king_pos.is_none() {
      return moves;
    }
    let opponent_king_pos = opponent_king_pos.unwrap();
    let (king_row, king_col) = cell::unpack(opponent_king_pos);
    for (pos, &v) in self.board.iter().enumerate() {
      if self.side * v <= 0 {
        continue;
      }
      match v.abs() {
        piece::PAWN => {
          if (opponent_king_pos as isize != pos as isize - 18 * self.side as isize
            && !cell::pawn_could_attack_king_by_move_with_promotion(
              (pos as isize - 9 * self.side as isize) as usize,
              self.side,
              king_row,
              king_col,
            ))
            && !self.is_discover_check_piece(pos, opponent_king_pos)
          {
            continue;
          }
        }
        piece::LANCE => {
          if (self.side > 0) != (opponent_king_pos < pos) {
            let delta = if cell::promotion_zone(opponent_king_pos, self.side) {
              1
            } else {
              0
            };
            let col = pos % 9;
            if (king_col as isize - col as isize).abs() <= delta
              && !self.is_discover_check_piece(pos, opponent_king_pos)
            {
              continue;
            }
          }
        }
        piece::KNIGHT => {
          let (knight_row, knight_col) = cell::unpack(pos);
          if !cell::knight_could_attack_king_after_move(
            knight_row, knight_col, self.side, king_row, king_col,
          ) && !self.is_discover_check_piece(pos, opponent_king_pos)
          {
            continue;
          }
        }
        piece::KING => {
          if !self.is_discover_check_piece(pos, opponent_king_pos) {
            continue;
          }
        }
        piece::SILVER => {
          if !self.is_discover_check_piece(pos, opponent_king_pos) {
            let a = if self.side > 0 {
              consts::BLACK_SILVER_MASKS[pos]
            } else {
              consts::WHITE_SILVER_MASKS[pos]
            };
            let (promoted_moves, not_promoted_moves) = if cell::promotion_zone(pos, self.side) {
              (a, 0)
            } else {
              let t = a & bitboards::promotion_zone(self.side);
              (t, a ^ t)
            };
            let promoted_moves = promoted_moves & consts::KING_MASKS[opponent_king_pos];
            if promoted_moves != 0 {
              let (b, c) = if self.side > 0 {
                (
                  consts::WHITE_SILVER_MASKS[opponent_king_pos],
                  consts::WHITE_GOLD_MASKS[opponent_king_pos],
                )
              } else {
                (
                  consts::BLACK_SILVER_MASKS[opponent_king_pos],
                  consts::BLACK_GOLD_MASKS[opponent_king_pos],
                )
              };
              for k in bitboards::Bits128(promoted_moves) {
                let t = self.board[k];
                if t * self.side > 0 {
                  continue;
                }
                let bit = 1u128 << k;
                if (c & bit) != 0 {
                  f(Move {
                    from: pos,
                    to: k,
                    from_piece: v,
                    to_piece: v + self.side * piece::PROMOTED,
                  });
                }
                if (b & bit) != 0 {
                  f(Move {
                    from: pos,
                    to: k,
                    from_piece: v,
                    to_piece: v,
                  });
                }
              }
            }
            let not_promoted_moves = not_promoted_moves
              & (if self.side > 0 {
                consts::WHITE_SILVER_MASKS[opponent_king_pos]
              } else {
                consts::BLACK_SILVER_MASKS[opponent_king_pos]
              });

            for k in bitboards::Bits128(not_promoted_moves) {
              let t = self.board[k];
              if t * self.side > 0 {
                continue;
              }
              f(Move {
                from: pos,
                to: k,
                from_piece: v,
                to_piece: v,
              });
            }
            continue;
          }
        }
        piece::GOLD
        | piece::PROMOTED_PAWN
        | piece::PROMOTED_LANCE
        | piece::PROMOTED_KNIGHT
        | piece::PROMOTED_SILVER => {
          if !self.is_discover_check_piece(pos, opponent_king_pos) {
            let c = if self.side > 0 {
              consts::BLACK_GOLD_MASKS[pos] & consts::WHITE_GOLD_MASKS[opponent_king_pos]
            } else {
              consts::WHITE_GOLD_MASKS[pos] & consts::BLACK_GOLD_MASKS[opponent_king_pos]
            };
            for k in bitboards::Bits128(c) {
              let t = self.board[k];
              if t * self.side > 0 {
                continue;
              }
              f(Move {
                from: pos,
                to: k,
                from_piece: v,
                to_piece: v,
              });
            }
            continue;
          }
        }
        piece::PROMOTED_ROOK => {
          if !self.is_discover_check_piece(pos, opponent_king_pos) {
            let a =
              bitboards::rook(pos, self.all_pieces, self.all_pieces2) | consts::KING_MASKS[pos];
            let b = bitboards::rook(opponent_king_pos, self.all_pieces, self.all_pieces2)
              | consts::KING_MASKS[opponent_king_pos];
            self.promoted_sliding_piece_checks(a & b, pos, v, &mut f);
            continue;
          }
        }
        piece::PROMOTED_BISHOP => {
          if !self.is_discover_check_piece(pos, opponent_king_pos) {
            let a =
              bitboards::bishop(pos, self.all_pieces3, self.all_pieces4) | consts::KING_MASKS[pos];
            let b = bitboards::bishop(opponent_king_pos, self.all_pieces3, self.all_pieces4)
              | consts::KING_MASKS[opponent_king_pos];
            self.promoted_sliding_piece_checks(a & b, pos, v, &mut f);
            continue;
          }
        }
        piece::ROOK => {
          if !self.is_discover_check_piece(pos, opponent_king_pos) {
            let a = bitboards::rook(pos, self.all_pieces, self.all_pieces2);
            let b = bitboards::rook(opponent_king_pos, self.all_pieces, self.all_pieces2);
            self.sliding_piece_checks(a, b, pos, opponent_king_pos, v, &mut f);
            continue;
          }
        }
        piece::BISHOP => {
          if !self.is_discover_check_piece(pos, opponent_king_pos) {
            let a = bitboards::bishop(pos, self.all_pieces3, self.all_pieces4);
            let b = bitboards::bishop(opponent_king_pos, self.all_pieces3, self.all_pieces4);
            self.sliding_piece_checks(a, b, pos, opponent_king_pos, v, &mut f);
            continue;
          }
        }
        _ => (),
      }
      let w = piece::unpromote(v);
      match v {
        piece::PAWN => {
          self.enumerate_piece_move(&mut f, pos, v, -1, 0, false);
        }
        piece::WHITE_PAWN => {
          self.enumerate_piece_move(&mut f, pos, v, 1, 0, false);
        }
        piece::LANCE => {
          self.enumerate_piece_move(&mut f, pos, v, -1, 0, true);
        }
        piece::WHITE_LANCE => {
          self.enumerate_piece_move(&mut f, pos, v, 1, 0, true);
        }
        piece::KNIGHT => {
          self.enumerate_piece_move(&mut f, pos, v, -2, -1, false);
          self.enumerate_piece_move(&mut f, pos, v, -2, 1, false);
        }
        piece::WHITE_KNIGHT => {
          self.enumerate_piece_move(&mut f, pos, v, 2, -1, false);
          self.enumerate_piece_move(&mut f, pos, v, 2, 1, false);
        }
        piece::SILVER => {
          for t in piece::SILVER_MOVES.iter() {
            self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false);
          }
        }
        piece::WHITE_SILVER => {
          for t in piece::SILVER_MOVES.iter() {
            self.enumerate_piece_move(&mut f, pos, v, -t.0, t.1, false);
          }
        }
        piece::GOLD
        | piece::PROMOTED_PAWN
        | piece::PROMOTED_LANCE
        | piece::PROMOTED_KNIGHT
        | piece::PROMOTED_SILVER => {
          for t in piece::GOLD_MOVES.iter() {
            self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false);
          }
        }
        piece::WHITE_GOLD
        | piece::WHITE_PROMOTED_PAWN
        | piece::WHITE_PROMOTED_LANCE
        | piece::WHITE_PROMOTED_KNIGHT
        | piece::WHITE_PROMOTED_SILVER => {
          for t in piece::GOLD_MOVES.iter() {
            self.enumerate_piece_move(&mut f, pos, v, -t.0, t.1, false);
          }
        }
        piece::KING | piece::WHITE_KING => {
          for t in piece::KING_MOVES.iter() {
            self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false);
          }
        }
        _ => {
          if w == piece::BISHOP || w == -piece::BISHOP {
            for t in piece::BISHOP_MOVES.iter() {
              self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, true);
            }
          } else if w == piece::ROOK || w == -piece::ROOK {
            for t in piece::ROOK_MOVES.iter() {
              self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, true);
            }
          }
        }
      }
      //promoted
      if v != w {
        if w == piece::BISHOP || w == -piece::BISHOP {
          for t in piece::ROOK_MOVES.iter() {
            self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false);
          }
        } else if w == piece::ROOK || w == -piece::ROOK {
          for t in piece::BISHOP_MOVES.iter() {
            self.enumerate_piece_move(&mut f, pos, v, t.0, t.1, false);
          }
        }
      }
    }
    moves
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
    if self.side > 0 {
      self.drop_masks & 0xffff
    } else {
      self.drop_masks >> 16
    }
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
        if f(Move {
          from: 0xff,
          to: k,
          from_piece: piece::NONE,
          to_piece,
        }) {
          return true;
        }
      }
    }
    false
  }
  fn attacked_by_sliding_piece_in_given_direction_no(
    &self,
    king_pos: usize,
    s: i8,
    i: usize,
  ) -> bool {
    let flags = if s > 0 {
      piece::BLACK_DIRECTIONS_FLAGS[i]
    } else {
      piece::WHITE_DIRECTIONS_FLAGS[i]
    };
    let p = consts::SLIDING_MASKS[8 * king_pos + i];
    let b = p & self.all_pieces;
    if b == 0 {
      return false;
    }
    let k = bitboards::scan(b, i);
    let piece = self.board[k];
    let t = s * piece;
    debug_assert_ne!(t, 0);
    if t < 0 {
      let pa = piece.abs();
      if piece::is_sliding_dir(pa, flags) {
        return true;
      }
    }
    false
  }
  fn attacked(&self, king_pos: usize, s: i8) -> bool {
    let (king_row, king_col) = cell::unpack(king_pos);
    for (i, (flags, p)) in if s > 0 {
      piece::BLACK_DIRECTIONS_FLAGS.iter()
    } else {
      piece::WHITE_DIRECTIONS_FLAGS.iter()
    }
    .zip(consts::SLIDING_MASKS.iter().skip(8 * king_pos))
    .enumerate()
    {
      let b = *p & self.all_pieces;
      if b != 0 {
        let k = bitboards::scan(b, i);
        let piece = self.board[k];
        let t = s * piece;
        debug_assert_ne!(t, 0);
        if t < 0 {
          let pa = piece.abs();
          let (row, col) = cell::unpack(k);
          let ok = if (king_row as isize - row as isize).abs() <= 1
            && (king_col as isize - col as isize).abs() <= 1
          {
            piece::is_near_dir(pa, *flags)
          } else {
            piece::is_sliding_dir(pa, *flags)
          };
          if ok {
            return true;
          }
        }
      }
    }
    //knight checks
    let r = (king_row as isize) - 2 * (s as isize);
    if r >= 0 && r < 9 {
      for t in piece::KNIGHT_MOVES_DELTA_COL.iter() {
        let c = (king_col as isize) + *t;
        if c < 0 || c >= 9 {
          continue;
        }
        let piece = self.board[9 * r as usize + c as usize];
        if s * piece >= 0 {
          continue;
        }
        if piece.abs() != piece::KNIGHT {
          continue;
        }
        return true;
      }
    }
    false
  }
  fn attacking_pieces(&self, king_pos: usize, s: i8) -> attacking_pieces::AttackingPieces {
    let mut attacking_pieces = attacking_pieces::AttackingPieces::default();
    for (i, (flags, p)) in if s > 0 {
      piece::BLACK_DIRECTIONS_FLAGS.iter()
    } else {
      piece::WHITE_DIRECTIONS_FLAGS.iter()
    }
    .zip(consts::SLIDING_MASKS.iter().skip(8 * king_pos))
    .enumerate()
    {
      let b = *p & self.all_pieces;
      if b != 0 {
        let k = bitboards::scan(b, i);
        let piece = self.board[k];
        let t = s * piece;
        debug_assert_ne!(t, 0);
        if t < 0 {
          let pa = piece.abs();
          let cells = *p ^ consts::SLIDING_MASKS[8 * k + i] ^ (1u128 << k);
          let ok = if cells == 0 {
            piece::is_near_dir(pa, *flags)
          } else {
            piece::is_sliding_dir(pa, *flags)
          };
          if ok {
            attacking_pieces.push(k);
          }
        }
      }
    }
    let (king_row, king_col) = cell::unpack(king_pos);
    //knight checks
    let r = (king_row as isize) - 2 * (s as isize);
    if r >= 0 && r < 9 {
      for t in piece::KNIGHT_MOVES_DELTA_COL.iter() {
        let c = (king_col as isize) + *t;
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
    }
    attacking_pieces
  }
  fn checks_after_move(&self, king_pos: usize, s: i8, m: &Move) -> Checks {
    let mut attacking_pieces = attacking_pieces::AttackingPieces::default();
    let mut blocking_cells = 0u128;
    let mut set = Vec::with_capacity(2);
    if !m.is_drop() {
      set.push(m.from);
    }
    set.push(m.to);
    let mut used = 0u32;
    for c in set {
      if let Some(i) = direction::try_to_find_delta_direction_no(king_pos, c) {
        let bit = 1 << i;
        if (used & bit) != 0 {
          continue;
        }
        used += bit;
        let flags = if s > 0 {
          piece::BLACK_DIRECTIONS_FLAGS[i]
        } else {
          piece::WHITE_DIRECTIONS_FLAGS[i]
        };
        let p = consts::SLIDING_MASKS[8 * king_pos + i];
        let b = p & self.all_pieces;
        if b != 0 {
          let k = bitboards::scan(b, i);
          let piece = self.board[k];
          let t = s * piece;
          debug_assert_ne!(t, 0);
          if t < 0 {
            let pa = piece.abs();
            let cells = p ^ consts::SLIDING_MASKS[8 * k + i] ^ (1u128 << k);
            if cells == 0 {
              if piece::is_near_dir(pa, flags) {
                attacking_pieces.push(k);
              }
            } else {
              if piece::is_sliding_dir(pa, flags) {
                attacking_pieces.push(k);
                blocking_cells |= cells;
              }
            };
          }
        }
      }
    }
    if m.to_piece.abs() == piece::KNIGHT {
      let (king_row, king_col) = cell::unpack(king_pos);
      let (r, c) = cell::unpack(m.to);
      if r as isize == (king_row as isize) - 2 * (s as isize)
        && (c as isize - king_col as isize).abs() == 1
      {
        attacking_pieces.push(m.to);
      }
    }
    //double checks can't be blocked
    if attacking_pieces.len() > 1 {
      blocking_cells = 0;
    }
    Checks {
      attacking_pieces,
      blocking_cells,
      king_pos: Some(king_pos),
      hash: self.hash,
    }
  }
  fn checks(&self, king_pos: usize, s: i8) -> Checks {
    let mut attacking_pieces = attacking_pieces::AttackingPieces::default();
    let mut blocking_cells = 0u128;
    for (i, (flags, p)) in if s > 0 {
      piece::BLACK_DIRECTIONS_FLAGS.iter()
    } else {
      piece::WHITE_DIRECTIONS_FLAGS.iter()
    }
    .zip(consts::SLIDING_MASKS.iter().skip(8 * king_pos))
    .enumerate()
    {
      let b = *p & self.all_pieces;
      if b != 0 {
        let k = bitboards::scan(b, i);
        let piece = self.board[k];
        let t = s * piece;
        debug_assert_ne!(t, 0);
        if t < 0 {
          let pa = piece.abs();
          let cells = *p ^ consts::SLIDING_MASKS[8 * k + i] ^ (1u128 << k);
          if cells == 0 {
            if piece::is_near_dir(pa, *flags) {
              attacking_pieces.push(k);
            }
          } else {
            if piece::is_sliding_dir(pa, *flags) {
              attacking_pieces.push(k);
              blocking_cells |= cells;
            }
          };
        }
      }
    }
    let (king_row, king_col) = cell::unpack(king_pos);
    //knight checks
    let r = (king_row as isize) - 2 * (s as isize);
    if r >= 0 && r < 9 {
      for t in piece::KNIGHT_MOVES_DELTA_COL.iter() {
        let c = (king_col as isize) + *t;
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
    }
    //double checks can't be blocked
    if attacking_pieces.len() > 1 {
      blocking_cells = 0;
    }
    Checks {
      attacking_pieces,
      blocking_cells,
      king_pos: Some(king_pos),
      hash: self.hash,
    }
  }
  fn find_king_position(&self, s: i8) -> Option<usize> {
    if s > 0 {
      self.black_king_position
    } else {
      self.white_king_position
    }
  }
  fn compute_potential_drops_map(&self, drops_mask: u32) -> PotentialDropsMap {
    let mut m = PotentialDropsMap::default();
    let king_pos = self.find_king_position(-self.side);
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
    let r = (king_row as isize) + 2 * (self.side as isize);
    if r >= 0 && r < 9 {
      for t in piece::KNIGHT_MOVES_DELTA_COL.iter() {
        let c = (king_col as isize) + *t;
        if c < 0 || c >= 9 {
          continue;
        }
        let k = 9 * r as usize + c as usize;
        if self.board[k] != piece::NONE {
          continue;
        }
        m.insert(k, knight_bit);
      }
    }
    m
  }
  pub fn compute_checks(&self) -> Checks {
    match self.find_king_position(self.side) {
      Some(king_pos) => self.checks(king_pos, self.side),
      None => Checks {
        blocking_cells: 0,
        attacking_pieces: attacking_pieces::AttackingPieces::default(),
        king_pos: None,
        hash: self.hash,
      },
    }
  }
  pub fn compute_checks_after_move(&self, m: &Move) -> Checks {
    match self.find_king_position(self.side) {
      Some(king_pos) => self.checks_after_move(king_pos, self.side, m),
      None => Checks {
        blocking_cells: 0,
        attacking_pieces: attacking_pieces::AttackingPieces::default(),
        king_pos: None,
        hash: self.hash,
      },
    }
  }
  pub fn compute_checks_after_drop_with_check(&self, drop: &Move) -> Checks {
    debug_assert!(drop.is_drop());
    let king_pos = self.find_king_position(self.side);
    let pos = king_pos.unwrap();
    Checks {
      blocking_cells: if piece::sliding(drop.to_piece) {
        cell::between(pos, drop.to)
      } else {
        0
      },
      attacking_pieces: attacking_pieces::AttackingPieces::once(drop.to),
      king_pos,
      hash: self.hash,
    }
  }
  pub fn is_legal_after_move_in_checkless_position(&self, m: &Move) -> bool {
    //could be used in the special case then there is no check in position before move m
    let s = -self.side;
    let king_pos = self.find_king_position(s);
    match king_pos {
      Some(king_pos) => {
        if m.from_piece == piece::KING * s {
          return self.is_legal();
        }
        if let Some(i) = direction::try_to_find_delta_direction_no(king_pos, m.from) {
          !self.attacked_by_sliding_piece_in_given_direction_no(king_pos, s, i)
        } else {
          true
        }
      }
      None => true,
    }
  }
  pub fn is_legal(&self) -> bool {
    let s = -self.side;
    let king_pos = self.find_king_position(s);
    match king_pos {
      Some(king_pos) => !self.attacked(king_pos, s),
      None => true,
    }
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
  //slow (mate or stalemate)
  pub fn has_legal_move(&mut self) -> bool {
    let c = self.compute_checks();
    let moves = self.compute_moves(&c);
    for m in &moves {
      let u = self.do_move(m);
      let legal = self.is_legal();
      self.undo_move(m, &u);
      if legal {
        return true;
      }
    }
    for m in self.compute_drops(&c) {
      let u = self.do_move(&m);
      let legal = self.is_legal();
      self.undo_move(&m, &u);
      if legal {
        return true;
      }
    }
    false
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
              checks.attacking_pieces.first().unwrap(),
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
  fn compute_moves_after_nonblocking_check(&self, attacking_piece: Option<usize>) -> Vec<Move> {
    let mut r = Vec::new();
    let king = self.side * piece::KING;
    if let Some(pos) = if king > 0 {
      self.black_king_position.as_ref()
    } else {
      self.white_king_position.as_ref()
    } {
      let mut func_add = |m| {
        r.push(m);
        false
      };
      for t in piece::KING_MOVES.iter() {
        self.enumerate_piece_move(&mut func_add, *pos, king, t.0, t.1, false);
      }
    }
    if let Some(to) = attacking_piece {
      let pz_to = cell::promotion_zone(to, self.side);
      for from in self.attacking_pieces(to, -self.side) {
        let piece = self.board[from];
        if piece == king {
          continue;
        }
        debug_assert_eq!(piece.signum(), self.side);
        if piece::is_promoted(piece) || piece::could_unpromoted(piece, to) {
          r.push(Move {
            from,
            to,
            from_piece: piece,
            to_piece: piece,
          });
        }
        if piece::could_promoted(piece) && (pz_to || cell::promotion_zone(from, self.side)) {
          r.push(Move {
            from,
            to,
            from_piece: piece,
            to_piece: piece + piece.signum() * piece::PROMOTED,
          });
        }
      }
    }
    r
  }
  pub fn compute_moves(&self, checks: &Checks) -> Vec<Move> {
    debug_assert!(self.validate_checks(checks));
    match checks.attacking_pieces.len() {
      0 => {
        //no check
        let mut r = Vec::new();
        self.enumerate_simple_moves(|m| {
          r.push(m);
          false
        });
        r
      }
      1 => {
        let p = checks.attacking_pieces.first().unwrap();
        if checks.blocking_cells == 0 {
          self.compute_moves_after_nonblocking_check(Some(p))
        } else {
          let mut r = Vec::new();
          self.enumerate_simple_moves(|m| {
            let b = checks.blocking_cell(m.to);
            if (m.from_piece.abs() != piece::KING) == b || m.to == p {
              r.push(m);
            }
            false
          });
          r
        }
      }
      2 => self.compute_moves_after_nonblocking_check(None),
      _ => panic!("too many attacking pieces"),
    }
  }
  pub fn do_move(&mut self, m: &Move) -> moves::UndoMove {
    let u = moves::UndoMove {
      all_pieces: self.all_pieces,
      all_pieces2: self.all_pieces2,
      all_pieces3: self.all_pieces3,
      all_pieces4: self.all_pieces4,
      hash: self.hash,
      drop_masks: self.drop_masks,
      nifu_masks: self.nifu_masks,
      taken_piece: self.board[m.to],
    };
    if m.from != 0xff {
      self.board[m.from] = piece::NONE;
      self.hash ^= hash::get_piece_hash(m.from_piece, m.from);
      if m.to_piece == piece::KING {
        self.black_king_position = Some(m.to);
      } else if m.to_piece == -piece::KING {
        self.white_king_position = Some(m.to);
      }
      self.all_pieces ^= 1u128 << m.from;
      self.all_pieces2 ^= consts::MASKS2[m.from];
      self.all_pieces3 ^= consts::MASKS3[m.from];
      self.all_pieces4 ^= consts::MASKS4[m.from];
    } else {
      if m.to_piece > 0 {
        self.hash ^=
          hash::get_black_pocket_hash(m.to_piece, self.black_pockets[m.to_piece as usize]);
        if decrement_pocket(&mut self.black_pockets[m.to_piece as usize]) {
          self.drop_masks ^= 1u32 << m.to_piece;
        }
      } else {
        self.hash ^=
          hash::get_white_pocket_hash(-m.to_piece, self.white_pockets[(-m.to_piece) as usize]);
        if decrement_pocket(&mut self.white_pockets[(-m.to_piece) as usize]) {
          self.drop_masks ^= 1u32 << (16 - m.to_piece);
        }
      }
    }
    if m.from_piece != m.to_piece {
      if m.from_piece.abs() == piece::PAWN || m.to_piece.abs() == piece::PAWN {
        self.nifu_masks ^= 1u32 << (((1 + self.side as i32) << 3) + (m.to % 9) as i32);
      }
    }
    if u.taken_piece != piece::NONE {
      self.hash ^= hash::get_piece_hash(u.taken_piece, m.to);
      if u.taken_piece.abs() == piece::PAWN {
        self.nifu_masks ^= 1u32 << (((1 - self.side as i32) << 3) + (m.to % 9) as i32);
      }
      let p = piece::unpromote(u.taken_piece);
      if p > 0 {
        if increment_pocket(&mut self.white_pockets[p as usize]) {
          self.drop_masks ^= 1u32 << (16 + p);
        }
        self.hash ^= hash::get_white_pocket_hash(p, self.white_pockets[p as usize]);
      } else {
        if increment_pocket(&mut self.black_pockets[(-p) as usize]) {
          self.drop_masks ^= 1u32 << (-p);
        }
        self.hash ^= hash::get_black_pocket_hash(-p, self.black_pockets[(-p) as usize]);
      }
    } else {
      self.all_pieces ^= 1u128 << m.to;
      self.all_pieces2 ^= consts::MASKS2[m.to];
      self.all_pieces3 ^= consts::MASKS3[m.to];
      self.all_pieces4 ^= consts::MASKS4[m.to];
    }
    self.board[m.to] = m.to_piece;
    self.hash ^= hash::get_piece_hash(m.to_piece, m.to);
    self.move_no += 1;
    self.side *= -1;
    self.hash = !self.hash;
    debug_assert!(
      self.validate_hash(),
      "hash validation failed after doing {:?}",
      m
    );
    u
  }
  pub fn undo_move(&mut self, m: &Move, u: &moves::UndoMove) {
    self.all_pieces = u.all_pieces;
    self.all_pieces2 = u.all_pieces2;
    self.all_pieces3 = u.all_pieces3;
    self.all_pieces4 = u.all_pieces4;
    self.hash = u.hash;
    self.drop_masks = u.drop_masks;
    self.nifu_masks = u.nifu_masks;
    self.board[m.to] = u.taken_piece;
    if m.from != 0xff {
      self.board[m.from] = m.from_piece;
      if m.from_piece == piece::KING {
        self.black_king_position = Some(m.from);
      } else if m.from_piece == -piece::KING {
        self.white_king_position = Some(m.from);
      }
    } else {
      if m.to_piece > 0 {
        self.black_pockets[m.to_piece as usize] += 1;
      } else {
        self.white_pockets[(-m.to_piece) as usize] += 1;
      }
    }
    if u.taken_piece != piece::NONE {
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
  pub fn parse_kif_move(&mut self, kif: &str, last_move: Option<Move>) -> Option<Move> {
    let checks = self.compute_checks();
    let moves = self.compute_moves(&checks);
    for m in moves {
      if kif == m.to_kif(&last_move) {
        return Some(m);
      }
    }
    let drops = self.compute_drops(&checks);
    for m in drops {
      if kif == m.to_kif(&last_move) {
        return Some(m);
      }
    }
    None
  }
  pub fn is_futile_drop(&mut self, checks: &Checks, drop: &Move) -> bool {
    let attacking_piece = checks.attacking_pieces.first().unwrap();
    let p = self.board[attacking_piece];
    let mut u = moves::Moves::with_capacity(2);
    u.push(
      self,
      Move {
        from: attacking_piece,
        to: drop.to,
        from_piece: p,
        to_piece: p,
      },
    );
    if !self.is_legal() {
      u.undo(self);
      return false;
    }
    let checks = Checks {
      blocking_cells: 0,
      attacking_pieces: attacking_pieces::AttackingPieces::once(drop.to),
      king_pos: self.find_king_position(self.side),
      hash: self.hash,
    };
    let moves = self.compute_moves(&checks);
    for m in moves {
      u.push(self, m);
      if self.is_legal() {
        //no mate
        u.undo(self);
        return false;
      }
      u.pop(self);
    }
    //mate
    u.undo(self);
    true
  }
  pub fn validate_move(&self, m: &Move) -> bool {
    if m.to_piece * self.side <= 0 {
      return false;
    }
    if m.is_drop() {
      if self.board[m.to] != piece::NONE || piece::is_promoted(m.to_piece) {
        return false;
      }
      let q = if m.to_piece > 0 {
        &self.black_pockets
      } else {
        &self.white_pockets
      };
      let p = m.to_piece.abs();
      if q[p as usize] == 0 {
        return false;
      }
      if p == piece::PAWN {
        let bit = 1u32 << (((1 + self.side as i32) << 3) + (m.to % 9) as i32);
        if (self.nifu_masks & bit) != 0 {
          return false;
        }
      }
      true
    } else {
      if self.board[m.from] != m.from_piece || m.from == m.to {
        return false;
      }
      self.attacking_pieces(m.to, -self.side).contains(&m.from)
    }
  }
  pub fn reorder_takes_to_front(&self, moves: &mut [Move]) -> usize {
    if moves.is_empty() {
      return 0;
    }
    let mut i = 0;
    let mut j = moves.len() - 1;
    while i < j {
      if self.is_take(&moves[j]) {
        moves.swap(i, j);
        i += 1;
      } else {
        j -= 1;
      }
    }
    i
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
          let s = piece::to_string(*c, true);
          write!(f, "{}", if *c > 0 { s } else { s.to_ascii_lowercase() })?;
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
        if k > 1 {
          write!(f, "{}", k)?;
        }
        write!(f, "{}", c.to_ascii_uppercase())?;
        t += k as u32;
      }
    }
    for (&k, c) in self.white_pockets[piece::PAWN as usize..piece::KING as usize]
      .iter()
      .zip(piece::PIECE_TO_CHAR.chars())
    {
      if k > 0 {
        if k > 1 {
          write!(f, "{}", k)?;
        }
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

impl Position {
  fn validate_checks(&self, checks: &Checks) -> bool {
    if self.hash != checks.hash {
      return false;
    }
    match checks.king_pos {
      Some(king_pos) => self.board[king_pos] == piece::KING * self.side,
      None => true,
    }
  }
  fn validate_hash(&self) -> bool {
    self.hash == self.compute_hash()
  }
}

impl Default for Position {
  fn default() -> Self {
    Position::parse_sfen("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1").unwrap()
  }
}

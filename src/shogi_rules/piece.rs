use super::cell::Direction;
pub const PIECE_TO_CHAR: &str = "plnsgbrk";
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
pub const KNIGHT_MOVES: [Direction; 2] = [(-2, -1), (-2, 1)];
pub const SILVER_MOVES: [Direction; 5] = [(-1, -1), (-1, 0), (-1, 1), (1, -1), (1, 1)];
pub const GOLD_MOVES: [Direction; 6] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, 0)];
pub const ROOK_MOVES: [Direction; 4] = [(-1, 0), (0, -1), (0, 1), (1, 0)];
pub const BISHOP_MOVES: [Direction; 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
pub const KING_MOVES: [Direction; 8] = [
  (-1, -1),
  (-1, 0),
  (-1, 1),
  (0, -1),
  (0, 1),
  (1, -1),
  (1, 0),
  (1, 1),
];
//flags: +1 - bishop
//flags: +2 - rook
//flags: +4 - general (forward)
pub const BLACK_DIRECTIONS: [(isize, isize, u8); 8] = [
  (-1, -1, 1 + 4),
  (-1, 0, 2 + 4),
  (-1, 1, 1 + 4),
  (0, -1, 2),
  (0, 1, 2),
  (1, -1, 1),
  (1, 0, 2),
  (1, 1, 1),
];
pub const WHITE_DIRECTIONS: [(isize, isize, u8); 8] = [
  (1, -1, 1 + 4),
  (1, 0, 2 + 4),
  (1, 1, 1 + 4),
  (0, -1, 2),
  (0, 1, 2),
  (-1, -1, 1),
  (-1, 0, 2),
  (-1, 1, 1),
];
pub fn unpromote(v: i8) -> i8 {
  if v >= PROMOTED {
    v - PROMOTED
  } else if v <= -PROMOTED {
    v + PROMOTED
  } else {
    v
  }
}
pub fn could_unpromoted(piece: i8, cell: usize) -> bool {
  if piece.abs() >= PROMOTED {
    return false;
  }
  if piece == PAWN || piece == LANCE {
    cell >= 9
  } else if piece == -PAWN || piece == -LANCE {
    cell < 72
  } else if piece == KNIGHT {
    cell >= 18
  } else if piece == -KNIGHT {
    cell < 63
  } else {
    true
  }
}

pub fn could_promoted(piece: i8) -> bool {
  let p = piece.abs();
  p < KING && p != GOLD
}

pub fn sliding_dir_to_mask(flags: u8) -> u32 {
  match flags {
    1 => 1u32 << BISHOP,
    2 => 1u32 << ROOK,
    5 => 1u32 << BISHOP,
    6 => (1u32 << ROOK) | (1u32 << LANCE),
    _ => panic!("unhandled flags {}", flags),
  }
}

pub fn is_sliding_dir(abs_piece: i8, flags: u8) -> bool {
  assert!(abs_piece > 0);
  match abs_piece {
    LANCE => (flags & 6) == 6,
    ROOK | PROMOTED_ROOK => (flags & 2) != 0,
    BISHOP | PROMOTED_BISHOP => (flags & 1) != 0,
    _ => false,
  }
}

pub fn near_dir_to_mask(flags: u8) -> u32 {
  match flags {
    1 => (1u32 << BISHOP) | (1u32 << SILVER),
    2 => (1u32 << ROOK) | (1u32 << GOLD),
    5 => (1u32 << BISHOP) | (1u32 << SILVER) | (1u32 << GOLD),
    6 => (1u32 << ROOK) | (1u32 << SILVER) | (1u32 << GOLD) | (1u32 << PAWN) | (1u32 << LANCE),
    _ => panic!("unhandled flags {}", flags),
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
    }
  }
}
pub fn from_char(c: char) -> i8 {
  let s = if c.is_uppercase() {
    1
  } else if c.is_lowercase() {
    -1
  } else {
    0
  };
  if s == 0 {
    return NONE;
  }
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
pub fn sliding(piece: i8) -> bool {
  match piece.abs() {
    LANCE | BISHOP | ROOK | PROMOTED_BISHOP | PROMOTED_ROOK => true,
    _ => false,
  }
}

pub fn is_promoted(piece: i8) -> bool {
  piece.abs() >= PROMOTED
}

pub fn to_string(piece: i8, plus_in_prefix: bool) -> String {
  let mut s = String::new();
  if plus_in_prefix && is_promoted(piece) {
    s.push('+');
  }
  let c = match unpromote(piece.abs()) {
    PAWN => 'P',
    LANCE => 'L',
    KNIGHT => 'N',
    SILVER => 'S',
    GOLD => 'G',
    BISHOP => 'B',
    ROOK => 'R',
    KING => 'K',
    _ => panic!("unhandled piece {} in match expression", piece),
  };
  s.push(c);
  if !plus_in_prefix && is_promoted(piece) {
    s.push('+');
  }
  s
}

pub fn color(piece: i8) -> &'static str {
  if piece > 0 {
    "black"
  } else {
    "white"
  }
}

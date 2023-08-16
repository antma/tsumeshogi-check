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
pub const WHITE_PAWN: i8 = -PAWN;
pub const WHITE_LANCE: i8 = -LANCE;
pub const WHITE_KNIGHT: i8 = -KNIGHT;
pub const WHITE_SILVER: i8 = -SILVER;
pub const WHITE_GOLD: i8 = -GOLD;
pub const WHITE_BISHOP: i8 = -BISHOP;
pub const WHITE_ROOK: i8 = -ROOK;
pub const WHITE_KING: i8 = -KING;
pub const WHITE_PROMOTED_PAWN: i8 = -PROMOTED_PAWN;
pub const WHITE_PROMOTED_LANCE: i8 = -PROMOTED_LANCE;
pub const WHITE_PROMOTED_KNIGHT: i8 = -PROMOTED_KNIGHT;
pub const WHITE_PROMOTED_SILVER: i8 = -PROMOTED_SILVER;
pub const WHITE_PROMOTED_BISHOP: i8 = -PROMOTED_BISHOP;
pub const WHITE_PROMOTED_ROOK: i8 = -PROMOTED_ROOK;
pub const KNIGHT_MOVES_DELTA_COL: [isize; 2] = [-1, 1];
pub const fn expected_number_of_pieces(abs_piece: i8) -> u32 {
  match abs_piece {
    PAWN => 18,
    LANCE | KNIGHT | SILVER | GOLD => 4,
    BISHOP | ROOK | KING => 2,
    _ => 0,
  }
}
pub fn promote(v: i8) -> i8 {
  v + (if v > 0 { PROMOTED } else { -PROMOTED })
}
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

const fn sliding_dir_to_drop_mask(flags: u8) -> u8 {
  match flags {
    1 => 1 << BISHOP,
    2 => 1 << ROOK,
    5 => 1 << BISHOP,
    6 => (1 << ROOK) | (1 << LANCE),
    _ => 0,
    //_ => panic!("unhandled flags {}", flags),
  }
}

pub fn is_sliding_dir(abs_piece: i8, flags: u8) -> bool {
  debug_assert!(abs_piece > 0);
  match abs_piece {
    LANCE => (flags & 6) == 6,
    ROOK | PROMOTED_ROOK => (flags & 2) != 0,
    BISHOP | PROMOTED_BISHOP => (flags & 1) != 0,
    _ => false,
  }
}

const fn near_dir_to_drop_mask(flags: u8) -> u8 {
  match flags {
    1 => (1 << BISHOP) | (1 << SILVER),
    2 => (1 << ROOK) | (1 << GOLD),
    5 => (1 << BISHOP) | (1 << SILVER) | (1 << GOLD),
    6 => (1 << ROOK) | (1 << SILVER) | (1 << GOLD) | (1 << PAWN) | (1 << LANCE),
    _ => 0,
    //_ => panic!("unhandled flags {}", flags),
  }
}

pub const fn flags_to_drop_mask(flags: u8) -> (u8, u8) {
  (
    near_dir_to_drop_mask(flags),
    sliding_dir_to_drop_mask(flags),
  )
}

pub fn is_near_dir(abs_piece: i8, flags: u8) -> bool {
  debug_assert!(abs_piece > 0);
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

pub fn to_human_string(p: i8) -> String {
  let s = match p {
    PAWN => "pawn",
    LANCE => "lance",
    KNIGHT => "knight",
    SILVER => "silver",
    GOLD => "gold",
    BISHOP => "bishop",
    ROOK => "rook",
    KING => "king",
    _ => "???",
  };
  String::from(s)
}

pub fn to_jp_char(abs_piece: i8) -> char {
  match abs_piece {
    PAWN => '歩',
    LANCE => '香',
    KNIGHT => '桂',
    SILVER => '銀',
    GOLD => '金',
    BISHOP => '角',
    ROOK => '飛',
    KING => '玉',
    PROMOTED_PAWN => 'と',
    PROMOTED_LANCE => '杏',
    PROMOTED_KNIGHT => '圭',
    PROMOTED_SILVER => '全',
    PROMOTED_BISHOP => '馬',
    PROMOTED_ROOK => '龍',
    _ => panic!("unhandled piece {}", abs_piece),
  }
}

pub fn to_jp_string(abs_piece: i8) -> &'static str {
  match abs_piece {
    PAWN => "歩",
    LANCE => "香",
    KNIGHT => "桂",
    SILVER => "銀",
    GOLD => "金",
    BISHOP => "角",
    ROOK => "飛",
    KING => "玉",
    PROMOTED_PAWN => "と",
    PROMOTED_LANCE => "成香",
    PROMOTED_KNIGHT => "成桂",
    PROMOTED_SILVER => "成銀",
    PROMOTED_BISHOP => "馬",
    PROMOTED_ROOK => "龍",
    _ => panic!("unhandled piece {}", abs_piece),
  }
}

const fn bit(p: i8) -> u32 {
  1 << p
}

pub const fn flags_to_mask(flags: u8) -> (u32, u32) {
  const GOLD_MASK: u32 = bit(GOLD)
    | bit(PROMOTED_PAWN)
    | bit(PROMOTED_LANCE)
    | bit(PROMOTED_KNIGHT)
    | bit(PROMOTED_SILVER);
  match flags {
    1 => (
      bit(BISHOP) | bit(KING) | bit(PROMOTED_BISHOP) | bit(PROMOTED_ROOK) | bit(SILVER),
      bit(BISHOP) | bit(PROMOTED_BISHOP),
    ),
    2 => (
      bit(ROOK) | bit(KING) | bit(PROMOTED_BISHOP) | bit(PROMOTED_ROOK) | GOLD_MASK,
      bit(ROOK) | bit(PROMOTED_ROOK),
    ),
    5 => (
      bit(BISHOP) | bit(KING) | bit(PROMOTED_BISHOP) | bit(PROMOTED_ROOK) | bit(SILVER) | GOLD_MASK,
      bit(BISHOP) | bit(PROMOTED_BISHOP),
    ),
    6 => (
      bit(ROOK)
        | bit(KING)
        | bit(PROMOTED_BISHOP)
        | bit(PROMOTED_ROOK)
        | bit(SILVER)
        | GOLD_MASK
        | bit(LANCE)
        | bit(PAWN),
      bit(ROOK) | bit(PROMOTED_ROOK) | bit(LANCE),
    ),
    _ => (0, 0),
  }
}

#[test]
fn test_flags_to_mask() {
  for flags in [1, 2, 5, 6] {
    let (near, far) = flags_to_mask(flags);
    for p in vec![
      PAWN,
      LANCE,
      KNIGHT,
      SILVER,
      GOLD,
      BISHOP,
      ROOK,
      KING,
      PROMOTED_PAWN,
      PROMOTED_LANCE,
      PROMOTED_KNIGHT,
      PROMOTED_SILVER,
      PROMOTED_BISHOP,
      PROMOTED_ROOK,
    ] {
      assert_eq!(
        (near & (1 << p)) != 0,
        is_near_dir(p, flags),
        "near: flags = {}, piece = {}",
        flags,
        p
      );
      assert_eq!(
        (far & (1 << p)) != 0,
        is_sliding_dir(p, flags),
        "far: flags = {}, piece = {}",
        flags,
        p
      );
    }
  }
}

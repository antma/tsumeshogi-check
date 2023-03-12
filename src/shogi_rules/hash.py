#!/usr/bin/python3
import random, sys

def p(name, l, pockets=False):
  a = []
  n = 1 << 64
  for i in range(l):
    a.append(random.randrange(n))
  if pockets:
    for p in range(6):
      for i in range(3):
        j = 4 * p + i
        a[j+1] ^= a[j]
    for i in range(17):
      j = 6 * 4 + i
      a[j+1] ^= a[j]
  s = str(a)
  sys.stdout.write('const {}: [u64; {}] = {};\n'.format(name, l, s))

random.seed('Habu almost hundred titles')
sys.stdout.write('''
use super::piece;
''')

p('BLACK_PIECES', 81 * 9)
p('WHITE_PIECES', 81 * 9)
p('BLACK_POCKETS', 18 + 6 * 4)
p('WHITE_POCKETS', 18 + 6 * 4)

sys.stdout.write('''
pub fn get_piece_hash(piece: i8, cell: usize) -> u64 {
  assert_ne!(piece, piece::NONE);
  let q = if piece > 0 { &BLACK_PIECES } else { &WHITE_PIECES }; 
  let piece = piece.abs();
  if piece >= piece::PROMOTED { 
    q[cell] ^ q[81 * (piece - piece::PROMOTED) as usize + cell]
  } else {
    q[81 * piece as usize + cell]
  }
}

fn get_pocket_hash(p: &[u64], abs_piece: i8, c: u8) -> u64 {
  assert!(abs_piece > 0);
  assert!(c > 0);
  if abs_piece == piece::PAWN { p[23 + (c as usize)] }
  else {
    p[4 * (abs_piece - 2) as usize + (c as usize) - 1]
  }
}

pub fn get_black_pocket_hash(abs_piece: i8, c: u8) -> u64 {
  get_pocket_hash(&BLACK_POCKETS, abs_piece, c)
}

pub fn get_cumulative_black_pocket_hash(abs_piece: i8, c: u8) -> u64 {
  (1 ..= c).fold(0, |acc, i| acc ^ get_pocket_hash(&BLACK_POCKETS, abs_piece, i))
}

pub fn get_white_pocket_hash(abs_piece: i8, c: u8) -> u64 {
  get_pocket_hash(&WHITE_POCKETS, abs_piece, c)
}

pub fn get_cumulative_white_pocket_hash(abs_piece: i8, c: u8) -> u64 {
  (1 ..= c).fold(0, |acc, i| acc ^ get_pocket_hash(&WHITE_POCKETS, abs_piece, i))
}

pub fn compute_black_pockets_hash(black_pockets: &[u8]) -> u64 {
  black_pockets.iter().enumerate().skip(1).filter(|&(_, &c)| c > 0).fold(0, |acc, (p, &c)| acc ^ get_cumulative_black_pocket_hash(p as i8, c))
}

pub fn compute_white_pockets_hash(white_pockets: &[u8]) -> u64 {
  white_pockets.iter().enumerate().skip(1).filter(|&(_, &c)| c > 0).fold(0, |acc, (p, &c)| acc ^ get_cumulative_white_pocket_hash(p as i8, c))
}


''')


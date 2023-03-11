#!/usr/bin/python3
import random, sys

def p(name, l):
  a = []
  n = 1 << 64
  for i in range(l):
    a.append(random.randrange(n))
  #s = str(a).replace('[', '{').replace(']', '}')
  s = str(a)
  sys.stdout.write('const {}: [u64; {}] = {};\n'.format(name, l, s))

random.seed('Habu hundred titles')
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

fn get_pocket_hash(p: &[u64], piece: i8, c: u8) -> u64 {
  assert!(piece > 0);
  if piece == piece::PAWN { p[23 + (c as usize)] }
  else {
    p[4 * (piece - 2) as usize + (c as usize) - 1]
  }
}

pub fn get_black_pocket_hash(piece: i8, c: u8) -> u64 {
  get_pocket_hash(&BLACK_POCKETS, piece, c)
}

pub fn get_white_pocket_hash(piece: i8, c: u8) -> u64 {
  get_pocket_hash(&WHITE_POCKETS, -piece, c)
}

''')


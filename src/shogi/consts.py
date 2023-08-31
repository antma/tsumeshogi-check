#!/usr/bin/python3
import random, sys

filename = sys.argv[0].removesuffix('.py') + '.rs'
f = open(filename, 'w')

def legal_coordinate(x): return (x >= 0) and (x < 9)
def legal_cell(y, x): return legal_coordinate(x) and legal_coordinate(y)
def unpack(i): return (i // 9, i % 9)
def mirrory(i):
   y, x = unpack(i)
   return 9 * (8 - y) + x
def mirrory_array(a):
  b = [None] * 81
  for i in range(81):
    r = 0
    x = a[i]
    for j in range(81):
      if (x & (1 << j)) != 0:
        r += 1 << mirrory(j)
    b[mirrory(i)] = r
  return b

def go(a, k, dy, dx, y, x):
  idx = 8 * (9 * y + x) + k
  if a[idx] != None: return a[idx]
  j = y + dy
  i = x + dx
  if legal_cell(j, i):
    v = go(a, k, dy, dx, j, i)
    bit = 1 << (j * 9 + i)
    assert((v & bit) == 0)
    a[idx] = v + bit
  else:
    a[idx] = 0
  return a[idx]

def p(f, name, a, tp):
  s = str(a)
  f.write('pub const {}: [u{}; {}] = {};\n'.format(name, tp, len(a), s))

def rp(f, name, l):
  a = []
  n = 1 << 64
  for i in range(l):
    a.append(random.randrange(n))
  p(f, name, a, 64)

def diagonal(f, diag_key, no):
  a = list(range(81))
  a.sort(key = diag_key)
  masks = [None] * 81
  data = [0]
  data_slice = [None] * 81
  shift = [None] * 81
  m = [None] * 81
  for i in range(81):
    j = a[i]
    masks[j] = 1 << i
  i = 0
  while i < 81:
    j = i + 1
    d = diag_key(a[i])
    while (j < 81) and d == diag_key(a[j]): j += 1
    l = j - i
    for k in range(i, j):
      shift[a[k]] = i + 1
      if l <= 2: m[a[k]] = 0
      else: m[a[k]] = (1 << (l - 2)) - 1
    if l == 1:
      data_slice[a[i]] = (0, 1)
    elif l == 2:
      for k in range(i, j):
        data_slice[a[k]] = (len(data), len(data) + 1)
        data.append((1 << a[k]) ^ (1 << a[i]) ^ (1 << a[i+1]))
    else:
      for s in range(l):
        u = len(data)
        for mask in range(1 << (l-2)):
          x = mask << 1
          r = 0
          for d in [-1, 1]:
            j = s
            while True:
              j += d
              if (j < 0) or (j >= l): break
              r |= 1 << a[i+j]
              if (x & (1 << j)) != 0: break
          data.append(r)
        data_slice[a[i+s]] = (u, len(data))
    i += l
  p(f, 'MASKS' + str(no), masks, 128)
  p(f, 'DATA' + str(no), data, 128)
  f.write('pub const BISHOP' + str(no) + ': [RotatedBitboard; 81] = [\n')
  for i in range(81):
    f.write('RotatedBitboard {{offset: {}, shift: {}, mask: {}}},\n'.format(data_slice[i][0], shift[i], m[i]))
  f.write('];\n')

random.seed('Habu almost hundred titles')

f.write('//Do not edit. Machine generated.\n')

rp(f, 'BLACK_PIECES', 81 * 9)
rp(f, 'WHITE_PIECES', 81 * 9)
rp(f, 'BLACK_POCKETS', 18 + 6 * 4)
rp(f, 'WHITE_POCKETS', 18 + 6 * 4)

a = [None] * (81 * 8)
k = 0
for dy in range(-1, 2):
  for dx in range(-1, 2):
    if (dx == 0) and (dy == 0): continue
    for y in range(9):
      for x in range(9):
        go(a, k, dy, dx, y, x)
    k += 1
p(f, 'SLIDING_MASKS', a, 128)
p(f, 'MASKS2', [1 << (9 * (i % 9) + (i // 9)) for i in range(81)], 128)
queen_attack = []
for i in range(81):
  r = 0
  for q in a[8*i:8*(i+1)]: r |= q
  queen_attack.append(r)
king_attack = []
black_gold_attack = []
white_gold_attack = []
black_silver_attack = []
white_silver_attack = []
black_knight_attack = []
white_knight_attack = []
black_local_check_candidates = []
potential_queen_positions = []
for row in range(9):
  for col in range(9):
    n1 = 0
    n2 = 0
    for delta_col in [-1, 1]:
      c = col + delta_col
      if not legal_coordinate(c): continue
      r = row - 2
      if legal_coordinate(r): n1 |= 1 << (9 * r + c)
      r = row + 2
      if legal_coordinate(r): n2 |= 1 << (9 * r + c)
    black_knight_attack.append(n1)
    white_knight_attack.append(n2)
    r = 0
    r2 = 0
    r3 = 0
    r4 = 0
    r5 = 0
    r6 = 0
    for y in range(-1, 2):
      j = row + y
      if not legal_coordinate(j): continue
      for x in range(-1, 2):
        if (y == 0) and (x == 0): continue
        i = col + x
        if not legal_coordinate(i): continue
        l = 9 * j + i
        r6 |= queen_attack[l]
        bit = 1 << l
        r |= bit
        if abs(x) == 1:
          if y != 1: r2 |= bit
          if y != -1: r3 |= bit
        else:
          r2 |= bit
          r3 |= bit
        if x == 0:
          if y == -1: r4 |= bit
          if y == 1: r5 |= bit
        elif y != 0:
          r4 |= bit
          r5 |= bit
    king_attack.append(r)
    black_gold_attack.append(r2)
    white_gold_attack.append(r3)
    black_silver_attack.append(r4)
    white_silver_attack.append(r5)
    potential_queen_positions.append(r6)
    r = 0
    for y in range(-2, 3):
      j = row + y
      if not legal_coordinate(j): continue
      for x in range(-2, 3):
        if abs(y) + abs(x) == 0: continue
        i = col + x
        if not legal_coordinate(i): continue
        r += 1 << (9 * j + i)
    j = row + 4
    if legal_coordinate(j):
      for x in [-2, 0, 2]:
        i = col + x
        if legal_coordinate(i):
          r += 1 << (9 * j + i)
    if row < 2:
      j = row + 3
      for x in range(-2, 3):
        i = col + x
        if legal_coordinate(i):
          r += 1 << (9 * j + i)
    black_local_check_candidates.append(r)

near_attack = []
for i in range(81): near_attack.append(king_attack[i] | white_knight_attack[i])
king_move_candidates = []
for row in range(9):
  for col in range(9):
    r = 0
    for y in range(-1, 2):
      j = row + y
      if not legal_coordinate(j): continue
      for x in range(-1, 2):
        if abs(y) + abs(x) == 0: continue
        i = col + x
        if not legal_coordinate(i): continue
        r |= near_attack[9 * j + i]
    king_move_candidates.append(r)

assert(mirrory_array(black_gold_attack) == white_gold_attack)
assert(mirrory_array(black_silver_attack) == white_silver_attack)
p(f, 'KING_MASKS', king_attack, 128)
p(f, 'BLACK_GOLD_MASKS', black_gold_attack, 128)
p(f, 'BLACK_SILVER_MASKS', black_silver_attack, 128)
p(f, 'WHITE_GOLD_MASKS', white_gold_attack, 128)
p(f, 'WHITE_SILVER_MASKS', white_silver_attack, 128)
p(f, 'BLACK_KNIGHT_MASKS', black_knight_attack, 128)
p(f, 'WHITE_KNIGHT_MASKS', white_knight_attack, 128)
p(f, 'BLACK_LOCAL_CHECK_CANDIDATES', black_local_check_candidates, 128)
p(f, 'WHITE_LOCAL_CHECK_CANDIDATES', mirrory_array(black_local_check_candidates), 128)
p(f, 'BLACK_KING_MOVES_CANDIDATES', mirrory_array(king_move_candidates), 128)
p(f, 'WHITE_KING_MOVES_CANDIDATES', king_move_candidates, 128)
p(f, 'POTENTIAL_SLIDING_PIECE_POSITIONS', potential_queen_positions, 128)
h = []
v = []
for s in range(9):
  for mask in range(128):
    x = mask << 1
    r = 0
    for d in [-1, 1]:
      j = s
      while True:
       j += d
       if not legal_coordinate(j): break
       bit = 1 << j
       r += bit
       if (x & bit) != 0: break
    h.append(r)
    r2 = 0
    for i in range(9):
      bit = 1 << i
      if (r & bit) != 0: r2 += 1 << (9 * i)
    v.append(r2)

p(f, 'ROOK_HORIZONTAL_MASKS', h, 16)
p(f, 'ROOK_VERTICAL_MASKS', v, 128)

f.write('''pub struct RotatedBitboard {
  pub offset: usize,
  pub shift: usize,
  pub mask: usize
}
''')
diagonal(f, lambda i: (i // 9) + (i % 9), 3)
diagonal(f, lambda i: 8 + (i // 9) - (i % 9), 4)

f.close()


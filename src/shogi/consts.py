#!/usr/bin/python3
import random, sys

filename = sys.argv[0].removesuffix('.py') + '.rs'
f = open(filename, 'w')

def legal_cell(y, x):
  return (x >= 0) and (x < 9) and (y >= 0) and (y < 9)

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
king_attack = []
black_gold_attack = []
white_gold_attack = []
black_silver_attack = []
white_silver_attack = []
for row in range(9):
  for col in range(9):
    r = 0
    r2 = 0
    r3 = 0
    r4 = 0
    r5 = 0
    for y in range(-1, 2):
      j = row + y
      if (j < 0) or (j > 8): continue
      for x in range(-1, 2):
        if (y == 0) and (x == 0): continue
        i = col + x
        if (i < 0) or (i > 8): continue
        l = 9 * j + i
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
p(f, 'KING_MASKS', king_attack, 128)
p(f, 'BLACK_GOLD_MASKS', black_gold_attack, 128)
p(f, 'BLACK_SILVER_MASKS', black_silver_attack, 128)
p(f, 'WHITE_GOLD_MASKS', white_gold_attack, 128)
p(f, 'WHITE_SILVER_MASKS', white_silver_attack, 128)
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
       if (j < 0) or (j >= 9): break
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

f.close()


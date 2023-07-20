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

f.close()


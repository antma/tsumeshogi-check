#!/usr/bin/python3
import random, sys

filename = sys.argv[0].removesuffix('.py') + '.rs'
f = open(filename, 'w')

def p(f, name, l, pockets=False):
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
  f.write('pub const {}: [u64; {}] = {};\n'.format(name, l, s))

random.seed('Habu almost hundred titles')

f.write('//Do not edit. Machine generated.\n')

p(f, 'BLACK_PIECES', 81 * 9)
p(f, 'WHITE_PIECES', 81 * 9)
p(f, 'BLACK_POCKETS', 18 + 6 * 4)
p(f, 'WHITE_POCKETS', 18 + 6 * 4)

f.close()


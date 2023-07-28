##How to compile

###Release
```
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release
```
###Debug
```
cargo build
```
##Usage examples
```
DEPTH=5 ./tsumeshogi-check --warn -d${DEPTH} -o out.sfen input.sfen
```
Analyse _input.sfen_ (text file with one position in SFEN format) and output results in _out1.sfen_, _out3.sfen_, _out5.sfen_.
_out1.sfen_ contains solutions for mate in one puzzles.
```
DEPTH=5 ./tsumeshogi-check --info -d${DEPTH} -o out.kif input.kif
```
Analyse _input.sfen_ (concatenation of 81dojo KIF games) and output results in _out1.kif_, _out3.kif_, _out5.kif_.

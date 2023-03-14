use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

struct PSNFileIterator {
  reader: BufReader<File>,
}

impl Iterator for PSNFileIterator {
  type Item = std::io::Result<Vec<String>>;
  fn next(&mut self) -> Option<Self::Item> {
    let mut r = Vec::new();
    loop {
      let mut s = String::new();
      match self.reader.read_line(&mut s) {
        Ok(sz) => {
          if sz == 0 {
            //EOF reached
            break None;
          }
          if r.is_empty() && s.trim().is_empty() {
            //skip first blank lines
            continue;
          }
          let last = s.starts_with("--") && s.ends_with("--");
          r.push(s);
          if last {
            return Some(Ok(r));
          }
        }
        Err(err) => break Some(Err(err)),
      }
    }
  }
}

impl PSNFileIterator {
  pub fn new(filename: &str) -> std::io::Result<Self> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    Ok(Self { reader })
  }
}

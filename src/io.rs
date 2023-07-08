use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

pub struct FileIterator {
  it: Lines<BufReader<File>>,
  separator: String,
  lines: usize,
  failed: bool,
}

impl Iterator for FileIterator {
  type Item = std::io::Result<Vec<String>>;
  fn next(&mut self) -> Option<Self::Item> {
    if self.failed {
      return None;
    }
    let mut r = Vec::new();
    let mut lines_read = 0;
    while let Some(t) = self.it.next() {
      if t.is_err() {
        self.failed = true;
        return Some(Err(t.err().unwrap()));
      }
      lines_read += 1;
      self.lines += 1;
      let t = t.unwrap();
      let t = t.trim();
      if t == self.separator {
        if self.lines > 1 {
          break;
        }
      } else {
        r.push(t.to_owned());
      }
    }
    if lines_read == 0 {
      None
    } else {
      Some(Ok(r))
    }
  }
}

impl FileIterator {
  pub fn new(filename: &str, separator: &str) -> std::io::Result<Self> {
    let file = File::open(filename)?;
    let it = BufReader::new(file).lines();
    Ok(Self {
      it,
      separator: separator.to_owned(),
      lines: 0,
      failed: false,
    })
  }
}

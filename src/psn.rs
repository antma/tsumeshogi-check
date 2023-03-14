use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[test]
fn test_parse_header() {
  assert_eq!(
    parse_header("[Date \"24/02/2023\"]"),
    Some((String::from("date"), String::from("24/02/2023")))
  );
}

fn parse_header(s: &str) -> Option<(String, String)> {
  let mut key = String::new();
  let mut value = String::new();
  let mut st = 0;
  for c in s.trim().chars() {
    match st {
      0 => {
        if c == '[' {
          st += 1;
        } else {
          return None;
        }
      }
      1 => {
        if c.is_ascii_whitespace() {
          st += 1;
        } else if c.is_ascii_alphabetic() {
          key.push(c.to_ascii_lowercase());
        } else {
          return None;
        }
      }
      2 => {
        if !c.is_ascii_whitespace() {
          if c == '"' {
            st += 1;
          } else {
            return None;
          }
        }
      }
      3 => {
        if c == '"' {
          st += 1;
        } else {
          value.push(c);
        }
      }
      4 => {
        if c == ']' {
          st += 1;
        } else {
          return None;
        }
      }
      _ => return None,
    }
  }
  if st < 5 {
    return None;
  }
  Some((key, value))
}

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

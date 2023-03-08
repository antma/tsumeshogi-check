use std::iter::{Iterator, Peekable};
use std::str::FromStr;

fn try_parse_option<R: FromStr<Err = impl std::fmt::Display>, I: Iterator<Item = String>>(
  it: &mut Peekable<I>,
  short: &str,
  long: &str,
) -> Option<R> {
  if let Some(s) = it.peek() {
    if let Some(t) = s.strip_prefix("--") {
      if let Some(u) = t.strip_prefix(long) {
        if let Some(w) = u.strip_prefix("=") {
          let w = w.trim();
          match R::from_str(w) {
            Ok(res) => {
              it.next();
              return Some(res);
            }
            Err(err) => panic!("can't parse command line argument {}, {}", s, err),
          }
        }
        let w = u.trim();
        if w.is_empty() {
          let s = s.clone();
          it.next();
          match it.next() {
            Some(w) => {
              match R::from_str(&w) {
                Ok(res) => return Some(res),
                Err(err) => panic!("can't parse command line argument {} {}, {}", s, w, err)
              }
            }
            None => panic!("empty argument for command line option {}", s),
          }
        }
      }
    }
    if let Some(t) = s.strip_prefix("-") {
      if let Some(u) = t.strip_prefix(short) {
        let w = u.trim();
        if w.is_empty() {
          let s = s.clone();
          it.next();
          match it.next() {
            Some(w) => {
              match R::from_str(&w) {
                Ok(res) => return Some(res),
                Err(err) => panic!("can't parse command line argument {} {}, {}", s, w, err)
              }
            }
            None => panic!("empty argument for command line option {}", s),
          }
        }
        match R::from_str(w) {
          Ok(res) => {
            it.next();
            return Some(res);
          }
          Err(err) => panic!("can't parse command line argument {}, {}", s, err),
        }
      }
    }
  }
  None
}

pub struct CMDOptions {
  depth: usize,
}

impl CMDOptions {
  pub fn new<I: Iterator<Item = String>>(it: I) -> Self {
    let mut depth = 0;
    let mut p = it.peekable();
    loop {
      if let Some(d) = try_parse_option::<usize, _>(&mut p, "d", "depth") {
        depth = d;
        continue;
      }
      break;
    }
    CMDOptions { depth }
  }
}

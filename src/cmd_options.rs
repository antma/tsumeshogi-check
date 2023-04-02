use std::iter::{Iterator, Peekable};
use std::str::FromStr;

use log::LevelFilter;

fn try_parse_option<I: Iterator<Item = String>>(
  it: &mut Peekable<I>,
  short: &str,
  long: &str,
) -> bool {
  if let Some(s) = it.peek() {
    if let Some(t) = s.strip_prefix("--") {
      if t == long {
        it.next();
        return true;
      }
    }
  }
  if short.is_empty() {
    return false;
  }
  assert_eq!(short.len(), 1, "short option {} is too long", short);
  if let Some(s) = it.peek() {
    if let Some(t) = s.strip_prefix("-") {
      if t == short {
        it.next();
        return true;
      }
    }
  }
  false
}

fn try_parse_arg_option<R: FromStr<Err = impl std::fmt::Display>, I: Iterator<Item = String>>(
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
            Some(w) => match R::from_str(&w) {
              Ok(res) => return Some(res),
              Err(err) => panic!("can't parse command line argument {} {}, {}", s, w, err),
            },
            None => panic!("empty argument for command line option {}", s),
          }
        }
      }
    }
    if short.is_empty() {
      return None;
    }
    assert_eq!(short.len(), 1);
    if let Some(t) = s.strip_prefix("-") {
      if let Some(u) = t.strip_prefix(short) {
        let w = u.trim();
        if w.is_empty() {
          let s = s.clone();
          it.next();
          match it.next() {
            Some(w) => match R::from_str(&w) {
              Ok(res) => return Some(res),
              Err(err) => panic!("can't parse command line argument {} {}, {}", s, w, err),
            },
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

#[derive(Debug)]
pub struct CMDOptions {
  pub depth: usize,
  pub depth_extend: usize,
  pub output: String,
  pub format_target: bool,
  pub level_filter: LevelFilter,
  pub args: Vec<String>,
}

impl CMDOptions {
  pub fn new<I: Iterator<Item = String>>(it: I) -> Self {
    let mut depth = 0;
    let mut depth_extend = 0;
    let mut p = it.peekable();
    let mut format_target = false;
    let mut level_filter = LevelFilter::Error;
    let mut output = String::new();
    loop {
      if let Some(d) = try_parse_arg_option::<usize, _>(&mut p, "d", "depth") {
        depth = d;
        continue;
      }
      if let Some(e) = try_parse_arg_option::<usize, _>(&mut p, "e", "extend") {
        depth_extend = e;
        continue;
      }
      if let Some(o) = try_parse_arg_option::<String, _>(&mut p, "o", "output") {
        output = o;
        continue;
      }
      if try_parse_option(&mut p, "w", "warn") {
        level_filter = LevelFilter::Warn;
        continue;
      }
      if try_parse_option(&mut p, "i", "info") {
        level_filter = LevelFilter::Info;
        continue;
      }
      if try_parse_option(&mut p, "", "debug") {
        level_filter = LevelFilter::Debug;
        continue;
      }
      if try_parse_option(&mut p, "t", "format-target") {
        format_target = true;
        continue;
      }
      break;
    }
    CMDOptions {
      depth,
      depth_extend,
      output,
      format_target,
      level_filter,
      args: p.collect(),
    }
  }
}

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::fs::OpenOptions;

use tsumeshogi_check::cmd_options::CMDOptions;
use tsumeshogi_check::kif;
use tsumeshogi_check::psn;
use tsumeshogi_check::shogi::Position;
use tsumeshogi_check::tsume_search::search_ext;

use log::{debug, error, info, warn};
//use log::{debug, info};

fn process_psn(filename: &str) -> std::io::Result<()> {
  let dst = filename.strip_suffix("psn").unwrap();
  let mut dst = String::from(dst);
  dst.push_str("kif");
  let mut f = OpenOptions::new().write(true).create_new(true).open(&dst)?;
  let it = psn::PSNFileIterator::new(filename)?;
  for (game_no, a) in it.enumerate() {
    if a.is_err() {
      error!("Game #{}: {:?}", game_no + 1, a);
      break;
    }
    let a = a.unwrap();
    let g = psn::parse_psn_game(&a);
    match g {
      Err(err) => {
        error!("Game #{}: {:?}", game_no + 1, err);
        break;
      }
      Ok(g) => {
        let l = kif::game_to_lines(&g);
        for s in l {
          write!(f, "{}\n", s)?;
          f.flush()?;
        }
      }
    }
  }
  Ok(())
}

fn process_file(filename: &str, depth: usize) -> std::io::Result<()> {
  let file = File::open(filename)?;
  let reader = BufReader::new(file);
  for (test, line) in reader.lines().enumerate() {
    let line = line?;
    let pos = Position::parse_sfen(&line);
    if pos.is_err() {
      error!(
        "Test #{}: fail to parse SFEN. {}",
        test + 1,
        pos.err().unwrap()
      );
      continue;
    }
    let pos = pos.unwrap();
    match search_ext(pos, depth, true) {
      Some(res) => {
        if res < depth as i32 {
          warn!(
            "Found faster mate in {} move(s). Test #{}, sfen: {}",
            res,
            test + 1,
            line
          );
        }
      }
      _ => {
        error!(
          "Mate in {} moves is not found. Test #{}, sfen: {}",
          depth,
          test + 1,
          line
        );
        panic!("");
      }
    }
    if (test + 1) % 1000 == 0 {
      info!("{} positions were processed.", test + 1);
    }
  }
  Ok(())
}

fn main() -> std::io::Result<()> {
  let opts = CMDOptions::new(std::env::args().skip(1));
  env_logger::builder()
    .filter_level(opts.level_filter)
    .format_target(opts.format_target)
    .init();
  debug!("{:?}", opts);
  if let Some(filename) = opts.args.into_iter().next() {
    if filename.ends_with(".psn") {
      process_psn(&filename)?;
    } else {
      process_file(&filename, opts.depth)?;
    }
  }
  Ok(())
}

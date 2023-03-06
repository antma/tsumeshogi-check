use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use tsumeshogi_check::shogi_rules::Position;
use tsumeshogi_check::tsume_search::search_ext;

use log::{error, info, warn};
//use log::{debug, info};

fn process_file(filename: &str, depth: usize) -> std::io::Result<()> {
  let file = File::open(filename)?;
  let reader = BufReader::new(file);
  for (test, line) in reader.lines().enumerate() {
    let line = line?;
    let pos = Position::parse_sfen(&line).unwrap();
    match search_ext(pos, depth, true, false) {
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
    if test % 1000 == 0 {
      info!("{} positions were processed.", test);
    }
  }
  Ok(())
}

fn main() -> std::io::Result<()> {
  env_logger::init();
  process_file("mate3.sfen", 3)
}

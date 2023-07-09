use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

use game::Game;
use shogi::{game, moves, Position};
use tsume_search::Search;
use tsumeshogi_check::cmd_options::CMDOptions;
use tsumeshogi_check::{psn, shogi, timer, tsume_search};

use log::{debug, error, info, warn};

const OVERWRITE_DESTINATION_FILE: bool = true;
const BUF_SIZE: usize = 1 << 16;
const FLUSH_INTERVAL: f64 = 10.0;

fn open_destination_file(dst: &str) -> std::io::Result<File> {
  if OVERWRITE_DESTINATION_FILE {
    File::create(dst)
  } else {
    std::fs::OpenOptions::new().create_new(true).open(dst)
  }
}

fn open_destination_writer(dst: &str) -> std::io::Result<BufWriter<File>> {
  let f = open_destination_file(dst)?;
  Ok(BufWriter::with_capacity(BUF_SIZE, f))
}

fn process_psn(filename: &str) -> std::io::Result<()> {
  let dst = filename.strip_suffix("psn").unwrap();
  let mut dst = String::from(dst);
  dst.push_str("kif");
  let mut f = open_destination_writer(&dst)?;
  let it = psn::PSNFileIterator::new(filename)?;
  let kb = shogi::kif::KIFBuilder::default();
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
        let s = kb.game_to_kif(&g, None);
        write!(f, "{}", s)?;
        f.flush()?;
      }
    }
  }
  Ok(())
}

#[derive(PartialEq, Debug)]
enum Format {
  Unknown,
  Kif,
  Sfen,
}

fn get_file_format(filename: &str) -> Format {
  if filename.ends_with(".kif") {
    Format::Kif
  } else if filename.ends_with(".sfen") {
    Format::Sfen
  } else {
    Format::Unknown
  }
}

fn process_file(filename: &str, opts: &CMDOptions) -> std::io::Result<()> {
  let kb = shogi::kif::KIFBuilder::default();
  let depth = opts.depth;
  let depth_extend = opts.depth_extend;
  let output_filename = &opts.output_filename;
  let output_format = get_file_format(output_filename);
  let id = filename.strip_suffix(".sfen").unwrap();
  let file = File::open(filename)?;
  let reader = BufReader::new(file);
  let mut nodes = 0;
  let mut writer = if output_format == Format::Unknown {
    None
  } else {
    let w = open_destination_writer(&output_filename)?;
    Some(w)
  };
  let mut writer = writer.as_mut();
  let allow_futile_drops = false;
  let mut s = Search::new(allow_futile_drops);
  let mut ttt = timer::Timer::new();
  for (test, line) in reader.lines().enumerate() {
    let line = line?;
    let test = test + 1;
    if test <= opts.skip {
      continue;
    }
    let pos = Position::parse_sfen(&line);
    if pos.is_err() {
      error!("Test #{}: fail to parse SFEN. {}", test, pos.err().unwrap());
      continue;
    }
    let mut pos = pos.unwrap();
    if pos.side < 0 {
      pos.swap_sides();
    }
    assert!(pos.side > 0);
    pos.move_no = 1;
    s.reset();
    match s.iterative_search(&mut pos, 1, depth) {
      Some(res) => {
        if res < depth as i32 {
          warn!(
            "Found faster mate in {} move(s). Test #{}, sfen: {}",
            res, test, line
          );
          nodes += s.nodes;
        }
      }
      _ => {
        error!(
          "Mate in {} moves is not found. Test #{}, sfen: {}",
          depth, test, line
        );
        panic!("");
      }
    }
    if let Some(ref mut writer) = writer {
      if let Some(p) = s.get_pv_from_hash(&mut pos) {
        if let Some(t) = s.is_unique_mate(&mut pos, &p, depth_extend) {
          warn!(
            "Tsume in {} moves isn't unique. Test #{}, sfen: {}, line: {}",
            t,
            test,
            pos.to_string(),
            moves::moves_to_kif(&p, 1)
          );
        } else {
          match output_format {
            //https://www.chessprogramming.org/Extended_Position_Description
            Format::Sfen => write!(
              writer,
              "{} c0 \"{}\"; id {}-{}; acn {};\n",
              pos,
              moves::moves_to_kif(&p, 1),
              id,
              test,
              s.nodes
            )?,
            Format::Kif => {
              let mut game = Game::default();
              game.set_header(String::from("event"), format!("{}-{}", id, test));
              game.moves = p;
              assert!(pos.side > 0);
              let s = kb.game_to_kif(&game, Some(&pos));
              write!(writer, "{}", s)?;
            }
            Format::Unknown => panic!("unhandled output format"),
          }
          if ttt.elapsed() > FLUSH_INTERVAL {
            writer.flush()?;
            ttt = timer::Timer::new();
          }
        }
      } else {
        error!(
          "Can't restore PV from hash table. Test #{}, sfen: {}",
          test, line
        );
      }
    }
    nodes += s.nodes;
    if test % 1000 == 0 {
      info!("{} positions were processed.", test);
    }
  }
  info!("{} nodes", nodes);
  s.log_stats();
  Ok(())
}

fn process_kif(filename: &str, opts: &CMDOptions) -> std::io::Result<()> {
  let kb = shogi::kif::KIFBuilder::default();
  let depth = opts.depth;
  let depth_extend = opts.depth_extend;
  let output_filename = &opts.output_filename;
  let output_format = get_file_format(output_filename);
  if output_format == Format::Unknown {
    error!("unknown output format for '{}'", output_filename);
    return Ok(());
  }
  let mut nodes = 0;
  let mut writer = open_destination_writer(&output_filename)?;
  let allow_futile_drops = false;
  let mut s = Search::new(allow_futile_drops);
  let mut ttt = timer::Timer::new();
  let it = shogi::kif::kif_file_iterator(filename)?;
  for (game_no, a) in it.enumerate() {
    if a.is_err() {
      error!("Game #{}: {:?}", game_no + 1, a);
      break;
    }
    let a = a.unwrap();
    let g = kb.parse_kif_game(&a);
    match g {
      Err(err) => {
        error!("Game #{}: {:?}", game_no + 1, err);
        break;
      }
      Ok(g) => {
        info!(
          "Game #{}: {}, {} moves",
          game_no + 1,
          g.to_short_string(),
          g.moves.len()
        );
        let mut pos = Position::default();
        for mv in g.moves.iter() {
          let move_no = pos.move_no;
          if move_no >= 20 {
            let swapped = pos.side < 0;
            let mut pos = pos.clone();
            if swapped {
              pos.swap_sides();
            }
            assert!(pos.side > 0);
            pos.move_no = 1;
            s.reset();
            match s.iterative_search(&mut pos, 1, depth) {
              Some(res) => {
                if let Some(p) = s.get_pv_from_hash(&mut pos) {
                  if p.first().unwrap() == mv {
                    info!(
                      "Tsume in {} moves was found, pos: {}, game: {}, move: {}",
                      res,
                      pos.to_string(),
                      game_no + 1,
                      move_no
                    );
                  } else {
                    if let Some(t) = s.is_unique_mate(&mut pos, &p, depth_extend) {
                      warn!(
                        "Tsume in {} moves isn't unique, sfen: {}, game: {}, move: {}",
                        t,
                        pos.to_string(),
                        game_no + 1,
                        move_no,
                      );
                    } else {
                      match output_format {
                        Format::Kif => {
                          let mut game = Game::default();
                          //game.set_header(String::from("event"), format!("{}-{}", id, test));
                          let (sente, gote) = if swapped {
                            ("gote", "sente")
                          } else {
                            ("sente", "gote")
                          };
                          let sente = sente.to_owned();
                          let gote = gote.to_owned();
                          game.set_header("sente".to_owned(), g.get_header(&sente).clone());
                          game.set_header("gote".to_owned(), g.get_header(&gote).clone());
                          game.copy_header(&g, "event");
                          game.copy_header(&g, "date");
                          game.copy_header(&g, "control");
                          game.copy_header(&g, "handicap");
                          game.moves = p;
                          assert!(pos.side > 0);
                          let s = kb.game_to_kif(&game, Some(&pos));
                          write!(writer, "{}", s)?;
                        }
                        _ => panic!("unhandled output format {:?}", output_format),
                      }
                    }
                  }
                }
              }
              None => (),
            }
            nodes += s.nodes;
          }
          pos.do_move(mv);
        }
      }
    }
    if ttt.elapsed() > FLUSH_INTERVAL {
      writer.flush()?;
      ttt = timer::Timer::new();
    }
  }
  info!("{} nodes", nodes);
  s.log_stats();
  Ok(())
}

fn main() -> std::io::Result<()> {
  let opts = CMDOptions::new(std::env::args().skip(1));
  env_logger::builder()
    .filter_level(opts.level_filter)
    .format_target(opts.format_target)
    .init();
  debug!("{:?}", opts);
  if let Some(filename) = opts.args.iter().next() {
    if filename.ends_with(".psn") {
      process_psn(&filename)?;
    } else if filename.ends_with(".sfen") {
      process_file(&filename, &opts)?;
    } else if filename.ends_with(".kif") {
      process_kif(&filename, &opts)?;
    }
  }
  Ok(())
}

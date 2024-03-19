use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::iter;

use game::Game;
use shogi::{game, moves, pgn, psn, Position};
use tsumeshogi_check::cmd_options::CMDOptions;
use tsumeshogi_check::{io, search, shogi, timer};

use log::{debug, error, info, warn};

const OVERWRITE_DESTINATION_FILE: bool = true;
const BUF_SIZE: usize = 1 << 16;

fn open_destination_writer(dst: &str) -> std::io::Result<BufWriter<File>> {
  let f = io::open_destination_file(dst, OVERWRITE_DESTINATION_FILE)?;
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

fn process_pgn(filename: &str) -> std::io::Result<()> {
  let it = pgn::PGNFileIterator::new(filename)?;
  let kb = shogi::kif::KIFBuilder::default();
  for (game_no, a) in it.enumerate() {
    if a.is_err() {
      error!("Game #{}: {:?}", game_no + 1, a);
      break;
    }
    let a = a.unwrap();
    let g = pgn::parse_pgn_game(&a);
    match g {
      Err(err) => {
        error!("Game #{}: {:?}", game_no + 1, err);
        break;
      }
      Ok(g) => {
        let s = kb.game_to_kif(&g, None);
        let dst = format!("{}.kif", game_no + 1);
        let mut f = open_destination_writer(&dst)?;
        write!(f, "{}", s)?;
        f.flush()?;
      }
    }
  }
  Ok(())
}

struct OutputStream<'a> {
  kb: shogi::kif::KIFBuilder,
  writers: io::PoolOfDestinationFiles<'a>,
  output_format: Format,
  puzzles: u32,
}

impl<'a> OutputStream<'a> {
  fn new(output_filename: &'a str) -> Option<Self> {
    let kb = shogi::kif::KIFBuilder::default();
    let output_format = get_file_format(output_filename);
    match output_format {
      Format::Unknown => {
        error!("unknown output format for '{}'", output_filename);
        return None;
      }
      Format::Sfen | Format::Kif => (),
    }
    let writers = io::PoolOfDestinationFiles::new(&output_filename, OVERWRITE_DESTINATION_FILE);
    Some(Self {
      kb,
      writers,
      output_format,
      puzzles: 0,
    })
  }
  fn write_puzzle(
    &mut self,
    res: u8,
    g: &Game,
    pos: &Position,
    pv: Vec<moves::Move>,
    swapped: bool,
    nodes: u64,
  ) -> std::io::Result<()> {
    self.puzzles += 1;
    match self.output_format {
      Format::Kif => {
        let mut game = Game::default();
        let (sente, gote) = if swapped {
          ("gote", "sente")
        } else {
          ("sente", "gote")
        };
        let sente = sente.to_owned();
        let gote = gote.to_owned();
        if let Some(u) = g.get_header(&sente) {
          game.set_header("sente".to_owned(), u.clone());
        }
        if let Some(u) = g.get_header(&gote) {
          game.set_header("gote".to_owned(), u.clone());
        }
        for key in vec!["event", "date", "location", "control", "handicap"] {
          game.copy_header(&g, key);
        }
        game.moves = pv;
        assert!(pos.side > 0);
        let s = self.kb.game_to_kif(&game, Some(&pos));
        self.writers.write_str(res as u32, &s)
      }
      Format::Sfen => {
        //https://www.chessprogramming.org/Extended_Position_Description
        let mut s = format!(
          "{} c0 \"{}\"; acn {};",
          pos,
          moves::moves_to_kif(&pv, 1),
          nodes
        );
        if let Some(u) = g.get_header(&"id".to_owned()) {
          s.push_str(&format!(" id \"{}\";", u));
        }
        s.push('\n');
        self.writers.write_str(res as u32, &s)
      }
      _ => panic!("unhandled output format {:?}", self.output_format),
    }
  }
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
  let tt = timer::Timer::new();
  let depth = opts.depth;
  let mut output_stream = OutputStream::new(&opts.output_filename).unwrap();
  let id = filename.strip_suffix(".sfen").unwrap();
  let file = File::open(filename)?;
  let reader = BufReader::new(file);
  let mut s = search::Search::new(opts.cache_memory_bytes);
  let mut g = Game::default();
  for (test, line) in reader.lines().enumerate() {
    let line = line?;
    let test = test + 1;
    if test < opts.skip {
      continue;
    }
    log::debug!("Test #{}: fen = {}", test, line);
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
    let nodes = s.nodes;
    let (res, pv) = s.search(&mut pos, depth as u8);
    s.hashes_clear();
    if res.is_some() {
      let res = res.unwrap();
      if res < depth as u8 {
        warn!(
          "Found faster mate in {} move(s). Test #{}, sfen: {}",
          res, test, line
        );
      }
      if let Some(p) = pv {
        let swapped = false;
        g.set_header(String::from("id"), format!("{}-{}", id, test));
        output_stream.write_puzzle(res, &g, &pos, p, swapped, s.nodes - nodes)?;
      } else {
        warn!(
          "Tsume in {} moves isn't unique. Test #{}, sfen: {}",
          res, test, pos,
        );
      }
    } else {
      error!(
        "Mate in {} moves is not found. Test #{}, sfen: {}",
        depth, test, line
      );
    }
    //nodes += s.nodes;
    if test % 1000 == 0 {
      info!(
        "{} positions were processed, {} puzzles",
        test, output_stream.puzzles
      );
    }
  }
  s.log_stats(output_stream.puzzles, tt.elapsed());
  Ok(())
}

fn process_kif(filename: &str, opts: &CMDOptions) -> std::io::Result<()> {
  let tt = timer::Timer::new();
  let depth = opts.depth;
  let mut output_stream = OutputStream::new(&opts.output_filename).unwrap();
  let mut s = search::Search::new(opts.cache_memory_bytes);
  let it = shogi::kif::kif_file_iterator(filename)?;
  for (game_no, a) in it.enumerate() {
    let game_no = game_no + 1;
    if a.is_err() {
      error!("Game #{}: {:?}", game_no, a);
      break;
    }
    if game_no < opts.skip {
      continue;
    }
    let a = a.unwrap();
    let g = output_stream.kb.parse_kif_game(&a);
    match g {
      Err(err) => {
        error!("Game #{}: {:?}", game_no, err);
        break;
      }
      Ok(g) => {
        info!(
          "Game #{}: {}, {} moves",
          game_no,
          g.to_short_string(),
          g.moves.len()
        );
        for current_side in iter::once(1i8).chain(iter::once(-1i8)) {
          s.hashes_clear();
          let mut pos = Position::default();
          let mut examined_positions_hashes = std::collections::HashSet::new();
          for mv in &g.moves {
            let move_no = pos.move_no;
            if move_no >= 20 && pos.side == current_side {
              if !examined_positions_hashes.insert(pos.hash) {
                info!(
                  "Position after move {} has been already examined (skipping)",
                  pos.move_no
                );
              } else {
                let swapped = pos.side < 0;
                let mut pos = pos.clone();
                let mut cur_move = mv.clone();
                if swapped {
                  pos.swap_sides();
                  cur_move.swap_side();
                }
                assert!(pos.side > 0);
                pos.move_no = 1;
                //s.hashes_retain(depth as u8);
                let nodes = s.nodes;
                let (res, pv) = s.search(&mut pos, depth as u8);
                let m = s.hashes_approximate_used_memory();
                if m > (1 << 29) {
                  warn!(
                    "Hashes used about {:.03} Mib, game {}, move {}, fen: {}",
                    m as f64 / ((1 << 20) as f64),
                    game_no,
                    move_no,
                    pos
                  );
                }
                if res.is_some() {
                  let res = res.unwrap();
                  if let Some(p) = pv {
                    if *p.first().unwrap() == cur_move {
                      info!(
                        "Tsume in {} moves was found and played, pos: {}, game: {}, move: {}",
                        res, pos, game_no, move_no
                      );
                    } else {
                      output_stream.write_puzzle(res, &g, &pos, p, swapped, s.nodes - nodes)?;
                    }
                  } else {
                    info!(
                      "Tsume in {} moves isn't unique, sfen: {}, game: {}, move: {}",
                      res, pos, game_no, move_no,
                    );
                  }
                }
              }
            }
            pos.do_move(mv);
            if current_side * pos.side < 0 {
              if !pos.is_check() {
                s.hashes_clear();
              }
            }
          }
        }
      }
    }
  }
  s.log_stats(output_stream.puzzles, tt.elapsed());
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
    } else if filename.ends_with(".pgn") {
      process_pgn(&filename)?;
    }
  }
  Ok(())
}

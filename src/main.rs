use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

use game::Game;
use shogi::{game, moves, Position};
use tsumeshogi_check::cmd_options::CMDOptions;
use tsumeshogi_check::{io, psn, search, shogi, timer, tsume_search};

use log::{debug, error, info, warn};

const OVERWRITE_DESTINATION_FILE: bool = true;
const BUF_SIZE: usize = 1 << 16;
const FLUSH_INTERVAL: f64 = 10.0;

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
  let mut s = tsume_search::Search::new(allow_futile_drops);
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

struct OutputStream<'a> {
  kb: shogi::kif::KIFBuilder,
  writers: io::PoolOfDestinationFiles<'a>,
  output_format: Format,
}

impl<'a> OutputStream<'a> {
  fn new(output_filename: &'a str) -> Option<Self> {
    let kb = shogi::kif::KIFBuilder::default();
    let output_format = get_file_format(output_filename);
    if output_format == Format::Unknown {
      error!("unknown output format for '{}'", output_filename);
      return None;
    }
    let writers = io::PoolOfDestinationFiles::new(&output_filename, OVERWRITE_DESTINATION_FILE);
    Some(Self {
      kb,
      writers,
      output_format,
    })
  }
  fn write_puzzle(
    &mut self,
    res: u8,
    g: &Game,
    pos: &Position,
    pv: Vec<moves::Move>,
    swapped: bool,
  ) -> std::io::Result<()> {
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
        game.set_header("sente".to_owned(), g.get_header(&sente).clone());
        game.set_header("gote".to_owned(), g.get_header(&gote).clone());
        for key in vec!["event", "date", "location", "control", "handicap"] {
          game.copy_header(&g, key);
        }
        game.moves = pv;
        assert!(pos.side > 0);
        let s = self.kb.game_to_kif(&game, Some(&pos));
        self.writers.write_str(res as u32, &s)
      }
      _ => panic!("unhandled output format {:?}", self.output_format),
    }
  }
}

fn process_kif(filename: &str, opts: &CMDOptions) -> std::io::Result<()> {
  let tt = timer::Timer::new();
  let depth = opts.depth;
  let mut output_stream = OutputStream::new(&opts.output_filename).unwrap();
  let mut s = search::Search::default();
  let it = shogi::kif::kif_file_iterator(filename)?;
  for (game_no, a) in it.enumerate() {
    s.hashes_clear();
    if a.is_err() {
      error!("Game #{}: {:?}", game_no + 1, a);
      break;
    }
    let a = a.unwrap();
    let g = output_stream.kb.parse_kif_game(&a);
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
        for mv in &g.moves {
          let move_no = pos.move_no;
          if move_no >= 20 {
            let swapped = pos.side < 0;
            let mut pos = pos.clone();
            let mut cur_move = mv.clone();
            if swapped {
              pos.swap_sides();
              cur_move.swap_side();
            }
            assert!(pos.side > 0);
            pos.move_no = 1;
            s.hashes_retain(depth as u8);
            let (res, pv) = s.search(&mut pos, depth as u8);
            if res.is_some() {
              let res = res.unwrap();
              if let Some(p) = pv {
                if *p.first().unwrap() == cur_move {
                  info!(
                    "Tsume in {} moves was found and played, pos: {}, game: {}, move: {}",
                    res,
                    pos,
                    game_no + 1,
                    move_no
                  );
                } else {
                  output_stream.write_puzzle(res, &g, &pos, p, swapped)?;
                }
              } else {
                info!(
                  "Tsume in {} moves isn't unique, sfen: {}, game: {}, move: {}",
                  res,
                  pos,
                  game_no + 1,
                  move_no,
                );
              }
            }
          }
          pos.do_move(mv);
        }
      }
    }
  }
  info!("{} nodes", s.nodes);
  info!("{:.3} nps", s.nodes as f64 / tt.elapsed());
  //s.log_stats();
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

use fumen::{Fumen, CellColor};
use clap::Parser;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
  /// CSV file from `spin` output of the solution finder
  #[arg(short, long)]
  file: String,
  /// blocks before T
  #[arg(short, long)]
  before_t: String,
  /// blocks after T
  #[arg(short, long)]
  after_t: String,
  /// lines
  #[arg(short, long)]
  line: u8,
}

fn letter_to_cellcolor(letter: char) -> CellColor {
  match letter {
    'I' => CellColor::I,
    'O' => CellColor::O,
    'T' => CellColor::T,
    'S' => CellColor::S,
    'Z' => CellColor::Z,
    'J' => CellColor::J,
    'L' => CellColor::L,
    _ => panic!("Invalid letter: {}", letter),
  }
}

fn main() {
  let args = Args::parse();
  let mut csv = csv::Reader::from_path(args.file).expect("Failed to open CSV file");
  for result in csv.records() {
    let record = result.expect("Failed to read CSV record");
    let fumen_str = record.get(0).unwrap().strip_prefix("http://fumen.zui.jp/?");
    if fumen_str.is_none() {
      continue;
    }
    let line = record.get(7).unwrap().parse::<u8>().unwrap();
    if line > args.line {
      continue;
    }
    let fumen = Fumen::decode(fumen_str.unwrap()).expect("Failed to decode fumen");
    let field = fumen.pages[0].field;
    assert_eq!(fumen.pages.len(), 1);
    let mut empty = [true; 10];
    let mut has_uncontinuous = false;
    'has_uncontinuous: for line in field.iter().rev() {
      if line.iter().all(|&cell| cell != CellColor::Empty) {
        continue;
      }
      let mut continuous_empties = vec![];
      let mut last_empty = false;
      let mut empty_start = 0;
      for (i, &cell) in line.iter().enumerate() {
        if cell == CellColor::Empty {
          if last_empty {
            continue;
          }
          last_empty = true;
          empty_start = i;
        } else {
          if !last_empty {
            continue;
          }
          last_empty = false;
          continuous_empties.push((empty_start, i));
        }
      }
      for (start, end) in continuous_empties {
        let mut has_empty = false;
        for i in start..end {
          if empty[i] {
            has_empty = true;
            break;
          }
        }
        if !has_empty {
          has_uncontinuous = true;
          break 'has_uncontinuous;
        }
      }
      empty = line.iter().map(|&cell| cell == CellColor::Empty).collect::<Vec<_>>()[..10].try_into().unwrap();
    }
    if has_uncontinuous {
      continue;
    }
    let mut before_t = args.before_t.to_uppercase().chars()
      .map(|letter| (letter_to_cellcolor(letter), 0))
      .collect::<HashMap<_, _>>();
    for (i, line) in field.iter().enumerate() {
      for (j, cell) in line.iter().enumerate() {
        if !before_t.contains_key(cell) || before_t[cell] == 2 {
          continue;
        }
        let has_support = if i == 0 {
          true
        } else {
          let below = field[i-1][j];
          if below == CellColor::Grey {
            true
          } else {
            before_t.contains_key(&below) && below != *cell
          }
        };
        before_t.insert(*cell, if has_support { 2 } else { 1 });
      }
    }
    if before_t.values().any(|&v| v == 1) {
      continue;
    }
    let mut after_t = args.after_t.to_uppercase().chars()
      .map(|letter| (letter_to_cellcolor(letter), 0))
      .collect::<HashMap<_, _>>();
    let sequence = args.before_t.to_uppercase() + &args.after_t.to_uppercase();
    let sequence = sequence.chars().map(letter_to_cellcolor).collect::<Vec<_>>();
    // assert each block in after_t has a block before it in sequence in the same column
    for (i, line) in field.iter().enumerate() {
      for (j, cell) in line.iter().enumerate() {
        if !after_t.contains_key(cell) || after_t[cell] == 2 {
          continue;
        }
        let has_support = if i == 0 {
          true
        } else {
          let below = field[i-1][j];
          if below == CellColor::Grey {
            true
          } else if below == CellColor::T || below == CellColor::Empty {
            false
          } else {
            let index_below = sequence.iter().position(|&c| c == below);
            if index_below.is_none() {
              panic!("{:?} not found in sequence", below);
            }
            let index_self = sequence.len() - sequence.iter().rev().position(|&c| c == *cell).unwrap() - 1;
            index_below.unwrap() < index_self
          }
        };
        after_t.insert(*cell, if has_support { 2 } else { 1 });
      }
    }
    if after_t.values().any(|&v| v == 1) {
      continue;
    }
    println!("http://fumen.zui.jp/?{}", fumen_str.unwrap());
  }
}

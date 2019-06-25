use std::io::*;
use std::str::FromStr;

#[derive(PartialEq, Clone)]
enum Piece {
  White,
  Black,
}

#[derive(PartialEq, Clone)]
enum BoardState {
  Playing,
  BlackWin,
  WhiteWin,
  Draw,
}

#[derive(PartialEq, Clone)]
enum LineState {
  BlackWin,
  WhiteWin,
  Playing,
  Draw,
}

#[derive(PartialEq, Clone)]
enum Turn {
  White,
  Black,
}

struct Valuation {
  board: CrossAndKnotsBoard,
  score: i32,
}

#[derive(Clone)]
struct CrossAndKnotsBoard {
  cells: Vec<Option<Piece>>,
}

struct WithIndex<T> {
  elem: T,
  index: usize,
}

impl CrossAndKnotsBoard {
  pub fn new() -> Self {
    let mut cells = vec![];
    for _ in 0..3 * 3 {
      cells.push(None);
    }
    CrossAndKnotsBoard { cells }
  }

  pub fn can_put(&self, index: usize) -> bool {
    self.cells[index] == None
  }

  pub fn put(&mut self, index: usize, piece: Piece) {
    self.cells[index] = Some(piece);
  }

  fn count_blank(&self) -> u8 {
    let mut count = 0;
    for cell in &self.cells {
      if let None = cell {
        count += 1;
      }
    }
    count
  }

  pub fn who_can_put_next_piece(&self) -> Turn {
    match self.count_blank() % 2 {
      1 => Turn::White,
      _ => Turn::Black,
    }
  }

  pub fn get_next_best_board(&self, depth: i32) -> Valuation {
    let boards = self.get_next_all_pattern_board(&self);
    if boards.len() == 0 {
      return self.calc_valuation(depth);
    }

    let vals: Vec<WithIndex<Valuation>> = boards
      .iter()
      .map(|board| {
        let _val = board.calc_valuation(depth);
        match _val.board.board_state() == BoardState::Playing {
          true => _val.board.get_next_best_board(depth + 1),
          false => _val,
        }
      })
      .enumerate()
      .map(|(index, elem)| WithIndex { elem, index })
      .collect();

    // minmaxの由来である
    // 次石置くのが自分なら最大を取る
    // 次石置くのが相手なら最小を取る (最小は相手が最も有利な手の為
    let val: Option<WithIndex<Valuation>> = match self.who_can_put_next_piece() {
      Turn::White => vals.into_iter().fold(None, |max, v| match &max {
        Some(WithIndex { elem, index: _ }) => match v.elem.score > elem.score {
          true => Some(v),
          false => max,
        },
        None => Some(v),
      }),
      Turn::Black => vals.into_iter().fold(None, |max, v| match &max {
        Some(WithIndex { elem, index: _ }) => match v.elem.score > elem.score {
          true => max,
          false => Some(v),
        },
        None => Some(v),
      }),
    };
    let val_with_index = val.unwrap();
    Valuation {
      score: val_with_index.elem.score,
      board: boards[val_with_index.index].clone(),
    }
  }

  fn get_next_all_pattern_board(&self, board: &CrossAndKnotsBoard) -> Vec<CrossAndKnotsBoard> {
    if board.board_state() != BoardState::Playing {
      return Vec::new();
    }
    let mut boards = vec![];
    match board.who_can_put_next_piece() {
      Turn::Black => {
        for i in 0..9 {
          if board.can_put(i) {
            let mut clone = board.clone();
            clone.put(i, Piece::Black);
            boards.push(clone);
          }
        }
      }
      Turn::White => {
        for i in 0..9 {
          if board.can_put(i) {
            let mut clone = board.clone();
            clone.put(i, Piece::White);
            boards.push(clone);
          }
        }
      }
    }
    boards
  }

  fn calc_valuation(&self, depth: i32) -> Valuation {
    let win_score = 99;

    match self.board_state() {
      BoardState::WhiteWin => Valuation {
        board: self.clone(),
        // 勝つ場合は短いターンの方がスコアを高くする
        score: win_score - depth,
      },
      BoardState::BlackWin => Valuation {
        board: self.clone(),
        // 負ける場合は延命した方がスコアを高くする
        score: win_score * -1 + depth,
      },
      // それ以外の場合は深さをとりあえず返す。
      _ => Valuation {
        board: self.clone(),
        score: depth,
      },
    }
  }

  fn board_state(&self) -> BoardState {
    for col_num in 0..3 {
      let raw_num = col_num * 3;

      // row
      match self.judge(
        &self.cells[raw_num],
        &self.cells[raw_num + 1],
        &self.cells[raw_num + 2],
      ) {
        LineState::BlackWin => return BoardState::BlackWin,
        LineState::WhiteWin => return BoardState::WhiteWin,
        _ => (),
      };

      // column
      match self.judge(
        &self.cells[col_num],
        &self.cells[col_num + 3],
        &self.cells[col_num + 6],
      ) {
        LineState::BlackWin => return BoardState::BlackWin,
        LineState::WhiteWin => return BoardState::WhiteWin,
        _ => (),
      };
    }
    // 左上から右下にかける線
    match self.judge(&self.cells[0], &self.cells[4], &self.cells[8]) {
      LineState::BlackWin => return BoardState::BlackWin,
      LineState::WhiteWin => return BoardState::WhiteWin,
      _ => (),
    };
    // 右上から左下にかける線
    match self.judge(&self.cells[2], &self.cells[4], &self.cells[6]) {
      LineState::BlackWin => return BoardState::BlackWin,
      LineState::WhiteWin => return BoardState::WhiteWin,
      _ => (),
    };
    for cell in &self.cells {
      if let None = cell {
        return BoardState::Playing;
      }
    }
    BoardState::Draw
  }

  fn judge(&self, a: &Option<Piece>, b: &Option<Piece>, c: &Option<Piece>) -> LineState {
    if a.is_none() || b.is_none() || c.is_none() {
      return LineState::Playing;
    }

    match a == b && b == c {
      true => match a {
        Some(Piece::White) => LineState::WhiteWin,
        Some(Piece::Black) => LineState::BlackWin,
        // ありえないケース
        None => LineState::Playing,
      },
      false => LineState::Draw,
    }
  }
}

fn get_cell_for_display(index: usize, cell: &Option<Piece>) -> String {
  match cell {
    Some(Piece::Black) => "x".to_string(),
    Some(Piece::White) => "o".to_string(),
    None => index.to_string(),
  }
}

fn show_board(board: &CrossAndKnotsBoard) {
  println!("------------");
  for i in 0..3 {
    let raw_num = i * 3;
    println!(
      "{} {} {}",
      get_cell_for_display(raw_num, &board.cells[raw_num]),
      get_cell_for_display(raw_num + 1, &board.cells[raw_num + 1]),
      get_cell_for_display(raw_num + 2, &board.cells[raw_num + 2])
    );
  }
  println!("------------");
}

fn read<T: FromStr>() -> T {
  let stdin = stdin();
  let stdin = stdin.lock();
  let token: String = stdin
    .bytes()
    .map(|c| c.expect("failed to read char") as char)
    .skip_while(|c| c.is_whitespace())
    .take_while(|c| !c.is_whitespace())
    .collect();

  token.parse().ok().expect("failed to parse token")
}

fn user_input(board: &CrossAndKnotsBoard) -> usize {
  loop {
    let index = read::<usize>();
    match board.can_put(index) {
      true => return index,
      false => {
        println!("You can not put piece on it index");
      }
    }
  }
}

fn is_start_by_player() -> bool {
  println!("Do you want to put the first piece?");
  println!("0: No, 1: Yes");
  loop {
    let answer = read::<i32>();
    match answer {
      0 => return false,
      1 => return true,
      _ => println!("0: No, 1: Yes"),
    }
  }
}

fn play_game() {
  let is_first_player = is_start_by_player();
  let mut board = CrossAndKnotsBoard::new();
  println!("Ready play game");
  show_board(&board);

  loop {
    match board.board_state() {
      BoardState::Playing => match board.who_can_put_next_piece() {
        Turn::White => match is_first_player {
          true => {
            println!("Your turn");
            let index = user_input(&board);
            board.put(index, Piece::White);
          }
          false => {
            println!("CPU turn");
            let tmp = board.get_next_best_board(0);
            println!("score: {:?}", tmp.score);
            board = tmp.board;
          }
        },
        Turn::Black => match is_first_player {
          false => {
            println!("Your turn");
            let index = user_input(&board);
            board.put(index, Piece::Black);
          }
          true => {
            println!("CPU turn");
            let tmp = board.get_next_best_board(0);
            println!("score: {:?}", tmp.score);
            board = tmp.board;
          }
        },
      },
      BoardState::Draw => {
        println!("Draw");
        break;
      }
      BoardState::BlackWin => {
        match is_first_player {
          true => println!("You lose"),
          false => println!("You win"),
        }
        break;
      }
      BoardState::WhiteWin => {
        match is_first_player {
          true => println!("You win"),
          false => println!("You lose"),
        }
        break;
      }
    };
    show_board(&board);
  }
}

fn main() {
  play_game();
}

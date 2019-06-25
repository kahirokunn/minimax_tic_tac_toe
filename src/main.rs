use std::io::*;
use std::str::FromStr;

// その盤面の状態を表します
#[derive(PartialEq, Clone)]
enum BoardState {
  Playing,  // プレイ中
  BlackWin, // 黒の勝ち
  WhiteWin, // 白の勝ち
  Draw,     // 引き分け
}

// 縦横斜めのラインの状態を表します
#[derive(PartialEq, Clone)]
enum LineState {
  BlackWin,
  WhiteWin,
  Playing,
  Draw,
}

// 誰のターンかを表します
#[derive(PartialEq, Clone)]
enum Turn {
  White,
  Black,
}

// 盤面とその盤面の価値を表します
struct Valuation {
  board: TicTacToeBoard,
  score: i32,
}

// 盤面のマス目の上に置く石を表します
#[derive(PartialEq, Clone)]
enum Piece {
  White,
  Black,
}

// マス目を表現します
// マス目に何も置かれてなければNone, 置けれていればSome<Piece>
type Cell = Option<Piece>;

// 盤面を表現します
// cellsは3*3のマルバツゲームの盤面を1次元配列にならしたものです.
#[derive(Clone)]
struct TicTacToeBoard {
  cells: Vec<Cell>,
}

// これは盤面とindexのペアを実現するために作りました.
// これはなくても良いけど、あると便利だったので.
struct WithIndex<T> {
  elem: T,
  index: usize,
}

impl TicTacToeBoard {
  pub fn new() -> Self {
    let mut cells = vec![];
    for _ in 0..3 * 3 {
      cells.push(None);
    }
    TicTacToeBoard { cells }
  }

  pub fn can_put(&self, index: usize) -> bool {
    self.cells[index] == None
  }

  pub fn put(&mut self, index: usize, piece: Piece) {
    self.cells[index] = Some(piece);
  }

  pub fn who_can_put_next_piece(&self) -> Turn {
    match self.count_blank() % 2 {
      1 => Turn::White,
      _ => Turn::Black,
    }
  }

  // 次に打つべきな最善手を取得します
  pub fn get_next_best_board(&self) -> TicTacToeBoard {
    self.minimax(0).board
  }

  fn minimax(&self, depth: i32) -> Valuation {
    let boards = self.get_next_all_pattern_board(&self);
    if boards.len() == 0 {
      return self.calc_valuation(depth);
    }

    let vals: Vec<WithIndex<Valuation>> = boards
      .iter()
      .map(|board| {
        let _val = board.calc_valuation(depth);
        match _val.board.board_state() == BoardState::Playing {
          true => _val.board.minimax(depth + 1),
          false => _val,
        }
      })
      .enumerate()
      .map(|(index, elem)| WithIndex { elem, index })
      .collect();

    // minimaxの由来である
    // 次石置くのが自分なら最大を取る
    // 次石置くのが相手なら最小を取る (最小は相手が最も有利な手の為
    let val: Option<WithIndex<Valuation>> = match self.who_can_put_next_piece() {
      // 最悪の手を取る
      Turn::White => vals.into_iter().fold(None, |min, v| match &min {
        Some(WithIndex {
          elem: min_elem,
          index: _,
        }) => match v.elem.score > min_elem.score {
          true => Some(v),
          false => min,
        },
        None => Some(v),
      }),
      // 最短で勝てる手を取る
      Turn::Black => vals.into_iter().fold(None, |max, v| match &max {
        Some(WithIndex {
          elem: max_elem,
          index: _,
        }) => match v.elem.score > max_elem.score {
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

  fn get_next_all_pattern_board(&self, board: &TicTacToeBoard) -> Vec<TicTacToeBoard> {
    if board.board_state() != BoardState::Playing {
      return Vec::new();
    }
    let mut boards = vec![];
    let piece = match board.who_can_put_next_piece() {
      Turn::Black => Piece::Black,
      Turn::White => Piece::White,
    };
    for i in 0..9 {
      if board.can_put(i) {
        let mut cloned_board = board.clone();
        cloned_board.put(i, piece.clone());
        boards.push(cloned_board);
      }
    }
    boards
  }

  // 盤面単位の評価関数
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

  fn count_blank(&self) -> u8 {
    let mut count = 0;
    for cell in &self.cells {
      if let None = cell {
        count += 1;
      }
    }
    count
  }

  fn board_state(&self) -> BoardState {
    for col_num in 0..3 {
      let raw_num = col_num * 3;

      // 横向きの線
      match self.judge_for_line(
        &self.cells[raw_num],
        &self.cells[raw_num + 1],
        &self.cells[raw_num + 2],
      ) {
        LineState::BlackWin => return BoardState::BlackWin,
        LineState::WhiteWin => return BoardState::WhiteWin,
        _ => (),
      };

      // 縦向きの線
      match self.judge_for_line(
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
    match self.judge_for_line(&self.cells[0], &self.cells[4], &self.cells[8]) {
      LineState::BlackWin => return BoardState::BlackWin,
      LineState::WhiteWin => return BoardState::WhiteWin,
      _ => (),
    };
    // 右上から左下にかける線
    match self.judge_for_line(&self.cells[2], &self.cells[4], &self.cells[6]) {
      LineState::BlackWin => return BoardState::BlackWin,
      LineState::WhiteWin => return BoardState::WhiteWin,
      _ => (),
    };
    // 全ての勝利パターンに該当せず、ここまですり抜けてきた
    // 何も置かれていないマスがあったらプレイ中
    for cell in &self.cells {
      if let None = cell {
        return BoardState::Playing;
      }
    }
    // 引き分け
    BoardState::Draw
  }

  // ラインの為の評価関数
  fn judge_for_line(&self, a: &Option<Piece>, b: &Option<Piece>, c: &Option<Piece>) -> LineState {
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

fn show_board(board: &TicTacToeBoard) {
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

fn user_input(board: &TicTacToeBoard) -> usize {
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
  let mut board = TicTacToeBoard::new();
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
            board = board.get_next_best_board();
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
            board = board.get_next_best_board();
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

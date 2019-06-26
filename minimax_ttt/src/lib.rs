extern crate wasm_bindgen;

use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
use std::iter::FromIterator;

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

fn num_to_cell(num: i32) -> Cell {
  match num {
    0 => None,
    1 => Some(Piece::White),
    2 => Some(Piece::Black),
    _ => panic!(format!("expected number type 0 or1 or 2 , got {}", num)),
  }
}

// and we'll implement FromIterator
impl FromIterator<i32> for TicTacToeBoard {
  fn from_iter<I: IntoIterator<Item = i32>>(iter: I) -> Self {
    let mut board = TicTacToeBoard::new();

    let mut index = 0;
    for num in iter {
      match num_to_cell(num) {
        Some(Piece::Black) => board.put(index, Piece::Black),
        Some(Piece::White) => board.put(index, Piece::White),
        None => (),
      }
      index += 1;
    }
    board
  }
}

fn cell_to_num(cell: Cell) -> i32 {
  match cell {
    None => 0,
    Some(Piece::White) => 1,
    Some(Piece::Black) => 2,
  }
}

impl TicTacToeBoard {
  pub fn to_json_able_mut(self) -> Vec<i32> {
    self
      .cells
      .into_iter()
      .map(|cell| cell_to_num(cell))
      .collect()
  }

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

#[wasm_bindgen]
pub fn get_next_best_board( js_objects: &JsValue ) -> JsValue {
  let board_source: Vec<i32> = js_objects.into_serde().unwrap();
  let board = TicTacToeBoard::from_iter(board_source.into_iter());
  JsValue::from_serde(&board.get_next_best_board().to_json_able_mut()).unwrap()
}

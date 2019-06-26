extern crate minimax_ttt;

use minimax_ttt::{Cell, Piece, TicTacToeBoard, BoardState, Turn};
use std::io::*;
use std::str::FromStr;

fn get_cell_for_display(index: usize, cell: &Cell) -> String {
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

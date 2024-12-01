use arrayvec::ArrayVec;
use mancala_board::{MancalaBoard, MoveResult, Side, Winner};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::Algorithm;

/// an attempt at getting some bitches
#[derive(Debug, Clone)]
pub struct BankBird1<const S: usize>(pub usize); // depth
impl<const S: usize> Algorithm<S> for BankBird1<S> {
    fn name(&self) -> String { format!("Bank Bird v1 (d:{})", self.0) }
    fn min_games(&self) -> usize { 1 }
    fn play_move(&mut self, board: &MancalaBoard<S>, side: Side) -> usize {
        fn recursive<const S: usize>(board: MancalaBoard<S>, side: Side, depth: usize, max_depth: usize) -> ArrayVec<Option<(usize, i32)>, S> {
            if depth >= max_depth { return ArrayVec::from_iter([Some((0, calculate_score_v1(&board)))]) }
            (0..S)
            .into_par_iter()
            .map(|i| {
                let mut board = board;
                let move_result = board.move_piece_kalah(side, i);
                match move_result {
                    MoveResult::Capture(cs, ci) => board.capture_kalah(cs, ci),
                    MoveResult::IllegalMove => return None,
                    _ => {}
                }
                if board.game_over() {
                    return Some((i, calculate_score_v1(&board)))
                }
                let next_side = if move_result.change_side() { !side } else { side };
                select_best(recursive(board, next_side, depth + 1, max_depth), !next_side).map(|v| (i, v.1))
            }).collect::<Vec<_>>().into_iter().collect()
        }
        let results = recursive(*board, side, 0, self.0);
        select_best(results, !side)
            .expect("at least a valid move").0
    }
    fn dyn_clone(&self) -> Box<dyn Algorithm<S>> { Box::new(self.clone()) }
}

fn select_best<const S: usize>(results: ArrayVec<Option<(usize, i32)>, S>, side: Side) -> Option<(usize, i32)> {
    let iter = results.iter().flatten();
    match !side {
        Side::Left => iter.max_by(|a,b| a.1.cmp(&b.1)),
        Side::Right => iter.min_by(|a,b| a.1.cmp(&b.1)),
    }.copied()
}

// I tried a bit, and 3 seems to be the best multiplier
const BANK_MULT: u32 = 3;

fn calculate_score_v1<const S: usize>(board: &MancalaBoard<S>) -> i32 {
    let total_win_score = BANK_MULT as i32 * (S * 2 * 10) as i32;

    let win_score = if board.game_over() {
        match board.winner() {
            Winner::Side(Side::Left) => total_win_score,
            Winner::Side(Side::Right) => -total_win_score,
            Winner::Tie => 0,
        }
    } else { 0 };

    win_score
    + (board.left.iter().sum::<u32>() + board.left_bank * BANK_MULT) as i32
    - (board.right.iter().sum::<u32>() + board.right_bank * BANK_MULT) as i32
}

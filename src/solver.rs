use rand::prelude::SliceRandom;
use rand::Rng;

use crate::exact_cover::{Matrix, H};
pub struct Solver {
    pub board: Vec<Vec<u32>>,
}

impl Solver {
    pub fn is_valid(&mut self) -> bool {
        for column in 0..9 {
            for row in 0..9 {
                let n = self.board[column][row];

                if !self.is_in_column(column, n)
                    && !self.is_in_row(row, n)
                    && !self.is_in_square(column, row, n)
                {
                    return false;
                }
            }
        }
        true
    }

    fn is_in_column(&mut self, column: usize, n: u32) -> bool {
        for y in 0..9 {
            if self.board[column][y] == n {
                return true;
            }
        }
        false
    }

    fn is_in_row(&mut self, row: usize, n: u32) -> bool {
        for x in 0..9 {
            if self.board[x][row] == n {
                return true;
            }
        }
        false
    }

    fn is_in_square(&mut self, column: usize, row: usize, n: u32) -> bool {
        for y in 0..3 {
            for x in 0..3 {
                if self.board[x + (column / 3) * 3][y + (row / 3) * 3] == n {
                    return true;
                }
            }
        }
        false
    }

    fn can_be_placed(&mut self, column: usize, row: usize, n: u32) -> bool {
        !self.is_in_column(column, n)
            && !self.is_in_row(row, n)
            && !self.is_in_square(column, row, n)
    }

    pub fn solve(&mut self) -> Vec<Vec<Vec<u32>>> {
        // let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        let mut rng = thread_rng();
        let mut solutions = vec![];
        self._solve(&mut rng, 0, 0);
        solutions
    }

    fn _solve(&mut self, rng: &mut impl Rng, current_column: usize, current_row: usize) -> bool {
        if current_row == 9 {
            return true;
        } else if current_column == 9 {
            self._solve(rng, 0, current_row + 1)
        } else if self.board[current_column][current_row] != 0 {
            self._solve(rng, current_column + 1, current_row)
        } else {
            let mut numbers_left: Vec<u32> = (1..=9).collect();
            numbers_left.shuffle(rng);

            while let Some(n) = numbers_left.pop() {
                if self.can_be_placed(current_column, current_row, n) {
                    println!("{} can be placed at {}, {}", n, current_column, current_row);
                    self.board[current_column][current_row] = n;
                    if self._solve(rng, current_column + 1, current_row) {
                        return true;
                    } else {
                        self.board[current_column][current_row] = 0;
                    }
                }
            }
            return false;
        }
    }
}

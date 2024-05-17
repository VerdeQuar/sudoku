use crate::dancing_links::Indexed;
use crate::exact_cover::{Cell, Matrix, MatrixSize, SolvingState};
use itertools::Itertools;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{thread_rng, SeedableRng};

use std::collections::HashSet;
pub struct Sudoku {
    pub choices: Vec<Choice>,
    constraints: Vec<Constraint>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Constraint {
    RowColumn { row: u32, column: u32 },
    RowNumber { row: u32, number: u32 },
    ColumnNumber { column: u32, number: u32 },
    SquareNumber { square: u32, number: u32 },
}

impl Constraint {
    pub fn all(n: u32) -> impl Iterator<Item = Constraint> {
        let row_column_iter = (0..n.pow(2))
            .cartesian_product(0..n.pow(2))
            .map(|(row, column)| Constraint::RowColumn { row, column });
        let row_number_iter = (0..n.pow(2))
            .cartesian_product(0..n.pow(2))
            .map(|(row, number)| Constraint::RowNumber { row, number });
        let column_number_iter = (0..n.pow(2))
            .cartesian_product(0..n.pow(2))
            .map(|(column, number)| Constraint::ColumnNumber { column, number });
        let square_number_iter = (0..n.pow(2))
            .cartesian_product(0..n.pow(2))
            .map(|(square, number)| Constraint::SquareNumber { square, number });

        row_column_iter
            .chain(row_number_iter)
            .chain(column_number_iter)
            .chain(square_number_iter)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Choice {
    pub row: u32,
    pub column: u32,
    pub square: u32,
    pub number: u32,
}

// Allow indexing column sizes by Cell
impl std::ops::Index<Cell> for Vec<Choice> {
    type Output = Choice;
    fn index(&self, index: Cell) -> &Choice {
        &self[index.get_index()]
    }
}

impl Choice {
    pub fn all(n: u32) -> impl Iterator<Item = Choice> {
        // let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        let mut rng = thread_rng();
        let mut row_range = (0..n.pow(2)).collect::<Vec<u32>>();
        row_range.shuffle(&mut rng);
        let mut column_range = (0..n.pow(2)).collect::<Vec<u32>>();
        column_range.shuffle(&mut rng);
        let mut number_range = (0..n.pow(2)).collect::<Vec<u32>>();
        number_range.shuffle(&mut rng);

        row_range
            .iter()
            .cartesian_product(column_range)
            .cartesian_product(number_range)
            .map(move |((row, column), number)| {
                let index = (row * n.pow(2)) + column;
                let square = ((index % n.pow(2)) / n) + (n * (index / (n.pow(3))));
                Choice {
                    row: *row,
                    column,
                    square,
                    number,
                }
            })
            .collect::<Vec<Choice>>()
            .into_iter()
    }

    fn satisfied_constraints(choice: &Choice) -> impl Iterator<Item = Constraint> {
        [
            Constraint::RowColumn {
                row: choice.row,
                column: choice.column,
            },
            Constraint::RowNumber {
                row: choice.row,
                number: choice.number,
            },
            Constraint::ColumnNumber {
                column: choice.column,
                number: choice.number,
            },
            Constraint::SquareNumber {
                square: choice.square,
                number: choice.number,
            },
        ]
        .into_iter()
    }
}
pub type Solution = Vec<Choice>;

impl<'a> Sudoku {
    pub fn new(n: u32, filled_values: impl IntoIterator<Item = Choice>) -> Self {
        let filled_values: Vec<Choice> = filled_values.into_iter().collect();

        let satisfied: HashSet<_> = filled_values
            .iter()
            .flat_map(Choice::satisfied_constraints)
            .collect();

        let filled_coordinates: HashSet<_> = filled_values
            .iter()
            .map(|choice| (choice.row, choice.column))
            .collect();

        let choices: Vec<Choice> = Choice::all(n)
            .filter(|c| !filled_coordinates.contains(&(c.row, c.column)))
            .collect();

        let constraints: Vec<Constraint> = Constraint::all(n)
            .filter(|c| !satisfied.contains(c))
            .collect();

        Self {
            choices,
            constraints,
        }
    }

    pub fn solve(&self, mut callback: impl FnMut(Solution) -> SolvingState) {
        let mut matrix = Matrix::new(MatrixSize {
            x: self.constraints.len(),
            y: self.choices.len(),
        });
        for choice in self.choices.iter() {
            let row = self
                .constraints
                .iter()
                .map(move |constraint| Choice::satisfied_constraints(choice).contains(constraint))
                .collect::<Vec<bool>>();
            matrix.add_row(&row);
        }

        matrix.solve(&mut |solution: crate::exact_cover::Solution| {
            return callback(solution.iter().map(|row| self.choices[*row]).collect());
        });
    }
}

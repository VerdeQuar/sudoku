use crate::dancing_links::{DoublyLinkedList, Indexed};

use std::{fmt, ops};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell(usize);

#[derive(Debug, Clone)]
pub struct Matrix {
    pub x: DoublyLinkedList<Cell>,
    pub y: DoublyLinkedList<Cell>,

    pub column_headers: Vec<Cell>,
    pub column_sizes: Vec<usize>,
    pub row_bounds: Vec<(Cell, Cell)>,

    partial_solution: Vec<Cell>,

    solving_state: SolvingState,
}

#[derive(Debug, Clone)]
pub struct MatrixSize {
    pub x: usize,
    pub y: usize,
}

impl Indexed for Cell {
    fn get_index(&self) -> usize {
        self.0
    }
    fn set_index(&mut self, index: usize) {
        self.0 = index;
    }
}

// Allow indexing column sizes by Cell
impl ops::Index<Cell> for Vec<usize> {
    type Output = usize;
    fn index(&self, index: Cell) -> &usize {
        &self[index.get_index()]
    }
}
impl ops::IndexMut<Cell> for Vec<usize> {
    fn index_mut(&mut self, index: Cell) -> &mut usize {
        &mut self[index.get_index()]
    }
}

// Allow indexing column headers by Cell
impl ops::Index<Cell> for Vec<Cell> {
    type Output = Cell;
    fn index(&self, index: Cell) -> &Cell {
        &self[index.get_index()]
    }
}
impl ops::IndexMut<Cell> for Vec<Cell> {
    fn index_mut(&mut self, index: Cell) -> &mut Cell {
        &mut self[index.get_index()]
    }
}

pub type Solution = Vec<usize>;

#[derive(Debug, Clone)]
pub enum SolvingState {
    Continue,
    Abort,
}

pub const H: Cell = Cell(0);

impl<'a> Matrix {
    pub fn new(size: MatrixSize) -> Self {
        let mut ret = Self {
            x: DoublyLinkedList::with_capacity(size.x + 1),
            y: DoublyLinkedList::with_capacity(size.y + 1),

            column_headers: Vec::with_capacity(size.x + 1),
            column_sizes: Vec::with_capacity(size.x + 1),
            row_bounds: Vec::with_capacity(size.y + 1),

            partial_solution: Vec::new(),

            solving_state: SolvingState::Continue,
        };

        // allocate root column that will only ever contain the matrix root H
        assert_eq!(ret.alloc_column(), H);

        // allocate the columns
        for _ in 0..size.x {
            ret.add_column();
        }
        ret
    }

    fn alloc_cell(&mut self) -> Cell {
        // allocate a cell in both linked lists and ensure their index is the same
        let cell = self.x.alloc();
        assert_eq!(self.y.alloc(), cell);

        cell
    }

    fn alloc_column(&mut self) -> Cell {
        let cell = self.alloc_cell();

        // keep track of the column header index and column size
        self.column_headers.push(cell);
        self.column_sizes.push(0);

        cell
    }

    fn add_column(&mut self) {
        let cell = self.alloc_column();

        // insert a new column header at the end of the header row
        self.x.insert(self.x[H].prev, cell);
    }

    pub fn add_row(&mut self, row: &[bool]) {
        // row length should be the number of column headers wihtout the root column
        assert_eq!(row.len(), self.column_sizes.len() - 1);

        let mut column_header = H;
        let mut prev_cell = None;
        let mut row_start = None;
        let mut row_end = None;

        for &is_one in row {
            column_header = self.x[column_header].next;

            if is_one {
                // set the column header for a curent cell
                self.column_headers.push(column_header);

                // increment column size
                self.column_sizes[column_header] += 1;

                // first cell is linked to itself
                let cell = self.alloc_cell();

                if let Some(prev_cell) = prev_cell {
                    // if it is not the first cell in this row, we insert it after the previous
                    self.x.insert(prev_cell, cell);
                } else {
                    row_start = Some(cell);
                    prev_cell = Some(cell);
                }

                row_end = Some(cell);

                // insert the new cell at the end of the column
                self.y.insert(self.y[column_header].prev, cell);
            }
        }
        self.row_bounds.push((
            row_start.expect("rows cannot be empty"),
            row_end.expect("rows cannot be empty"),
        ));
    }

    fn remove_row(&mut self, row: Cell) {
        let mut iter_columns = self.x.iter(row);
        while let Some(column) = iter_columns.next(&self.x) {
            // remove the vertical link
            self.y.remove(column);
            self.column_sizes[self.column_headers[column]] -= 1;
        }
    }

    fn restore_row(&mut self, row: Cell) {
        let mut iter_columns = self.x.iter(row);
        while let Some(column) = iter_columns.prev(&self.x) {
            // restore the vectical link
            self.column_sizes[self.column_headers[column]] += 1;
            self.y.restore(column);
        }
    }

    fn cover_column(&mut self, column: Cell) {
        // to cover a column is to delete all rows that have one's in this column, and then delete the column itself

        let mut iter_rows = self.y.iter(column);
        while let Some(row) = iter_rows.next(&self.y) {
            self.remove_row(row);
        }

        // by this point there is no way to access the smallest column through horizontal iteration
        // because all rows that lead to it are excluded from vertical iteration
        // so we can just unlink it from header row
        self.x.remove(column);
    }

    fn uncover_column(&mut self, column: Cell) {
        let mut iter_rows = self.y.iter(column);
        while let Some(row) = iter_rows.prev(&self.y) {
            self.restore_row(row);
        }

        self.x.restore(column);
    }

    pub fn solve<F>(&mut self, callback: &mut F)
    where
        F: FnMut(Solution) -> SolvingState,
    {
        match self.solving_state {
            SolvingState::Continue => {}
            SolvingState::Abort => {
                return;
            }
        }
        // choose a collumn with the least amount of one's
        let mut iter = self.x.iter(H);
        let mut smallest_column = match iter.next(&self.x) {
            Some(cell) => cell,
            None => {
                // if there are no columns, the current partial_solution is correct
                let mut solution = vec![];

                for row in self.partial_solution.iter() {
                    for (index, (start, end)) in self.row_bounds.iter().enumerate() {
                        if row >= start && row <= end {
                            solution.push(index);
                        }
                    }
                }

                self.solving_state = callback(solution);
                // println!("solution found");
                return;
            }
        };

        while let Some(cell) = iter.next(&self.x) {
            if self.column_sizes[cell] < self.column_sizes[smallest_column] {
                smallest_column = cell;
            }
        }

        // WINDING

        self.cover_column(smallest_column);
        // println!("cover smallest column");

        // for each row in the smallest column
        let mut iter_rows = self.y.iter(smallest_column);
        while let Some(row) = iter_rows.next(&self.y) {
            // push row candidate
            self.partial_solution.push(row);
            // println!("push partial");

            // for each column that this row covers
            let mut iter_columns = self.x.iter(row);
            while let Some(column) = iter_columns.next(&self.x) {
                self.cover_column(self.column_headers[column]);
                // println!("cover column");
            }

            // RECURR
            self.solve(callback);

            // UNWINDING

            // for each column that this row covers
            let mut iter_columns = self.x.iter(row);
            while let Some(column) = iter_columns.prev(&self.x) {
                self.uncover_column(self.column_headers[column]);
                // println!("uncover column");
            }
            // pop row candidate as it proved incorrect
            self.partial_solution.pop();
            // println!("pop partial");
        }

        self.uncover_column(smallest_column);

        // println!("uncover smallest column");
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "s: ")?;
        for s in &self.column_sizes {
            write!(f, "{:^7}", s)?;
        }
        writeln!(f)?;

        write!(f, "c: ")?;
        for &Cell(c) in &self.column_headers {
            write!(f, "{:^7}", c)?;
        }
        writeln!(f)?;

        write!(f, "x: ")?;
        for link in &self.x.data {
            write!(f, " {:>2}|{:<2} ", link.prev.0, link.next.0)?
        }
        writeln!(f)?;

        write!(f, "y: ")?;
        for link in &self.y.data {
            write!(f, " {:>2}|{:<2} ", link.prev.0, link.next.0)?
        }
        writeln!(f)?;

        write!(f, "i: ")?;
        for i in 0..self.x.data.len() {
            write!(f, "{:^7}", i)?;
        }
        writeln!(f)?;

        Ok(())
    }
}

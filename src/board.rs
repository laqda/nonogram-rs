use rand::Rng;
use std::rc::Rc;
use std::cmp;
use std::cell::{RefCell, Ref, RefMut};

#[derive(Debug)]
pub struct Board {
    pub grid: Grid,
    pub cursor: Cursor,
    pub lives: u16,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Board {
        Board {
            grid: Grid::new(width, height),
            cursor: Cursor {
                x: 0,
                y: 0,
                max_x: width - 1,
                max_y: height - 1,
            },
            lives: 3,
        }
    }

    pub fn mark(&mut self, x: usize, y: usize) -> Result<(), NonogramErrors> {
        match self.grid.get_cell_mut(x, y).unwrap().mark() {
            Ok(v) => Ok(v),
            Err(e) => match e {
                NonogramErrors::PutMarkInWrongSpot { x: _, y: _ } => {
                    self.lives -= 1;
                    if self.lives > 0 {
                        Ok(())
                    } else {
                        Err(e)
                    }
                }
                _ => Err(e),
            }
        }
    }

    pub fn none(&mut self, x: usize, y: usize) -> Result<(), NonogramErrors> {
        match self.grid.get_cell_mut(x, y).unwrap().none() {
            Ok(v) => Ok(v),
            Err(e) => match e {
                NonogramErrors::PutNoneInWrongSpot { x: _, y: _ } => {
                    self.lives -= 1;
                    if self.lives > 0 {
                        Ok(())
                    } else {
                        Err(e)
                    }
                }
                _ => Err(e),
            }
        }
    }
}

#[derive(Debug)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    max_x: usize,
    max_y: usize,
}

impl Cursor {
    pub fn left(&mut self) {
        if self.x > 0 {
            self.x -= 1;
        }
    }
    pub fn right(&mut self) {
        if self.x < self.max_x {
            self.x += 1;
        }
    }
    pub fn up(&mut self) {
        if self.y > 0 {
            self.y -= 1;
        }
    }
    pub fn down(&mut self) {
        if self.y < self.max_y {
            self.y += 1;
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    rows: Vec<Line>,
    columns: Vec<Line>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        let mut rows = vec![Line::new(width, 2); height];
        let mut columns = vec![Line::new(height, 1); width];

        for i in 0..width {
            for j in 0..height {
                let cell: Rc<RefCell<Cell>> = Rc::new(RefCell::new(Cell::new(i, j)));
                rows[j].cells[i] = Some(cell.clone());
                columns[i].cells[j] = Some(cell.clone());
            }
        }

        rows.iter_mut().for_each(|row| row.update_indications());
        columns.iter_mut().for_each(|column| column.update_indications());

        Grid {
            width,
            height,
            rows,
            columns,
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<Ref<Cell>> {
        match &self.rows[y].cells[x] {
            Some(c) => Some(c.borrow()),
            None => None
        }
    }
    pub fn get_cell_mut(&self, x: usize, y: usize) -> Option<RefMut<Cell>> {
        match &self.rows[y].cells[x] {
            Some(c) => Some(c.borrow_mut()),
            None => None
        }
    }

    pub fn get_row(&self, y: usize) -> Option<&Line> {
        self.rows.get(y)
    }

    pub fn get_column(&self, x: usize) -> Option<&Line> {
        self.columns.get(x)
    }

    pub fn get_indications_max_char_space_needed_rows(&self) -> usize {
        get_indications_max_char_space_needed_lines(&self.rows)
    }
    pub fn get_indications_max_char_space_needed_columns(&self) -> usize {
        get_indications_max_char_space_needed_lines(&self.columns)
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    cells: Vec<Option<Rc<RefCell<Cell>>>>,
    pub indications: Vec<i32>,
    space_equivalent: u16,
}

impl Line {
    fn new(size: usize, space_equivalent: u16) -> Line {
        Line {
            cells: vec![None; size],
            indications: Vec::new(),
            space_equivalent,
        }
    }

    fn update_indications(&mut self) {
        let mut indications = Vec::new();
        let mut last_cell_active_distance = 0;

        self.cells.iter().for_each(|cell| {
            let _cell = cell.as_ref().unwrap().borrow();
            if _cell.active {
                last_cell_active_distance += 1;
            } else {
                if last_cell_active_distance != 0 {
                    indications.push(last_cell_active_distance);
                }
                last_cell_active_distance = 0;
            }
        });
        if last_cell_active_distance != 0 {
            indications.push(last_cell_active_distance);
        }

        self.indications = indications.into_iter().rev().collect();
    }

    pub fn get_indications_as_string(&self) -> String {
        let mut initial_space = "".to_string();
        for _ in 0..self.space_equivalent {
            initial_space.push_str(" ");
        }
        self.indications.iter().fold(initial_space, |s, indication| {
            let mut result = s;
            result.push_str(indication.to_string().chars().rev().collect::<String>().as_str());
            for _ in 0..self.space_equivalent {
                result.push_str(" ");
            }
            result
        })
    }
}

//  For each line, the function will calculate the 'indications_max_char_space_needed_lines' and return the maximum.
//  ex: for an indications vec such as [2, 13, 4], the space needed is 8 because we need to display " 2 13 4 " witch as a len of 8.
fn get_indications_max_char_space_needed_lines(lines: &Vec<Line>) -> usize {
    lines.iter().fold(0, |max, row| {
        let space = row.space_equivalent as usize + row.indications.iter().fold(row.space_equivalent as usize, |space, indication| {
            space + row.space_equivalent as usize + (*indication).to_string().len()
        });
        cmp::max(max, space)
    })
}

#[derive(Debug, Copy, Clone)]
pub enum Status {
    EMPTY,
    NONE,
    MARKED,
}

#[derive(Debug, Copy, Clone)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
    pub status: Status,
    pub active: bool,
}

impl Cell {
    fn new(x: usize, y: usize) -> Cell {
        Cell {
            x,
            y,
            status: Status::EMPTY,
            active: match rand::thread_rng().gen_range(0, 10) {
                0..=3 => false, // 40% chance to be false
                _ => true,
            },
        }
    }

    fn mark(&mut self) -> Result<(), NonogramErrors> {
        match self.status {
            Status::EMPTY => match self.active {
                true => {
                    self.status = Status::MARKED;
                    Ok(())
                }
                false => Err(NonogramErrors::PutMarkInWrongSpot { x: self.x, y: self.y }),
            }
            _ => Ok(())
        }
    }

    fn none(&mut self) -> Result<(), NonogramErrors> {
        match self.status {
            Status::EMPTY => match self.active {
                false => {
                    self.status = Status::NONE;
                    Ok(())
                }
                true => Err(NonogramErrors::PutNoneInWrongSpot { x: self.x, y: self.y }),
            }
            _ => Ok(())
        }
    }
}

// Errors

#[derive(Debug, Fail)]
pub enum NonogramErrors {
    #[fail(display = "the player put a mark on the cell ({},{}) and he is wrong", x, y)]
    PutMarkInWrongSpot {
        x: usize,
        y: usize,
    },
    #[fail(display = "the player put a none on the cell ({},{}) and he is wrong", x, y)]
    PutNoneInWrongSpot {
        x: usize,
        y: usize,
    },
}
